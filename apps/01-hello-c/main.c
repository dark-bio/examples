// The smallest possible Ark app, in C.
//
// With no arguments it prints its manifest. With a data directory as the first
// argument it does its real work, which here is to print a greeting. See
// ../../docs/01-app-model.md for the two passes.
#include <stdio.h>

int main(int argc, char *argv[]) {
  (void)argv;
  // The manifest pass: no data directory was given, so describe the app and
  // stop. This app reads nothing, so its dataset list is empty.
  if (argc < 2) {
    printf(
      "[package]\n"
      "name = \"hello-c\"\n"
      "version = \"0.1.0\"\n"
      "datasets = []\n");
    return 0;
  }

  // The run pass: a data directory was given. A real app would read it here.
  printf("Hello from an Ark app written in C.\n");
  return 0;
}
