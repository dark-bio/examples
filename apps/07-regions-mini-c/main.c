// One interval's reference length and GC content through the regions/ lens.
#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define REGION "chrM:1-16569"
#define LENS "v1/genome/regions/chrM/1-16569"

int main(int argc, char *argv[]) {
  if (argc < 2) {
    printf("manifest = 1\n\n[app]\nname = \"One Region\"\nversion = \"0.1.0\"\n\n"
           "[reads]\npaths = [\"" LENS "\"]\n\n"
           "[listing]\n"
           "language = \"en\"\n"
           "icon = \"📏\"\n"
           "summary = \"Stream an interval's sequence without ever loading it whole.\"\n"
           "category = \"developer\"\n"
           "license = \"BSD-3-Clause\"\n"
           "source = \"https://github.com/dark-bio/examples/tree/main/apps/07-regions-mini-c\"\n"
           "keywords = [\"tutorial\", \"regions\", \"streaming\", \"mitochondria\"]\n"
           "description = '''\n"
           "Grant any interval as `v1/genome/regions/<chr>/<start>-<end>`. This app takes the whole mitochondrial genome, `chrM:1-16569`, and streams its `sequence` in 8 KiB chunks to report its length and GC content. Memory use does not grow with the interval.\n\n"
           "Repeats are soft masked in lowercase, so count both cases. A failed read stops the app.\n\n"
           "An interval grant covers your variants inside it, which Powerhouse of the Cell lists.\n"
           "'''\n\n"
           "[listing.purposes]\n"
           "\"v1/genome/regions/chrM/1-16569\" = \"The mitochondrial genome, to stream its sequence\"\n");
    return 0;
  }
  size_t size = strlen(argv[1]) + sizeof(LENS) + sizeof("/sequence") + 1;
  char *path = malloc(size);
  if (!path) {
    fprintf(stderr, "Could not allocate a region path\n");
    return 1;
  }
  snprintf(path, size, "%s/" LENS "/sequence", argv[1]);
  FILE *sequence = fopen(path, "rb");
  int error = errno;
  free(path);
  if (!sequence) {
    fprintf(stderr, "Could not open region sequence: %s\n", strerror(error));
    return 1;
  }
  uint64_t length = 0, gc = 0;
  unsigned char buffer[8192];
  // Forward-strand sequence has no newline; lowercase bases are soft-masked.
  size_t count;
  while ((count = fread(buffer, 1, sizeof(buffer), sequence)) != 0) {
    length += count;
    for (size_t i = 0; i < count; i++) {
      unsigned char base = buffer[i];
      if (base == 'G' || base == 'g' || base == 'C' || base == 'c') gc++;
    }
  }
  if (ferror(sequence)) {
    fprintf(stderr, "Could not read region sequence: %s\n", strerror(errno));
    fclose(sequence);
    return 1;
  }
  if (fclose(sequence) != 0) {
    fprintf(stderr, "Could not close region sequence: %s\n", strerror(errno));
    return 1;
  }
  if (length != 16569) {
    fprintf(stderr, "Region sequence length does not match its span\n");
    return 1;
  }
  printf("## Region " REGION "\n\n");
  printf("- Reference length: %" PRIu64 " bp\n", length);
  printf("- GC content: %.1f%%\n", 100.0 * (double)gc / (double)length);
  return 0;
}
