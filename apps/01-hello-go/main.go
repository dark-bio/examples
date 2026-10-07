// The smallest possible Ark app, in Go.
//
// With no arguments it prints its manifest. With a data directory as the first
// argument it does its real work, which here is to print a greeting. See
// ../../docs/01-app-model.md for the two passes.
package main

import (
	"fmt"
	"os"
)

func main() {
	// The manifest pass: no data directory was given, so describe the app and
	// stop. This app reads nothing, so it has no reads table.
	if len(os.Args) < 2 {
		fmt.Print(
			"manifest = 1\n\n" +
				"[app]\n" +
				"name = \"hello-go\"\n" +
				"version = \"0.1.0\"\n")
		return
	}

	// The run pass: a data directory was given. A real app would read it here.
	fmt.Println("Hello from an Ark app written in Go.")
}
