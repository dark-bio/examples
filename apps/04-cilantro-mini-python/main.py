"""Cilantro taste, from one variant through rsids/."""
import sys

RSID = "rs72921001"
SOAPY_ALLELE = "C"
LENS = "v1/genome/rsids/rs72921001"


def leaf(base, name):
    # Generated files hold exactly their value; only absence means no answer.
    try:
        with open(f"{base}/{name}", encoding="utf-8") as file:
            return file.read()
    except FileNotFoundError:
        return None
    except OSError as error:
        raise OSError(f"Could not read {RSID} {name}: {error}") from error


def main():
    if len(sys.argv) < 2:
        print(f"""manifest = 1

[app]
name = "One Variant"
version = "0.1.0"

[reads]
paths = ["{LENS}"]

[listing]
language = "en"
icon = "🌱"
summary = "One variant, four languages, zero call file. The smallest real read."
category = "developer"
license = "BSD-3-Clause"
source = "https://github.com/dark-bio/examples/tree/main/apps/04-cilantro-mini-python"
keywords = ["tutorial", "rsids", "genotype", "four languages"]
description = '''
The `rsids/` folder hands you one variant as plain files. Grant `v1/genome/rsids/rs72921001`, read `genotype`, `chromosome`, `position` and `reference`, and you're done. The call file never comes near it.

Every app handles three outcomes. A missing allele, as in `./.`, makes the count inconclusive. A missing genotype is no answer, never two reference alleles. Any other read error stops the app.

Written in Rust, Go, C and Python. Compare them, then meet the full report, Cilantro Taste Test.
'''

[listing.purposes]
"v1/genome/rsids/rs72921001" = "The one variant this tutorial reads"
""", end="")
        return

    base = f"{sys.argv[1]}/{LENS}"
    genotype = leaf(base, "genotype")
    if genotype is None:
        print("## Cilantro or soap?\n")
        print(f"No genotype at `{RSID}`. "
              "Silence isn't two reference alleles.")
        return

    # This known SNP needs a simple split, including a possible leading phase marker.
    alleles = genotype.lstrip("/|").replace("|", "/").split("/")
    copies = alleles.count(SOAPY_ALLELE)
    print("## Cilantro or soap?\n")
    print(f"Your genotype at `{RSID}` is `{genotype}`.\n")
    if "." in alleles:
        print("An allele is missing (`.`), so the soap count is inconclusive.")
    elif copies == 0:
        print(f"You carry no copies of the soapy `{SOAPY_ALLELE}` allele. "
              "Cilantro probably tastes fresh and herby.")
    elif copies == 1:
        print(f"You carry one copy of the soapy `{SOAPY_ALLELE}` allele. "
              "Cilantro may have a faint soapy note.")
    else:
        print(f"You carry {copies} copies of the soapy `{SOAPY_ALLELE}` allele. "
              "Cilantro likely tastes like dish soap.")

    names = ("chromosome", "position", "reference")
    chrom, pos, reference = (leaf(base, name) for name in names)
    if chrom is not None and pos is not None and reference is not None:
        print(f"\nLocus `{chrom}:{pos}`, reference allele `{reference}`.")


if __name__ == "__main__":
    try:
        main()
    except (OSError, UnicodeError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
