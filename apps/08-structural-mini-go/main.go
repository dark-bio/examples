// One structural variant through regions/.
package main

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
)

const lens = "v1/genome/regions/chr17/63488479-63488579"

// change is one change entry, with the leaves the app prints.
type change struct {
	name                   string
	start, end, suffix     uint64
	kind, dataset, carried *string
	newBases               *uint64
}

func main() {
	if len(os.Args) < 2 {
		fmt.Print("manifest = 1\n\n" +
			"[app]\n" +
			"name = \"Structural Variant\"\n" +
			"version = \"0.1.0\"\n\n" +
			"[reads]\n" +
			"paths = [\"v1/genome/regions/chr17/63488479-63488579\"]\n\n" +
			"[listing]\n" +
			"language = \"en\"\n" +
			"icon = \"🧩\"\n" +
			"summary = \"Nearly 300 bases of Alu, read as plain files, in four languages.\"\n" +
			"category = \"developer\"\n" +
			"license = \"BSD-3-Clause\"\n" +
			"source = \"https://github.com/dark-bio/examples/tree/main/apps/08-structural-mini-go\"\n" +
			"keywords = [\"tutorial\", \"changes\", \"structural variants\", \"ACE\"]\n" +
			"description = '''\n" +
			"Grant a short window inside the _ACE_ gene and list its `changes`. Each directory is one alternate allele you carry, named after the span it changes. Its `type`, `bases`, `dataset` and `carried` say what changed, into what, in which file and on how many copies.\n\n" +
			"The window holds the ACE insertion, nearly 300 bases long and too long for a small variant caller, so it comes from your structural variant file. The window spans 101 bases, since callers can place an insertion a few bases apart.\n\n" +
			"An interval grant also covers your small variants inside it. Sprint or Marathon turns the same read into a report.\n" +
			"'''\n\n" +
			"[listing.purposes]\n" +
			"\"v1/genome/regions/chr17/63488479-63488579\" = \"101 bases of the ACE gene, around its Alu insertion\"\n")
		return
	}
	if err := run(os.Args[1]); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

// leaf reads one scalar, and only ENOENT means no answer.
func leaf(base, name string) (*string, error) {
	value, err := os.ReadFile(filepath.Join(base, name))
	if errors.Is(err, fs.ErrNotExist) {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("Could not read %s: %w", name, err)
	}
	text := string(value)
	return &text, nil
}

// newBases counts the new bases from the size of bases, or from length for a
// symbolic insertion. A long allele is measured, never read whole.
func newBases(base string) (*uint64, error) {
	info, err := os.Stat(filepath.Join(base, "bases"))
	if err == nil {
		size := uint64(info.Size())
		return &size, nil
	}
	if !errors.Is(err, fs.ErrNotExist) {
		return nil, fmt.Errorf("Could not read bases size: %w", err)
	}
	value, err := leaf(base, "length")
	if err != nil || value == nil {
		return nil, err
	}
	size, err := strconv.ParseUint(*value, 10, 64)
	if err != nil {
		return nil, fmt.Errorf("Invalid length: %w", err)
	}
	return &size, nil
}

// run prints every change in the window, reading an empty listing as no answer.
func run(root string) error {
	// A missing listing has no answer, just like an empty one
	base := filepath.Join(root, lens, "changes")
	entries, err := os.ReadDir(base)
	if err != nil && !errors.Is(err, fs.ErrNotExist) {
		return fmt.Errorf("Could not list changes: %w", err)
	}
	var changes []change
	for _, entry := range entries {
		name := entry.Name()
		first, rest, ok := strings.Cut(name, "-")
		if !ok {
			return fmt.Errorf("Invalid change name: %s", name)
		}
		last, suffix, suffixed := strings.Cut(rest, "-")
		if !suffixed {
			suffix = "1"
		}
		start, startErr := strconv.ParseUint(first, 10, 64)
		end, endErr := strconv.ParseUint(last, 10, 64)
		ordinal, suffixErr := strconv.ParseUint(suffix, 10, 64)
		if startErr != nil || endErr != nil || suffixErr != nil {
			return fmt.Errorf("Invalid change name: %s", name)
		}
		directory := filepath.Join(base, name)
		kind, err := leaf(directory, "type")
		if err != nil {
			return err
		}
		dataset, err := leaf(directory, "dataset")
		if err != nil {
			return err
		}
		carried, err := leaf(directory, "carried")
		if err != nil {
			return err
		}
		bases, err := newBases(directory)
		if err != nil {
			return err
		}
		changes = append(changes, change{name, start, end, ordinal, kind, dataset, carried, bases})
	}
	// os.ReadDir sorts names as text, so sort again by span
	sort.Slice(changes, func(i, j int) bool {
		if changes[i].start != changes[j].start {
			return changes[i].start < changes[j].start
		}
		if changes[i].end != changes[j].end {
			return changes[i].end < changes[j].end
		}
		return changes[i].suffix < changes[j].suffix
	})

	fmt.Print("## Changes near the ACE insertion\n\n")
	if len(changes) == 0 {
		fmt.Println("No change reaches this window. That is no answer, since your files may not cover it.")
	}
	for _, change := range changes {
		kind, dataset, copies, bases := "no answer", "no answer", "no answer", "no new bases"
		if change.kind != nil {
			kind = *change.kind
		}
		if change.dataset != nil {
			dataset = *change.dataset
		}
		if change.carried != nil {
			copies = *change.carried + " copies"
			if *change.carried == "1" {
				copies = "1 copy"
			}
		}
		if change.newBases != nil {
			bases = fmt.Sprintf("%d new bases", *change.newBases)
			if *change.newBases == 1 {
				bases = "1 new base"
			}
		}
		fmt.Printf("- %s: %s, %s, %s, carried on %s\n", change.name, kind, bases, dataset, copies)
	}
	return nil
}
