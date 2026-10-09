// One gene's metadata and reference length through the genes/ lens.
package main

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strconv"
)

const gene = "TAS2R38"
const lens = "v1/genome/genes/TAS2R38"

func main() {
	if len(os.Args) < 2 {
		fmt.Printf("manifest = 1\n\n[app]\nname = \"One Gene\"\nversion = \"0.1.0\"\n\n"+
			"[reads]\npaths = [\"%s\"]\n\n"+
			"[listing]\n"+
			"language = \"en\"\n"+
			"icon = \"🧬\"\n"+
			"summary = \"A gene's coordinates and sequence length, through the genes lens.\"\n"+
			"category = \"developer\"\n"+
			"license = \"BSD-3-Clause\"\n"+
			"source = \"https://github.com/dark-bio/examples/tree/main/apps/06-genes-mini-go\"\n"+
			"keywords = [\"tutorial\", \"genes\", \"sequence\", \"TAS2R38\"]\n"+
			"description = '''\n"+
			"Grant `v1/genome/genes/TAS2R38` and read the gene's `chromosome`, `start`, `end`, `strand` and `biotype` as files. The `sequence` file holds exactly end - start + 1 bases, so its size is the gene's length and the gene never has to fit in memory.\n\n"+
			"An absent value prints as no answer. A missing sequence is an error.\n\n"+
			"A gene grant also covers your variants inside the gene, even though this app does not read them. Bitter Meter does.\n"+
			"'''\n\n"+
			"[listing.purposes]\n"+
			"\"v1/genome/genes/TAS2R38\" = \"A gene to read coordinates and sequence length from\"\n", lens)
		return
	}
	if err := run(filepath.Join(os.Args[1], lens)); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func run(base string) error {
	// File size gives the sequence length in bp without loading the gene.
	info, err := os.Stat(filepath.Join(base, "sequence"))
	if err != nil {
		return fmt.Errorf("Could not read %s sequence: %w", gene, err)
	}
	length := info.Size()
	if length == 0 {
		return errors.New("The gene sequence is empty")
	}
	names := []string{"chromosome", "start", "end", "strand", "biotype"}
	values := make([]string, len(names))
	present := make([]bool, len(names))
	var start, end uint64
	for i, name := range names {
		data, err := os.ReadFile(filepath.Join(base, name))
		// Only an absent scalar means no answer; every other failure stops the app.
		if errors.Is(err, fs.ErrNotExist) {
			values[i] = "no answer"
			continue
		}
		if err != nil {
			return fmt.Errorf("Could not read %s %s: %w", gene, name, err)
		}
		present[i], values[i] = true, string(data)
		if name == "start" || name == "end" {
			n, err := strconv.ParseUint(values[i], 10, 64)
			if err != nil {
				return fmt.Errorf("Invalid gene %s: %w", name, err)
			}
			values[i] = strconv.FormatUint(n, 10)
			if name == "start" {
				start = n
			} else {
				end = n
			}
		}
	}
	if present[1] && present[2] &&
		(start == 0 || end < start || uint64(length) != end-start+1) {
		return errors.New("Gene sequence length does not match its span")
	}
	fmt.Printf("## Gene %s\n\n", gene)
	for i, label := range []string{"Chromosome", "Start", "End", "Strand", "Biotype"} {
		fmt.Printf("- %s: %s\n", label, values[i])
	}
	fmt.Printf("- Reference length: %d bp\n", length)
	return nil
}
