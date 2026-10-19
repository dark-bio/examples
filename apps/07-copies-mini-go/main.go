// One gene's copy number through genes/.
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

const lens = "v1/genome/genes/UGT2B17"

// segment is one copy number segment, named start-end after its span.
type segment struct {
	name  string
	start uint64
	end   uint64
	// count is the copy number as the file holds it
	count string
	// present is false when the file holds no count
	present bool
}

func main() {
	if len(os.Args) < 2 {
		fmt.Print("manifest = 1\n\n" +
			"[app]\n" +
			"name = \"Copy Number\"\n" +
			"version = \"0.1.0\"\n\n" +
			"[reads]\n" +
			"paths = [\"v1/genome/genes/UGT2B17\"]\n\n" +
			"[listing]\n" +
			"language = \"en\"\n" +
			"icon = \"🔢\"\n" +
			"summary = \"Spot a gene that isn't there. Copy numbers as plain files, in four languages.\"\n" +
			"category = \"developer\"\n" +
			"license = \"BSD-3-Clause\"\n" +
			"source = \"https://github.com/dark-bio/examples/tree/main/apps/07-copies-mini-go\"\n" +
			"keywords = [\"tutorial\", \"copies\", \"copy number\", \"UGT2B17\"]\n" +
			"description = '''\n" +
			"Grant `v1/genome/genes/UGT2B17` and list its `copies`. Each directory is one segment of your copy number file that reaches the gene, named after its span, and its `count` holds how many copies your genome has there.\n\n" +
			"Whole genes can go missing, and a small variant file never notices. An empty listing is no answer, never zero copies, since a caller can skip a region.\n\n" +
			"A gene grant also covers your variants inside the gene, even though this app does not read them. Doping Test turns the same read into a report.\n" +
			"'''\n\n" +
			"[listing.purposes]\n" +
			"\"v1/genome/genes/UGT2B17\" = \"A gene to read your copy number over\"\n")
		return
	}
	if err := run(os.Args[1]); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

// run prints each segment's count, reading an empty listing as no answer.
func run(root string) error {
	// A missing listing has no answer, just like an empty one
	base := filepath.Join(root, lens, "copies")
	entries, err := os.ReadDir(base)
	if err != nil && !errors.Is(err, fs.ErrNotExist) {
		return fmt.Errorf("Could not list copies: %w", err)
	}
	var segments []segment
	for _, entry := range entries {
		name := entry.Name()
		first, last, ok := strings.Cut(name, "-")
		start, startErr := strconv.ParseUint(first, 10, 64)
		end, endErr := strconv.ParseUint(last, 10, 64)
		if !ok || startErr != nil || endErr != nil {
			return fmt.Errorf("Invalid segment name: %s", name)
		}
		// Only a missing count means no answer, and other failures stop the app
		count, err := os.ReadFile(filepath.Join(base, name, "count"))
		if err != nil && !errors.Is(err, fs.ErrNotExist) {
			return fmt.Errorf("Could not read %s count: %w", name, err)
		}
		segments = append(segments, segment{name, start, end, string(count), err == nil})
	}
	// os.ReadDir sorts names as text, so sort again by the first base
	sort.Slice(segments, func(i, j int) bool {
		if segments[i].start == segments[j].start {
			return segments[i].end < segments[j].end
		}
		return segments[i].start < segments[j].start
	})

	fmt.Print("## Copies of UGT2B17\n\n")
	if len(segments) == 0 {
		fmt.Println("No copy number segment reaches UGT2B17. That is no answer, not zero copies.")
	}
	for _, segment := range segments {
		switch {
		case !segment.present:
			fmt.Printf("- Segment %s: no answer\n", segment.name)
		case segment.count == "1":
			fmt.Printf("- Segment %s: 1 copy\n", segment.name)
		default:
			fmt.Printf("- Segment %s: %s copies\n", segment.name, segment.count)
		}
	}
	return nil
}
