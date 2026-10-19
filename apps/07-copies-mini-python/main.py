"""One gene's copy number through genes/."""
import os
import sys

LENS = "v1/genome/genes/UGT2B17"


def span(name):
    """Splits a segment name, start-end, into its first and last base."""
    start, end = name.split("-", 1)
    return int(start), int(end)


def main():
    if len(sys.argv) < 2:
        print(
            """manifest = 1

[app]
name = "Copy Number"
version = "0.1.0"

[reads]
paths = ["v1/genome/genes/UGT2B17"]

[listing]
language = "en"
icon = "🔢"
summary = "A gene's copy number, read from its segments, in four languages."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/07-copies-mini-python"
keywords = ["tutorial", "copies", "copy number", "UGT2B17"]
description = '''
Grant `v1/genome/genes/UGT2B17` and list its `copies`. Each directory is one segment of your copy number file that reaches the gene, named after its span, and its `count` holds how many copies your genome has there.

A whole gene can go missing, and a small variant file never shows it. An empty listing is no answer, never zero copies, since a caller can leave a region out.

A gene grant also covers your variants inside the gene, even though this app does not read them. Doping Test turns the same read into a report.
'''

[listing.purposes]
"v1/genome/genes/UGT2B17" = "A gene to read your copy number over"
""",
            end="",
        )
        return

    # A missing listing has no answer, just like an empty one
    base = f"{sys.argv[1]}/{LENS}/copies"
    try:
        names = os.listdir(base)
    except FileNotFoundError:
        names = []
    segments = []
    for name in names:
        coordinates = span(name)

        # Only a missing count means no answer, and other failures stop the app
        try:
            with open(f"{base}/{name}/count", encoding="utf-8") as file:
                count = file.read()
        except FileNotFoundError:
            count = None
        segments.append((coordinates, name, count))
    # Listing order isn't promised, so sort by the first base before printing
    segments.sort()

    print("## Copies of UGT2B17\n")
    if not segments:
        print(
            "No copy number segment reaches UGT2B17. That is no answer, not zero copies."
        )
    for _, name, count in segments:
        answer = (
            "no answer"
            if count is None
            else f"{count} {'copy' if count == '1' else 'copies'}"
        )
        print(f"- Segment {name}: {answer}")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
