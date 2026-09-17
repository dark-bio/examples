// One gene's metadata and reference length through the genes/ lens.
#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#define GENE "TAS2R38"
#define LENS "v1/genome/genes/TAS2R38"

static void fail(const char *message) {
  fprintf(stderr, "%s\n", message);
  exit(1);
}

static char *path(const char *root, const char *name) {
  size_t size = strlen(root) + sizeof(LENS) + strlen(name) + 3;
  char *value = malloc(size);
  if (!value) fail("Could not allocate a gene path");
  snprintf(value, size, "%s/" LENS "/%s", root, name);
  return value;
}

// Only ENOENT means an absent scalar; other failures stop the app.
static char *leaf(const char *root, const char *name) {
  char *filename = path(root, name);
  FILE *file = fopen(filename, "rb");
  int error = errno;
  free(filename);
  if (!file) {
    if (error == ENOENT) return NULL;
    fprintf(stderr, "Could not read " GENE " %s: %s\n", name, strerror(error));
    exit(1);
  }
  size_t length = 0, capacity = 128;
  char *value = malloc(capacity);
  if (!value) fail("Could not allocate a gene value");
  for (;;) {
    length += fread(value + length, 1, capacity - length - 1, file);
    if (ferror(file)) fail("Could not read gene value");
    if (feof(file)) break;
    if (capacity > SIZE_MAX / 2) fail("Gene value is too large");
    capacity *= 2;
    char *grown = realloc(value, capacity);
    if (!grown) fail("Could not allocate a gene value");
    value = grown;
  }
  if (fclose(file) != 0) fail("Could not close gene value");
  value[length] = '\0';
  return value;
}

static uint64_t coordinate(const char *value) {
  const char *digits = value + (value[0] == '+');
  if (!*digits || strspn(digits, "0123456789") != strlen(digits)) {
    fail("Invalid gene coordinate");
  }
  errno = 0;
  uintmax_t number = strtoumax(value, NULL, 10);
  if (errno == ERANGE || number > UINT64_MAX) fail("Gene coordinate is too large");
  return (uint64_t)number;
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    printf("[package]\nname = \"genes-mini\"\nversion = \"0.1.0\"\n"
           "datasets = [\"" LENS "\"]\n");
    return 0;
  }

  // File size gives the sequence length in bp without loading the gene.
  char *filename = path(argv[1], "sequence");
  struct stat info;
  if (stat(filename, &info) != 0) {
    fprintf(stderr, "Could not read " GENE " sequence: %s\n", strerror(errno));
    return 1;
  }
  free(filename);
  uint64_t length = (uint64_t)info.st_size;
  if (length == 0) fail("The gene sequence is empty");
  char *chromosome = leaf(argv[1], "chromosome");
  char *start_text = leaf(argv[1], "start");
  char *end_text = leaf(argv[1], "end");
  char *strand = leaf(argv[1], "strand");
  char *biotype = leaf(argv[1], "biotype");
  uint64_t start = start_text ? coordinate(start_text) : 0;
  uint64_t end = end_text ? coordinate(end_text) : 0;
  if (start_text && end_text &&
      (start == 0 || end < start || length != end - start + 1)) {
    fail("Gene sequence length does not match its span");
  }

  printf("## Gene " GENE "\n\n");
  printf("- Chromosome: %s\n", chromosome ? chromosome : "no answer");
  if (start_text) printf("- Start: %" PRIu64 "\n", start);
  else printf("- Start: no answer\n");
  if (end_text) printf("- End: %" PRIu64 "\n", end);
  else printf("- End: no answer\n");
  printf("- Strand: %s\n", strand ? strand : "no answer");
  printf("- Biotype: %s\n", biotype ? biotype : "no answer");
  printf("- Reference length: %" PRIu64 " bp\n", length);
  free(chromosome);
  free(start_text);
  free(end_text);
  free(strand);
  free(biotype);
  return 0;
}
