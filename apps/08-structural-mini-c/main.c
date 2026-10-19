// One structural variant through regions/.
#include <dirent.h>
#include <errno.h>
#include <inttypes.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#define LENS "v1/genome/regions/chr17/63488479-63488579"

// One change entry, with the leaves the app prints.
struct change {
  char *name;
  uint64_t start, end, suffix;
  char *kind, *dataset, *carried;
  uint64_t new_bases;
  int has_bases;
};

static void fail(const char *message) {
  fprintf(stderr, "%s\n", message);
  exit(1);
}

static char *join(const char *base, const char *name) {
  size_t size = strlen(base) + strlen(name) + 2;
  char *path = malloc(size);
  if (!path) fail("Could not allocate a change path");
  snprintf(path, size, "%s/%s", base, name);
  return path;
}

// Splits an entry name such as 100-100 or 100-100-2 into start, end and suffix.
static void span(struct change *change) {
  const char *name = change->name;
  char *tail;
  errno = 0;
  uintmax_t start = strtoumax(name, &tail, 10);
  if (errno == ERANGE || start > UINT64_MAX || tail == name || *tail != '-') {
    fail("Invalid change start");
  }
  name = tail + 1;
  errno = 0;
  uintmax_t end = strtoumax(name, &tail, 10);
  if (errno == ERANGE || end > UINT64_MAX || tail == name || (*tail && *tail != '-')) {
    fail("Invalid change end");
  }
  uintmax_t suffix = 1;
  if (*tail == '-') {
    name = tail + 1;
    errno = 0;
    suffix = strtoumax(name, &tail, 10);
    if (errno == ERANGE || suffix > UINT64_MAX || tail == name || *tail) {
      fail("Invalid change suffix");
    }
  }
  change->start = (uint64_t)start;
  change->end = (uint64_t)end;
  change->suffix = (uint64_t)suffix;
}

// Only ENOENT means an absent scalar. Other failures stop the app.
static char *leaf(const char *base, const char *name) {
  char *path = join(base, name);
  FILE *file = fopen(path, "rb");
  int error = errno;
  free(path);
  if (!file) {
    if (error == ENOENT) return NULL;
    fprintf(stderr, "Could not read %s: %s\n", name, strerror(error));
    exit(1);
  }
  size_t length = 0, capacity = 128;
  char *value = malloc(capacity);
  if (!value) fail("Could not allocate a change value");
  for (;;) {
    length += fread(value + length, 1, capacity - length - 1, file);
    if (ferror(file)) fail("Could not read change value");
    if (feof(file)) break;
    if (capacity > SIZE_MAX / 2) fail("Change value is too large");
    capacity *= 2;
    char *grown = realloc(value, capacity);
    if (!grown) fail("Could not allocate a change value");
    value = grown;
  }
  if (fclose(file) != 0) fail("Could not close change value");
  value[length] = '\0';
  return value;
}

// Counts the new bases from the size of bases, or from length for a symbolic
// insertion. A long allele is measured, never read whole.
static void new_bases(const char *base, struct change *change) {
  char *path = join(base, "bases");
  struct stat info;
  int result = stat(path, &info);
  int error = errno;
  free(path);
  if (result == 0) {
    change->new_bases = (uint64_t)info.st_size;
    change->has_bases = 1;
    return;
  }
  if (error != ENOENT) {
    fprintf(stderr, "Could not read bases size: %s\n", strerror(error));
    exit(1);
  }
  char *length = leaf(base, "length");
  change->has_bases = length != NULL;
  if (length) {
    errno = 0;
    char *tail;
    uintmax_t size = strtoumax(length, &tail, 10);
    if (errno == ERANGE || size > UINT64_MAX || tail == length || *tail) {
      fail("Invalid length");
    }
    change->new_bases = (uint64_t)size;
    free(length);
  }
}

// Orders changes by start, then end, then suffix.
static int compare(const void *left, const void *right) {
  const struct change *a = left, *b = right;
  if (a->start != b->start) return (a->start > b->start) - (a->start < b->start);
  if (a->end != b->end) return (a->end > b->end) - (a->end < b->end);
  return (a->suffix > b->suffix) - (a->suffix < b->suffix);
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    printf("manifest = 1\n\n"
           "[app]\n"
           "name = \"Structural Variant\"\n"
           "version = \"0.1.0\"\n\n"
           "[reads]\n"
           "paths = [\"v1/genome/regions/chr17/63488479-63488579\"]\n\n"
           "[listing]\n"
           "language = \"en\"\n"
           "icon = \"🧩\"\n"
           "summary = \"A structural variant read as plain files, in four languages.\"\n"
           "category = \"developer\"\n"
           "license = \"BSD-3-Clause\"\n"
           "source = \"https://github.com/dark-bio/examples/tree/main/apps/08-structural-mini-c\"\n"
           "keywords = [\"tutorial\", \"changes\", \"structural variants\", \"ACE\"]\n"
           "description = '''\n"
           "Grant a short window inside the _ACE_ gene and list its `changes`. Each directory is one alternate allele you carry, named after the span it changes. Its `type`, `bases`, `dataset` and `carried` say what changed, into what, in which file and on how many copies.\n\n"
           "The window holds the ACE insertion, nearly 300 bases long and too long for a small variant caller, so it comes from your structural variant file. The window spans 101 bases, since callers can place an insertion a few bases apart.\n\n"
           "An interval grant also covers your small variants inside it. Sprint or Marathon turns the same read into a report.\n"
           "'''\n\n"
           "[listing.purposes]\n"
           "\"v1/genome/regions/chr17/63488479-63488579\" = \"101 bases of the ACE gene, around its Alu insertion\"\n");
    return 0;
  }

  // List and sort every change first, since listing order isn't promised
  char *base = join(argv[1], LENS "/changes");
  DIR *directory = opendir(base);
  if (!directory && errno != ENOENT) {
    fprintf(stderr, "Could not list changes: %s\n", strerror(errno));
    return 1;
  }
  struct change *changes = NULL;
  size_t length = 0, capacity = 0;
  if (directory) {
    for (;;) {
      errno = 0;
      struct dirent *entry = readdir(directory);
      if (!entry) {
        if (errno) fail("Could not read change entry");
        break;
      }
      if (!strcmp(entry->d_name, ".") || !strcmp(entry->d_name, "..")) continue;
      if (length == capacity) {
        if (capacity > SIZE_MAX / sizeof(*changes) / 2) fail("Too many changes");
        capacity = capacity ? capacity * 2 : 16;
        struct change *grown = realloc(changes, capacity * sizeof(*changes));
        if (!grown) fail("Could not allocate changes");
        changes = grown;
      }
      struct change *change = &changes[length++];
      change->name = strdup(entry->d_name);
      if (!change->name) fail("Could not allocate a change name");
      span(change);
    }
    if (closedir(directory) != 0) fail("Could not close changes listing");
  }
  if (length > 1) qsort(changes, length, sizeof(*changes), compare);
  for (size_t i = 0; i < length; i++) {
    char *entry = join(base, changes[i].name);
    changes[i].kind = leaf(entry, "type");
    changes[i].dataset = leaf(entry, "dataset");
    changes[i].carried = leaf(entry, "carried");
    new_bases(entry, &changes[i]);
    free(entry);
  }

  printf("## Changes near the ACE insertion\n\n");
  if (!length) printf("No change reaches this window. That is no answer, since your files may not cover it.\n");
  for (size_t i = 0; i < length; i++) {
    struct change *change = &changes[i];
    printf("- %s: %s, ", change->name, change->kind ? change->kind : "no answer");
    if (change->has_bases) {
      printf("%" PRIu64 " new %s", change->new_bases, change->new_bases == 1 ? "base" : "bases");
    } else printf("no new bases");
    printf(", %s, carried on ", change->dataset ? change->dataset : "no answer");
    if (change->carried) {
      printf("%s %s\n", change->carried, !strcmp(change->carried, "1") ? "copy" : "copies");
    } else printf("no answer\n");
    free(change->name);
    free(change->kind);
    free(change->dataset);
    free(change->carried);
  }
  free(changes);
  free(base);
  return 0;
}
