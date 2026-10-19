// One gene's copy number through genes/.
#include <dirent.h>
#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define LENS "v1/genome/genes/UGT2B17"

// One copy number segment, named start-end after its span.
struct segment {
  char *name;
  uint64_t start;
  uint64_t end;
  // The count as the file holds it, or NULL when absent
  char *count;
};

static void fail(const char *message) {
  fprintf(stderr, "%s\n", message);
  exit(1);
}

static char *join(const char *base, const char *name) {
  size_t size = strlen(base) + strlen(name) + 2;
  char *path = malloc(size);
  if (!path) fail("Could not allocate a segment path");
  snprintf(path, size, "%s/%s", base, name);
  return path;
}

// Splits a segment name into its first and last base.
static void span(struct segment *segment) {
  const char *name = segment->name;
  errno = 0;
  char *tail;
  uintmax_t start = strtoumax(name, &tail, 10);
  if (errno == ERANGE || start > UINT64_MAX || tail == name || *tail != '-') {
    fail("Invalid segment name");
  }
  name = tail + 1;
  errno = 0;
  uintmax_t end = strtoumax(name, &tail, 10);
  if (errno == ERANGE || end > UINT64_MAX || tail == name || *tail) {
    fail("Invalid segment name");
  }
  segment->start = (uint64_t)start;
  segment->end = (uint64_t)end;
}

// Only ENOENT means an absent count. Other failures stop the app.
static char *count(const char *base, const char *name) {
  char *directory = join(base, name);
  char *path = join(directory, "count");
  free(directory);
  FILE *file = fopen(path, "rb");
  int error = errno;
  free(path);
  if (!file) {
    if (error == ENOENT) return NULL;
    fprintf(stderr, "Could not read %s count: %s\n", name, strerror(error));
    exit(1);
  }
  size_t length = 0, capacity = 128;
  char *value = malloc(capacity);
  if (!value) fail("Could not allocate a count");
  for (;;) {
    length += fread(value + length, 1, capacity - length - 1, file);
    if (ferror(file)) fail("Could not read count");
    if (feof(file)) break;
    if (capacity > SIZE_MAX / 2) fail("Count is too large");
    capacity *= 2;
    char *grown = realloc(value, capacity);
    if (!grown) fail("Could not allocate a count");
    value = grown;
  }
  if (fclose(file) != 0) fail("Could not close count");
  value[length] = '\0';
  return value;
}

// Orders segments by their first base, then their last.
static int compare(const void *left, const void *right) {
  const struct segment *a = left, *b = right;
  if (a->start != b->start) return (a->start > b->start) - (a->start < b->start);
  return (a->end > b->end) - (a->end < b->end);
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    printf("manifest = 1\n\n[app]\nname = \"Copy Number\"\nversion = \"0.1.0\"\n\n"
           "[reads]\npaths = [\"v1/genome/genes/UGT2B17\"]\n\n"
           "[listing]\nlanguage = \"en\"\nicon = \"🔢\"\n"
           "summary = \"Spot a gene that isn't there. Copy numbers as plain files, in four languages.\"\n"
           "category = \"developer\"\nlicense = \"BSD-3-Clause\"\n"
           "source = \"https://github.com/dark-bio/examples/tree/main/apps/07-copies-mini-c\"\n"
           "keywords = [\"tutorial\", \"copies\", \"copy number\", \"UGT2B17\"]\n"
           "description = '''\n"
           "Grant `v1/genome/genes/UGT2B17` and list its `copies`. Each directory is one segment of your copy number file that reaches the gene, named after its span, and its `count` holds how many copies your genome has there.\n\n"
           "Whole genes can go missing, and a small variant file never notices. An empty listing is no answer, never zero copies, since a caller can skip a region.\n\n"
           "A gene grant also covers your variants inside the gene, even though this app does not read them. Doping Test turns the same read into a report.\n"
           "'''\n\n[listing.purposes]\n"
           "\"v1/genome/genes/UGT2B17\" = \"A gene to read your copy number over\"\n");
    return 0;
  }

  // List and sort every segment first, since listing order isn't promised
  char *base = join(argv[1], LENS "/copies");
  DIR *directory = opendir(base);
  if (!directory && errno != ENOENT) {
    fprintf(stderr, "Could not list copies: %s\n", strerror(errno));
    return 1;
  }
  struct segment *segments = NULL;
  size_t length = 0, capacity = 0;
  if (directory) {
    for (;;) {
      errno = 0;
      struct dirent *entry = readdir(directory);
      if (!entry) {
        if (errno) fail("Could not read segment entry");
        break;
      }
      if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
      if (length == capacity) {
        if (capacity > SIZE_MAX / sizeof(*segments) / 2) fail("Too many segments");
        capacity = capacity ? capacity * 2 : 16;
        struct segment *grown = realloc(segments, capacity * sizeof(*segments));
        if (!grown) fail("Could not allocate segments");
        segments = grown;
      }
      struct segment *segment = &segments[length++];
      segment->name = strdup(entry->d_name);
      if (!segment->name) fail("Could not allocate a segment name");
      span(segment);
    }
    if (closedir(directory) != 0) fail("Could not close copies listing");
  }
  if (length > 1) qsort(segments, length, sizeof(*segments), compare);
  for (size_t i = 0; i < length; i++) segments[i].count = count(base, segments[i].name);

  printf("## Copies of UGT2B17\n\n");
  if (!length) printf("No copy number segment reaches UGT2B17. That is no answer, not zero copies.\n");
  for (size_t i = 0; i < length; i++) {
    struct segment *segment = &segments[i];
    if (!segment->count) printf("- Segment %s: no answer\n", segment->name);
    else printf("- Segment %s: %s %s\n", segment->name, segment->count,
                !strcmp(segment->count, "1") ? "copy" : "copies");
    free(segment->name);
    free(segment->count);
  }
  free(segments);
  free(base);
  return 0;
}
