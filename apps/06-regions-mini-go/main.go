// One interval's reference length and GC content through the regions/ lens.
package main

import (
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
)

const region = "chrM:1-16569"
const lens = "v1/genome/regions/chrM/1-16569"

func main() {
	if len(os.Args) < 2 {
		fmt.Printf("[package]\nname = \"regions-mini\"\nversion = \"0.1.0\"\n"+
			"datasets = [\"%s\"]\n", lens)
		return
	}
	if err := run(os.Args[1]); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func run(root string) error {
	sequence, err := os.Open(filepath.Join(root, lens, "sequence"))
	if err != nil {
		return fmt.Errorf("Could not open region sequence: %w", err)
	}
	defer sequence.Close()
	var length, gc uint64
	buffer := make([]byte, 8192)
	// Forward-strand sequence has no newline; lowercase bases are soft-masked.
	for {
		n, err := sequence.Read(buffer)
		length += uint64(n)
		for _, base := range buffer[:n] {
			if base == 'G' || base == 'g' || base == 'C' || base == 'c' {
				gc++
			}
		}
		if err == io.EOF {
			break
		}
		if err != nil {
			return fmt.Errorf("Could not read region sequence: %w", err)
		}
	}
	if length != 16569 {
		return errors.New("Region sequence length does not match its span")
	}
	fmt.Printf("## Region %s\n\n", region)
	fmt.Printf("- Reference length: %d bp\n", length)
	fmt.Printf("- GC content: %.1f%%\n", 100.0*float64(gc)/float64(length))
	return nil
}
