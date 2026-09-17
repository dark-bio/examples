"""Count headers and records in the decompressed VCF view."""
import sys

LENS = "v1/genome/snp-indel"


def main():
    if len(sys.argv) < 2:
        print('[package]\nname = "vcf-mini"\nversion = "0.1.0"\n'
              f'datasets = ["{LENS}"]')
        return

    headers = records = 0
    first = None
    # Keep one line at a time so a whole-genome call file need not fit in memory.
    with open(f"{sys.argv[1]}/{LENS}/vcf", encoding="utf-8", newline="\n") as file:
        for line in file:
            if line.endswith("\n"):
                line = line[:-1].removesuffix("\r")
            if line.startswith("#"):
                headers += 1
            elif line:
                records += 1
                if first is None:
                    first = " ".join(line.split("\t")[:5])

    print("## Variant file\n")
    print(f"- Header lines: {headers}")
    print(f"- Variant records: {records}")
    if first is not None:
        print(f"- First record: `{first}`")


if __name__ == "__main__":
    try:
        main()
    except (OSError, UnicodeError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
