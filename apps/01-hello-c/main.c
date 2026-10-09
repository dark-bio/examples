// The smallest possible Ark app, in C.
//
// With no arguments it prints its manifest. With a data directory as the first
// argument it does its real work, which here is to print a greeting. See
// ../../docs/01-app-model.md for the two passes.
#include <stdio.h>

int main(int argc, char *argv[]) {
  (void)argv;
  // The manifest pass has no data directory, so describe the app and stop.
  // This app reads nothing, so it has no reads table.
  if (argc < 2) {
    printf(
      "manifest = 1\n\n"
      "[app]\n"
      "name = \"Hello, Ark\"\n"
      "version = \"0.1.0\"\n\n"
      "[listing]\n"
      "language = \"en\"\n"
      "icon = \"👋\"\n"
      "summary = \"The two passes of an app, in the fewest lines. Start here.\"\n"
      "category = \"developer\"\n"
      "license = \"BSD-3-Clause\"\n"
      "source = \"https://github.com/dark-bio/examples/tree/main/apps/01-hello-c\"\n"
      "keywords = [\"tutorial\", \"manifest\", \"two passes\", \"WASI\"]\n"
      "description = '''\n"
      "An Ark app is one WebAssembly file built for WASI Preview 1. The Ark runs it twice.\n\n"
      "**The manifest pass.** With no arguments, the app prints a short TOML manifest naming itself, its version and the data it wants. The Ark checks the manifest and shows it on your phone.\n\n"
      "**The report pass.** With a data directory as its first argument, the app reads the files it was granted and prints its report. Reports are Markdown, which is how Ark Hub shows them.\n\n"
      "This app asks for no data, so approving it grants nothing. It proves the round trip, from upload through approval to a rendered report, in the fewest lines possible. Pick a language, read the source, then run it on your Ark.\n\n"
      "Each language's folder in the examples repository builds and runs it with one `make run` command.\n\n"
      "The docs cover the two passes, the manifest and the sandbox limits in detail.\n"
      "'''\n");
    return 0;
  }

  // The report pass receives a data directory. This app reads nothing.
  printf("Hello from an Ark app written in C.\n");
  return 0;
}
