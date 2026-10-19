# 07 - copies-mini (Python)

[07-copies-mini-rust](../07-copies-mini-rust) written in Python. It lists the
segments with `os.listdir`, sorts them by numeric start, and reads an absent
`count` as no answer when `FileNotFoundError` arrives.
[01-hello-python](../01-hello-python) covers what a Python module costs.

## Build and run

```sh
make run APP=07-copies-mini-python
make run APP=07-copies-mini-python FIXTURES=fixtures/no-call
make run APP=07-copies-mini-python FIXTURES=fixtures/unanswered
```
