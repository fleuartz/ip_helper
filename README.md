# Interactive問題デバッグ補助ツール
`mkfifo fifo && (python judge.py < fifo) | (./solver > fifo)` の代わりです．

## 使用例
```
$ ip_helper ./solve 'python judge.py'
[2 → 1] 39
[1 → 2] 6
[1 → 2] 19 1 3 5 7 9 11 13 15 17 19 21 23 25 27 29 31 33 35 37
[1 → 2] 19 2 3 6 7 10 11 14 15 18 19 22 23 26 27 30 31 34 35 38
[1 → 2] 19 4 5 6 7 12 13 14 15 20 21 22 23 28 29 30 31 36 37 38
[1 → 2] 16 8 9 10 11 12 13 14 15 24 25 26 27 28 29 30 31
[1 → 2] 16 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
[1 → 2] 7 32 33 34 35 36 37 38
[2 → 1] 011110
[1 → 2] 30
[2 → 1] AC
```
[使用問題](https://atcoder.jp/contests/abc337/tasks/abc337_e)

```
$ ip_helper -e ./solve 'python judge.py'
[2 → 1] 34
[1 → 2] ? 17
[2 → 1] 1
[Error][proc1] left: 1 | right: 17
[1 → 2] ? 9
[2 → 1] 0
[Error][proc1] left: 9 | right: 17
[1 → 2] ? 13
[2 → 1] 0
[Error][proc1] left: 13 | right: 17
[1 → 2] ? 15
[2 → 1] 1
[Error][proc1] left: 13 | right: 15
[1 → 2] ? 14
[2 → 1] 0
[1 → 2] ! 14
[2 → 1] AC
[2 → 1] Quary num: 6
```
[使用問題](https://atcoder.jp/contests/abc299/tasks/abc299_d)