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

const lens = "v1/genome/snp-indel"

func main() {
	if len(os.Args) < 2 {
		fmt.Printf("[package]\nname = \"vcf-mini\"\nversion = \"0.1.0\"\n"+
			"datasets = [\"%s\"]\n", lens)
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
	fmt.Printf("## Variant file\n\n")
	fmt.Printf("- Header lines: %d\n", headers)
	fmt.Printf("- Variant records: %d\n", records)
	if records != 0 {
		fmt.Printf("- First record: `%s`\n", first)
	}
	return nil
}
