# 03 - cilantro-mini (Python)

[03-cilantro-mini-rust](../03-cilantro-mini-rust) written in Python. An absent
genotype arrives as `FileNotFoundError` and reads as no answer, while every
other `OSError` stops the app. [01-hello-python](../01-hello-python) covers what
a Python module costs.

## Build and run

```sh
make run APP=03-cilantro-mini-python
make run APP=03-cilantro-mini-python FIXTURES=fixtures/no-call
make run APP=03-cilantro-mini-python FIXTURES=fixtures/unanswered
```
