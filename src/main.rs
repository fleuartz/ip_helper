mod error;
use error::ProcessCommunicationError;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
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
            .spawn()
            .map_err(|e| ProcessCommunicationError::CommandError(format!("Failed to spawn {}: {}", label, e)))
    }

    fn establish_data_pipeline(&mut self) -> Result<(), ProcessCommunicationError> {
        let proc1_pipes = Self::extract_pipes(&mut self.proc1, "proc1")?;
        let proc2_pipes = Self::extract_pipes(&mut self.proc2, "proc2")?;

        let proc1_to_proc2 = Self::pipe_data(proc1_pipes.0, proc2_pipes.1, "1 → 2".to_string(), self.capture_stderr)?;
        let proc2_to_proc1 = Self::pipe_data(proc2_pipes.0, proc1_pipes.1, "2 → 1".to_string(), self.capture_stderr)?;

        proc1_to_proc2.join().map_err(|_| ProcessCommunicationError::ThreadJoinError)??;
        proc2_to_proc1.join().map_err(|_| ProcessCommunicationError::ThreadJoinError)??;

        Ok(())
    }

    fn extract_pipes(child: &mut Child, label: &str) -> Result<(impl std::io::Read, Arc<Mutex<impl Write>>), ProcessCommunicationError> {
        let stdout = child.stdout.take().ok_or_else(|| ProcessCommunicationError::CommandError(format!("Failed to take {} stdout", label)))?;
        let stdin = Arc::new(Mutex::new(child.stdin.take().ok_or_else(|| ProcessCommunicationError::CommandError(format!("Failed to take {} stdin", label)))?));
        Ok((stdout, stdin))
    }

    fn pipe_data<R, W>(reader: R, writer: Arc<Mutex<W>>, label: String, capture_stderr: bool) -> Result<thread::JoinHandle<Result<(), ProcessCommunicationError>>, ProcessCommunicationError>
    where
        R: 'static + Read + Send,
        W: 'static + Write + Send, {
        Ok(thread::spawn(move || -> Result<(), ProcessCommunicationError> {
            let mut buf_reader = BufReader::new(reader);
            let mut line = String::new();
            while buf_reader.read_line(&mut line)? > 0 {
                if !capture_stderr {
                    eprintln!("[{}] {}", label, line.trim_end());
                }
                let mut locked_writer = writer.lock().map_err(|_| ProcessCommunicationError::MutexLockError)?;
                locked_writer.write_all(line.as_bytes())?;
                locked_writer.flush()?;
                line.clear();
            }
            Ok(())
        }))
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "process_communicator")]
struct Opt {
    /// Activate standard error capture
    #[structopt(short, long)]
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
