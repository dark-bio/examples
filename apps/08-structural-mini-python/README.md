# 08 - structural-mini (Python)

[08-structural-mini-rust](../08-structural-mini-rust) written in Python. It
lists the changes with `os.listdir`, sorts them by span, and takes the new bases
from `os.stat` on `bases`. A missing leaf arrives as `FileNotFoundError` and
reads as no answer. [01-hello-python](../01-hello-python) covers what a Python
module costs.

## Build and run

```sh
make run APP=08-structural-mini-python
make run APP=08-structural-mini-python FIXTURES=fixtures/no-call
make run APP=08-structural-mini-python FIXTURES=fixtures/unanswered
```
