"""One gene's metadata and reference length through the genes/ lens."""
import os
import sys

GENE = "TAS2R38"
LENS = "v1/genome/genes/TAS2R38"


def leaf(base, name):
    # An absent scalar has no answer; every other read failure stops the app.
    try:
        with open(f"{base}/{name}", encoding="utf-8") as file:
            return file.read()
    except FileNotFoundError:
        return None
    except OSError as error:
        raise OSError(f"Could not read {GENE} {name}: {error}") from error


def coordinate(value):
    if value is None:
        return None
    digits = value.removeprefix("+")
    if not digits or not digits.isascii() or not digits.isdecimal():
        raise ValueError("Invalid gene coordinate")
    number = int(value)
    if number >= 1 << 64:
        raise ValueError("Gene coordinate is too large")
    return number


def main():
    if len(sys.argv) < 2:
        print(f"""manifest = 1

[app]
name = "One Gene"
version = "0.1.0"

[reads]
paths = ["{LENS}"]

[listing]
language = "en"
icon = "🧬"
summary = "A gene's coordinates and sequence length, through the genes lens."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/06-genes-mini-python"
keywords = ["tutorial", "genes", "sequence", "TAS2R38"]
description = '''
Grant `v1/genome/genes/TAS2R38` and read the gene's `chromosome`, `start`, `end`, `strand` and `biotype` as files. The `sequence` file holds exactly end - start + 1 bases, so its size is the gene's length and the gene never has to fit in memory.

An absent value prints as no answer. A missing sequence is an error.

A gene grant also covers your variants inside the gene, even though this app does not read them. Bitter Meter does.
'''

[listing.purposes]
"v1/genome/genes/TAS2R38" = "A gene to read coordinates and sequence length from"
""", end="")
        return

    base = f"{sys.argv[1]}/{LENS}"
    # File size gives the sequence length in bp without loading the gene.
    length = os.stat(f"{base}/sequence").st_size
    if length == 0:
        raise ValueError("The gene sequence is empty")
    chromosome = leaf(base, "chromosome")
    start = coordinate(leaf(base, "start"))
    end = coordinate(leaf(base, "end"))
    strand = leaf(base, "strand")
    biotype = leaf(base, "biotype")
    if start is not None and end is not None:
        if start == 0 or end < start or length != end - start + 1:
            raise ValueError("Gene sequence length does not match its span")

    print(f"## Gene {GENE}\n")
    for name, value in (("Chromosome", chromosome), ("Start", start), ("End", end),
                        ("Strand", strand), ("Biotype", biotype)):
        print(f"- {name}: {value if value is not None else 'no answer'}")
    print(f"- Reference length: {length} bp")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
