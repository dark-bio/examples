"""One interval's reference length and GC content through the regions/ lens."""
import sys

REGION = "chrM:1-16569"
LENS = "v1/genome/regions/chrM/1-16569"


def main():
    if len(sys.argv) < 2:
        print(f"""manifest = 1

[app]
name = "One Region"
version = "0.1.0"

[reads]
paths = ["{LENS}"]

[listing]
language = "en"
icon = "📏"
summary = "Stream an interval's sequence without ever loading it whole."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/07-regions-mini-python"
keywords = ["tutorial", "regions", "streaming", "mitochondria"]
description = '''
Grant any interval as `v1/genome/regions/<chr>/<start>-<end>`. This app takes the whole mitochondrial genome, `chrM:1-16569`, and streams its `sequence` in 8 KiB chunks to report its length and GC content. Memory use does not grow with the interval.

Repeats are soft masked in lowercase, so count both cases. A failed read stops the app.

An interval grant covers your variants inside it, which Powerhouse of the Cell lists.
'''

[listing.purposes]
"v1/genome/regions/chrM/1-16569" = "The mitochondrial genome, to stream its sequence"
""", end="")
        return

    length = gc = 0
    # Forward-strand sequence has no newline; lowercase bases are soft-masked.
    with open(f"{sys.argv[1]}/{LENS}/sequence", "rb") as sequence:
        while chunk := sequence.read(8192):
            length += len(chunk)
            gc += sum(chunk.count(base) for base in (b"G", b"g", b"C", b"c"))
    if length != 16569:
        raise ValueError("Region sequence length does not match its span")

    print(f"## Region {REGION}\n")
    print(f"- Reference length: {length} bp")
    print(f"- GC content: {100.0 * gc / length:.1f}%")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
