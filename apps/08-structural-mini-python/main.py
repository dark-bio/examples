"""One structural variant through regions/."""

import os
import sys

LENS = "v1/genome/regions/chr17/63488479-63488579"


def leaf(base, name):
    """Reads one scalar, and only a missing file means no answer."""
    try:
        with open(f"{base}/{name}", encoding="utf-8") as file:
            return file.read()
    except FileNotFoundError:
        return None


def span(name):
    """Splits an entry name such as 100-100 or 100-100-2 into start, end and suffix."""
    start, rest = name.split("-", 1)
    end, separator, suffix = rest.partition("-")
    return int(start), int(end), int(suffix) if separator else 1


def new_bases(base):
    """Counts the new bases from the size of bases, or from length when symbolic."""
    try:
        return os.stat(f"{base}/bases").st_size
    except FileNotFoundError:
        length = leaf(base, "length")
        return int(length) if length is not None else None


def main():
    if len(sys.argv) < 2:
        print(
            """manifest = 1

[app]
name = "Structural Variant"
version = "0.1.0"

[reads]
paths = ["v1/genome/regions/chr17/63488479-63488579"]

[listing]
language = "en"
icon = "🧩"
summary = "A structural variant read as plain files, in four languages."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/08-structural-mini-python"
keywords = ["tutorial", "changes", "structural variants", "ACE"]
description = '''
Grant a short window inside the _ACE_ gene and list its `changes`. Each directory is one alternate allele you carry, named after the span it changes. Its `type`, `bases`, `dataset` and `carried` say what changed, into what, in which file and on how many copies.

The window holds the ACE insertion, nearly 300 bases long and too long for a small variant caller, so it comes from your structural variant file. The window spans 101 bases, since callers can place an insertion a few bases apart.

An interval grant also covers your small variants inside it. Sprint or Marathon turns the same read into a report.
'''

[listing.purposes]
"v1/genome/regions/chr17/63488479-63488579" = "101 bases of the ACE gene, around its Alu insertion"
""",
            end="",
        )
        return

    # A missing listing has no answer, just like an empty one
    base = f"{sys.argv[1]}/{LENS}/changes"
    try:
        names = os.listdir(base)
    except FileNotFoundError:
        names = []
    changes = []
    for name in names:
        coordinates = span(name)
        directory = f"{base}/{name}"
        changes.append(
            (
                coordinates,
                name,
                leaf(directory, "type"),
                new_bases(directory),
                leaf(directory, "dataset"),
                leaf(directory, "carried"),
            )
        )
    # Listing order isn't promised, so sort by span before printing
    changes.sort(key=lambda change: change[0])

    print("## Changes near the ACE insertion\n")
    if not changes:
        print(
            "No change reaches this window. That is no answer, since your files may not cover it."
        )
    for _, name, kind, size, dataset, carried in changes:
        bases = (
            "no new bases"
            if size is None
            else f"{size} new {'base' if size == 1 else 'bases'}"
        )
        copies = (
            "no answer"
            if carried is None
            else f"{carried} {'copy' if carried == '1' else 'copies'}"
        )
        kind = kind if kind is not None else "no answer"
        dataset = dataset if dataset is not None else "no answer"
        print(f"- {name}: {kind}, {bases}, {dataset}, carried on {copies}")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
