// Count headers and records in the decompressed VCF view.
#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define LENS "v1/genome/snp-indel"

int main(int argc, char *argv[]) {
  if (argc < 2) {
    printf("[package]\nname = \"vcf-mini\"\nversion = \"0.1.0\"\n"
           "datasets = [\"" LENS "\"]\n");
    return 0;
  }
  size_t size = strlen(argv[1]) + sizeof(LENS) + sizeof("/vcf") + 1;
  char *path = malloc(size);
  if (!path) {
    fprintf(stderr, "Could not allocate a VCF path\n");
    return 1;
  }
  snprintf(path, size, "%s/" LENS "/vcf", argv[1]);
  FILE *file = fopen(path, "rb");
  int error = errno;
  free(path);
  if (!file) {
    fprintf(stderr, "Could not open VCF: %s\n", strerror(error));
    return 1;
  }
  // Each refill is a call out of the sandbox, and the default buffer is small,
  // so a whole-genome scan spends most of its time in them without this.
  static char io_buffer[65536];
  if (setvbuf(file, io_buffer, _IOFBF, sizeof(io_buffer)) != 0) {
    fprintf(stderr, "Could not buffer the VCF\n");
    return 1;
  }
  uint64_t headers = 0, records = 0;
  char *line = NULL, *first = NULL;
  size_t capacity = 0;
  ssize_t length;
  // getline accepts long records while keeping only one line at a time.
  while ((length = getline(&line, &capacity, file)) >= 0) {
    if (length > 0 && line[length - 1] == '\n') {
      line[--length] = '\0';
      if (length > 0 && line[length - 1] == '\r') line[--length] = '\0';
    }
    if (line[0] == '#') {
      headers++;
    } else if (length != 0) {
      records++;
      if (!first) {
        unsigned fields = 1;
        for (char *p = line; *p; p++) {
          if (*p == '\t') {
            if (fields++ == 5) {
              *p = '\0';
              break;
            }
            *p = ' ';
          }
        }
        first = strdup(line);
        if (!first) {
          fprintf(stderr, "Could not allocate the first VCF record\n");
          return 1;
        }
      }
    }
  }
  if (!feof(file)) {
    fprintf(stderr, "Could not read VCF: %s\n", strerror(errno));
    return 1;
  }
  if (fclose(file) != 0) {
    fprintf(stderr, "Could not close VCF: %s\n", strerror(errno));
    return 1;
  }
  printf("## Variant file\n\n");
  printf("- Header lines: %" PRIu64 "\n", headers);
  printf("- Variant records: %" PRIu64 "\n", records);
  if (first) printf("- First record: `%s`\n", first);
  free(line);
  free(first);
  return 0;
}
