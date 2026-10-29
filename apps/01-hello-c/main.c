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
      "summary = \"Two passes, a handful of lines, zero data. Start here.\"\n"
      "category = \"developer\"\n"
      "license = \"BSD-3-Clause\"\n"
      "source = \"https://github.com/dark-bio/examples/tree/main/apps/01-hello-c\"\n"
      "keywords = [\"tutorial\", \"manifest\", \"two passes\", \"WASI\"]\n"
      "description = '''\n"
      "An Ark app is one WebAssembly file built for WASI Preview 1, and the Ark runs it twice.\n\n"
      "**The manifest pass.** No arguments. The app prints a short TOML manifest with its name, its version and the data it wants, and your phone shows it to you.\n\n"
      "**The processing pass.** A data directory as its first argument. The app reads\n"
      "what you granted and prints a Markdown report, which Ark Hub renders.\n\n"
      "This one asks for nothing, so approving it shares nothing. It's the whole round trip, upload to approval to report, in a handful of lines.\n\n"
      "Each language's folder in the examples repository builds and runs it with one `make run`.\n\n"
      "Pick a language, read the source, then run it on your Ark.\n"
      "'''\n");
    return 0;
  }

  // The processing pass receives a data directory even without grants
  printf("Hello from inside an Ark. No network, no disk, no way out. C says hi.\n");
  return 0;
}
