mod error;
use error::ProcessCommunicationError;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;
use std::thread;
use structopt::StructOpt;

struct ProcessCommunicator {
    proc1: Child,
    proc2: Child,
    capture_stderr: bool,
}

impl ProcessCommunicator {
    fn new(cmd1: &str, cmd2: &str, capture_stderr: bool) -> Result<Self, ProcessCommunicationError> {
        let proc1 = Self::spawn_process(cmd1, "proc1")?;
        let proc2 = Self::spawn_process(cmd2, "proc2")?;
        Ok(ProcessCommunicator { proc1, proc2, capture_stderr })
    }

    fn spawn_process(command: &str, label: &str) -> Result<Child, ProcessCommunicationError> {
        Command::new("sh")
            .args(&["-c", command])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| ProcessCommunicationError::CommandError(format!("Failed to spawn {}: {}", label, e)))
    }

    fn establish_data_pipeline(&mut self) -> Result<(), ProcessCommunicationError> {
        let proc1_pipes = Self::extract_pipes(&mut self.proc1, "proc1")?;
        let proc2_pipes = Self::extract_pipes(&mut self.proc2, "proc2")?;
    
        let mut proc1_stderr_thread: Option<thread::JoinHandle<Result<(), ProcessCommunicationError>>> = None;
        let mut proc2_stderr_thread: Option<thread::JoinHandle<Result<(), ProcessCommunicationError>>> = None;

    
        let proc1_to_proc2 = Self::pipe_data(proc1_pipes.0, proc2_pipes.1, "1 → 2".to_string())?;
        let proc2_to_proc1 = Self::pipe_data(proc2_pipes.0, proc1_pipes.1, "2 → 1".to_string())?;
    
        if self.capture_stderr {
            let proc1_stderr = self.proc1.stderr.take().ok_or_else(|| ProcessCommunicationError::CommandError("Failed to take proc1 stderr".to_string()))?;
            let proc2_stderr = self.proc2.stderr.take().ok_or_else(|| ProcessCommunicationError::CommandError("Failed to take proc2 stderr".to_string()))?;
    
            proc1_stderr_thread = Some(Self::pipe_stderr(proc1_stderr, "proc1".to_string())?);
            proc2_stderr_thread = Some(Self::pipe_stderr(proc2_stderr, "proc2".to_string())?);
        }

        let timeout = Duration::from_secs(5);

        Self::wait_with_timeout(proc1_to_proc2, timeout)?;
        Self::wait_with_timeout(proc2_to_proc1, timeout)?;

        if let Some(thread) = proc1_stderr_thread {
            Self::wait_with_timeout(thread, timeout)?;
        }
        if let Some(thread) = proc2_stderr_thread {
            Self::wait_with_timeout(thread, timeout)?;
        }
    
        Ok(())
    }

    fn extract_pipes(child: &mut Child, label: &str) -> Result<(impl std::io::Read, Arc<Mutex<impl Write>>), ProcessCommunicationError> {
        let stdout = child.stdout.take().ok_or_else(|| ProcessCommunicationError::CommandError(format!("Failed to take {} stdout", label)))?;
        let stdin = Arc::new(Mutex::new(child.stdin.take().ok_or_else(|| ProcessCommunicationError::CommandError(format!("Failed to take {} stdin", label)))?));
        Ok((stdout, stdin))
    }

    fn pipe_data<R, W>(reader: R, writer: Arc<Mutex<W>>, label: String) -> Result<thread::JoinHandle<Result<(), ProcessCommunicationError>>, ProcessCommunicationError>
    where
        R: 'static + Read + Send,
        W: 'static + Write + Send, {
        Ok(thread::spawn(move || -> Result<(), ProcessCommunicationError> {
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();
            while buf_reader.read_line(&mut line)? > 0 {
                eprintln!("[{}] {}", label, line.trim_end());
                let mut locked_writer = writer.lock().map_err(|_| ProcessCommunicationError::MutexLockError)?;
                locked_writer.write_all(line.as_bytes())?;
                locked_writer.flush()?;
                line.clear();
            }
            Ok(())
        }))
    }

    fn pipe_stderr<R>(reader: R, label: String) -> Result<thread::JoinHandle<Result<(), ProcessCommunicationError>>, ProcessCommunicationError>
    where
        R: 'static + Read + Send {
        Ok(thread::spawn(move || -> Result<(), ProcessCommunicationError> {
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();
            while buf_reader.read_line(&mut line)? > 0 {
                eprintln!("[  {}  ] {}", if label == "proc1" { 1 } else { 2 }, line.trim_end());
                line.clear();
            }
            Ok(())
        }))
    }

    fn wait_with_timeout<T>(thread: thread::JoinHandle<Result<T, ProcessCommunicationError>>, timeout: Duration) -> Result<T, ProcessCommunicationError>
    where
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let result = thread.join().map_err(|_| ProcessCommunicationError::ThreadJoinError);
            tx.send(result).expect("Could not send data through channel.");
        });

        match rx.recv_timeout(timeout) {
            Ok(Ok(result)) => Ok(result?),
            Ok(Err(_)) => Err(ProcessCommunicationError::ThreadPanicked),
            Err(mpsc::RecvTimeoutError::Timeout) => Err(ProcessCommunicationError::TimeoutError),
            Err(mpsc::RecvTimeoutError::Disconnected) => Err(ProcessCommunicationError::ChannelDisconnected),
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "process_communicator")]
struct Opt {
    /// Activate standard error capture
    #[structopt(short = "e", long)]
    capture_errors: bool,

    /// Command 1 to execute
    cmd1: String,

    /// Command 2 to execute
    cmd2: String,
}

fn main() -> Result<(), ProcessCommunicationError> {
    let opt = Opt::from_args();

    let mut pair = ProcessCommunicator::new(&opt.cmd1, &opt.cmd2, opt.capture_errors)?;
    pair.establish_data_pipeline()?;

    Ok(())
}