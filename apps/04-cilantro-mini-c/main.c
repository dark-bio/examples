// Cilantro taste, from one variant.
//
// Some people taste cilantro as soap. The SNP rs72921001, near the OR6A2
// olfactory receptor, tracks the trait: the more copies of the C allele you
// carry, the more soapy cilantro tends to taste (Eriksson et al., 2012).
//
// This is ../04-cilantro-mini-rust in C. The rsids/ lens hands
// back the genotype and coordinate as plain files; the app never opens a
// variant file. See ../../docs/04-reading-data.md.
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define RSID "rs72921001"
#define SOAPY_ALLELE "C"
#define LENS "v1/genome/rsids/rs72921001"

static void fail(const char *name) {
  fprintf(stderr, "Could not read " RSID " %s: %s\n", name, strerror(errno));
  exit(1);
}

// Generated files hold exactly their value. Only ENOENT returns NULL.
static char *read_leaf(const char *dir, const char *name) {
  size_t path_size = strlen(dir) + sizeof(LENS) + strlen(name) + 3;
  char *path = malloc(path_size);
  if (!path) fail(name);
  snprintf(path, path_size, "%s/" LENS "/%s", dir, name);
  FILE *file = fopen(path, "rb");
  int open_error = errno;
  free(path);
  if (!file) {
    if (open_error == ENOENT) return NULL;
    errno = open_error;
    fail(name);
  }

  // Grow the buffer so long alleles cannot be silently truncated.
  size_t size = 0, capacity = 128;
  char *value = malloc(capacity);
  if (!value) fail(name);
  for (;;) {
    size_t count = fread(value + size, 1, capacity - size - 1, file);
    size += count;
    if (ferror(file)) fail(name);
    if (feof(file)) break;
    if (capacity > SIZE_MAX / 2) {
      errno = ENOMEM;
      fail(name);
    }
    capacity *= 2;
    char *grown = realloc(value, capacity);
    if (!grown) fail(name);
    value = grown;
  }
  if (fclose(file) != 0) fail(name);
  value[size] = '\0';
  return value;
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    printf("manifest = 1\n\n"
           "[app]\n"
           "name = \"One Variant\"\n"
           "version = \"0.1.0\"\n\n"
           "[reads]\n"
           "paths = [\"" LENS "\"]\n\n"
           "[listing]\n"
           "language = \"en\"\n"
           "icon = \"🌱\"\n"
           "summary = \"The smallest real data access, through the rsids lens, in four languages.\"\n"
           "category = \"developer\"\n"
           "license = \"BSD-3-Clause\"\n"
           "source = \"https://github.com/dark-bio/examples/tree/main/apps/04-cilantro-mini-c\"\n"
           "keywords = [\"tutorial\", \"rsids\", \"genotype\", \"four languages\"]\n"
           "description = '''\n"
           "The rsids lens hands back one variant as plain files. Grant `v1/genome/rsids/rs72921001` and read `genotype`, `chromosome`, `position` and `reference` from that directory. The app never opens the call file.\n\n"
           "Three outcomes every app has to handle. A missing allele, as in `./.`, makes a copy count inconclusive. An absent genotype means no answer, never two reference alleles. Any other read error stops the app.\n\n"
           "The same program is written in Rust, Go, C and Python. Compare them, then see the full report version, Cilantro Taste Test.\n"
           "'''\n\n"
           "[listing.purposes]\n"
           "\"v1/genome/rsids/rs72921001\" = \"The one variant this tutorial reads\"\n");
    return 0;
  }

  // ENOENT means no answer, including reference sites in variants-only calls.
  char *genotype = read_leaf(argv[1], "genotype");
  if (!genotype) {
    printf("## Cilantro taste\n\n");
    printf("No genotype answer for `" RSID "`. Absence does not imply two reference alleles.\n");
    return 0;
  }

  // This known SNP needs only a simple split; keep the original text for display.
  size_t copies = 0;
  int missing = 0;
  const char *allele = genotype;
  if (*allele == '/' || *allele == '|') allele++;
  while (*allele) {
    size_t length = strcspn(allele, "/|");
    if (length == 1 && allele[0] == '.') missing = 1;
    if (length == 1 && allele[0] == SOAPY_ALLELE[0]) copies++;
    allele += length;
    if (*allele) allele++;
  }

  printf("## Cilantro taste\n\n");
  printf("Your genotype at `" RSID "` is `%s`.\n\n", genotype);
  if (missing) {
    printf("The copy count is inconclusive because an allele is missing (`.`).\n");
  } else if (copies == 0) {
    printf("You carry no copies of the soapy `" SOAPY_ALLELE "` allele. "
           "Cilantro probably tastes fresh and herby.\n");
  } else if (copies == 1) {
    printf("You carry one copy of the soapy `" SOAPY_ALLELE "` allele. "
           "Cilantro may have a faint soapy note.\n");
  } else {
    printf("You carry %zu copies of the soapy `" SOAPY_ALLELE "` allele. "
           "Cilantro likely tastes like dish soap.\n", copies);
  }

  char *chrom = read_leaf(argv[1], "chromosome");
  char *pos = read_leaf(argv[1], "position");
  char *reference = read_leaf(argv[1], "reference");
  if (chrom && pos && reference) {
    printf("\nLocus `%s:%s`, reference allele `%s`.\n", chrom, pos, reference);
  }
  free(genotype);
  free(chrom);
  free(pos);
  free(reference);
  return 0;
}
