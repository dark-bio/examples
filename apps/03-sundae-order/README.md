# 03 - sundae order

An app can ask the owner questions when they approve it. This one asks for a
flavor, any number of toppings and the words on the card, then prints the
sundae. It reads none of the owner's data, so its manifest has no `[reads]`.

Each answer arrives as a file under `inputs/` in the data directory, named after
its input. `flavor` holds the picked choice, `toppings` the picks one per line,
and `card` the text, each without a trailing newline. The report quotes every
file exactly as the app read it. [02-manifest.md](../../docs/02-manifest.md)
covers inputs.

## Build and run

`make run` takes the answers from this app's `inputs/` folder, and `INPUTS`
points it at another directory of answer files.

```sh
make run APP=03-sundae-order
INPUTS=my-answers make run APP=03-sundae-order
```
