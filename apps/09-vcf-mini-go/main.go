// Count headers and records in the decompressed VCF view.
package main

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
)

const lens = "v1/genome/files/small-variants"

func main() {
	if len(os.Args) < 2 {
		fmt.Printf("manifest = 1\n\n[app]\nname = \"The Call File\"\nversion = \"0.1.0\"\n\n"+
			"[reads]\npaths = [\"%s\"]\n\n"+
			"[listing]\n"+
			"language = \"en\"\n"+
			"icon = \"📄\"\n"+
			"summary = \"Count a whole genome's records one line at a time, in 128 MiB.\"\n"+
			"category = \"developer\"\n"+
			"license = \"BSD-3-Clause\"\n"+
			"source = \"https://github.com/dark-bio/examples/tree/main/apps/09-vcf-mini-go\"\n"+
			"keywords = [\"tutorial\", \"VCF\", \"call file\", \"streaming\"]\n"+
			"description = '''\n"+
			"Grant `v1/genome/files/small-variants` and open its `vcf`, the decompressed call file. A whole genome runs to gigabytes, far past an app's 128 MiB of memory, so this app streams it line by line and counts headers and records.\n\n"+
			"Every buffer refill leaves the sandbox, and that adds up at this scale. The Rust, Go and C versions read through a 64 KiB buffer, and the Rust and C ones reuse a single line instead of allocating one per record.\n\n"+
			"This is the broadest grant there is. VCF Roll Call parses the same file with a library.\n"+
			"'''\n\n"+
			"[listing.purposes]\n"+
			"\"v1/genome/files/small-variants\" = \"Your whole small variant file, to count its records\"\n", lens)
		return
	}
	if err := run(os.Args[1]); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func run(root string) error {
	file, err := os.Open(filepath.Join(root, lens, "vcf"))
	if err != nil {
		return fmt.Errorf("Could not open VCF: %w", err)
	}
	defer file.Close()
	var headers, records uint64
	var first string
	// ReadString accepts long records while keeping only one line at a time.
	// Each refill is a call out of the sandbox, so the buffer is a large one.
	reader := bufio.NewReaderSize(file, 64*1024)
	for {
		line, err := reader.ReadString('\n')
		if err != nil && err != io.EOF {
			return fmt.Errorf("Could not read VCF: %w", err)
		}
		if strings.HasSuffix(line, "\n") {
			line = strings.TrimSuffix(strings.TrimSuffix(line, "\n"), "\r")
		}
		if strings.HasPrefix(line, "#") {
			headers++
		} else if line != "" {
			records++
			if records == 1 {
				fields := strings.SplitN(line, "\t", 6)
				if len(fields) > 5 {
					fields = fields[:5]
				}
				first = strings.Join(fields, " ")
			}
		}
		if err == io.EOF {
			break
		}
	}
	fmt.Printf("## Your whole call file, one line at a time\n\n")
	fmt.Printf("- Header lines: %d\n", headers)
	fmt.Printf("- Variant records: %d\n", records)
	if records != 0 {
		fmt.Printf("- First record: `%s`\n", first)
	}
	fmt.Printf("\nRead every line, kept none.\n")
	return nil
}
