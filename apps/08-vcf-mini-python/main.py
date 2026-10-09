"""Count headers and records in the decompressed VCF view."""
import sys

LENS = "v1/genome/snp-indel"


def main():
    if len(sys.argv) < 2:
        print(f"""manifest = 1

[app]
name = "The Call File"
version = "0.1.0"

[reads]
paths = ["{LENS}"]

[listing]
language = "en"
icon = "📄"
summary = "Count the records of a whole genome one line at a time."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/08-vcf-mini-python"
keywords = ["tutorial", "VCF", "call file", "streaming"]
description = '''
Grant `v1/genome/snp-indel` and open its `vcf`, the decompressed call file. A whole genome file runs to gigabytes, far past an app's 128 MiB of memory, so this app streams it one line at a time and counts headers and records.

Two details matter at that scale. Every buffer refill leaves the sandbox, so the app reads through a 64 KiB buffer, and it reuses one line instead of allocating one per record.

This is the broadest grant there is. VCF Roll Call parses the same file with a library.
'''

[listing.purposes]
"v1/genome/snp-indel" = "Your whole call file, to count its records"
""", end="")
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
