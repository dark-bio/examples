"""One interval's reference length and GC content through the regions/ lens."""
import sys

REGION = "chrM:1-16569"
LENS = "v1/genome/regions/chrM/1-16569"


def main():
    if len(sys.argv) < 2:
        print('[package]\nname = "regions-mini"\nversion = "0.1.0"\n'
              f'datasets = ["{LENS}"]')
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
