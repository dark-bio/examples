// Cilantro taste, from one variant.
//
// Some people taste cilantro as soap. The SNP rs72921001, near the OR6A2
// olfactory receptor, tracks the trait: the more copies of the C allele you
// carry, the more soapy cilantro tends to taste (Eriksson et al., 2012).
//
// It is the same program as ../03-cilantro-mini-rust, in C. The rsids/ lens hands
// back the genotype and coordinate as plain files; the app never opens a
// variant file. See ../../docs/04-data-access.md.
#include <stdio.h>
#include <string.h>

#define RSID "rs72921001"
#define SOAPY_ALLELE "C"
#define LENS "v1/genome/rsids/rs72921001"

// Read a lens leaf into buf and trim trailing whitespace. Returns its length,
// or -1 if the file is absent. The data directory is the run pass argument.
static int read_leaf(const char *dir, const char *name, char *buf, int cap) {
  const char *sep = (dir[0] && dir[strlen(dir) - 1] == '/') ? "" : "/";
  char path[1024];
  snprintf(path, sizeof(path), "%s%s" LENS "/%s", dir, sep, name);

  FILE *file = fopen(path, "r");
  if (!file) {
    return -1;
  }
  int n = (int)fread(buf, 1, cap - 1, file);
  fclose(file);
  while (n > 0 && (buf[n - 1] == '\n' || buf[n - 1] == '\r' || buf[n - 1] == ' ')) {
    n--;
  }
  buf[n] = '\0';
  return n;
}

int main(int argc, char *argv[]) {
  // The manifest pass: name the app and ask for the one variant directory.
  if (argc < 2) {
    printf("[package]\n"
           "name = \"cilantro-mini\"\n"
           "version = \"0.1.0\"\n"
           "datasets = [\"" LENS "\"]\n");
    return 0;
  }

  // The run pass: read the genotype leaf. A missing file means your data does
  // not cover the site, so there is nothing to report.
  char genotype[64];
  if (read_leaf(argv[1], "genotype", genotype, sizeof(genotype)) < 0) {
    printf("## Cilantro taste\n\n");
    printf("Your data does not cover `" RSID "`, so there is nothing to report.\n");
    return 0;
  }

  // The lens returns alleles as bases (C/C, or C|C when phased), so split on the
  // separators and count copies of the soapy allele. No REF/ALT index to decode.
  int copies = 0;
  char work[64];
  snprintf(work, sizeof(work), "%s", genotype);
  for (char *allele = strtok(work, "/|"); allele; allele = strtok(NULL, "/|")) {
    if (strcmp(allele, SOAPY_ALLELE) == 0) {
      copies++;
    }
  }

  printf("## Cilantro taste\n\n");
  printf("Your genotype at `" RSID "` is `%s`.\n\n", genotype);
  if (copies == 0) {
    printf("You carry no copies of the soapy `" SOAPY_ALLELE "` allele. "
           "Cilantro probably tastes fresh and herby.\n");
  } else if (copies == 1) {
    printf("You carry one copy of the soapy `" SOAPY_ALLELE "` allele. "
           "Cilantro may have a faint soapy note.\n");
  } else {
    printf("You carry two copies of the soapy `" SOAPY_ALLELE "` allele. "
           "Cilantro likely tastes like dish soap.\n");
  }

  // The coordinate, read from the lens's other leaves, shown for context.
  char chrom[32], pos[32], reference[32];
  if (read_leaf(argv[1], "chromosome", chrom, sizeof(chrom)) > 0 &&
      read_leaf(argv[1], "position", pos, sizeof(pos)) > 0) {
    read_leaf(argv[1], "reference", reference, sizeof(reference));
    printf("\nLocus `%s:%s`, reference allele `%s`.\n", chrom, pos, reference);
  }

  return 0;
}
