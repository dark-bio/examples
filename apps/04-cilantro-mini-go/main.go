// Cilantro taste, from one variant.
//
// Some people taste cilantro as soap. The SNP rs72921001, near the OR6A2
// olfactory receptor, tracks the trait: the more copies of the C allele you
// carry, the more soapy cilantro tends to taste (Eriksson et al., 2012).
//
// This is ../04-cilantro-mini-rust in Go. The rsids/ folder hands
// back the genotype and coordinate as plain files; the app never opens a
// variant file. See ../../docs/04-reading-data.md.
package main

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
)

const (
	rsID        = "rs72921001"
	soapyAllele = "C"
	lens        = "v1/genome/rsids/rs72921001"
)

func main() {
	// The manifest pass names the app and asks for the one variant directory
	if len(os.Args) < 2 {
		fmt.Printf("manifest = 1\n\n"+
			"[app]\n"+
			"name = \"One Variant\"\n"+
			"version = \"0.1.0\"\n\n"+
			"[reads]\n"+
			"paths = [\"%s\"]\n\n"+
			"[listing]\n"+
			"language = \"en\"\n"+
			"icon = \"🌱\"\n"+
			"summary = \"One variant, four languages, zero call file. The smallest real read.\"\n"+
			"category = \"developer\"\n"+
			"license = \"BSD-3-Clause\"\n"+
			"source = \"https://github.com/dark-bio/examples/tree/main/apps/04-cilantro-mini-go\"\n"+
			"keywords = [\"tutorial\", \"rsids\", \"genotype\", \"four languages\"]\n"+
			"description = '''\n"+
			"The `rsids/` folder hands you one variant as plain files. Grant `v1/genome/rsids/rs72921001`, read `genotype`, `chromosome`, `position` and `reference`, and you're done. The call file never comes near it.\n\n"+
			"Every app handles three outcomes. A missing allele, as in `./.`, makes the count inconclusive. A missing genotype is no answer, never two reference alleles. Any other read error stops the app.\n\n"+
			"Written in Rust, Go, C and Python. Compare them, then meet the full report, Cilantro Taste Test.\n"+
			"'''\n\n"+
			"[listing.purposes]\n"+
			"\"v1/genome/rsids/rs72921001\" = \"The one variant this tutorial reads\"\n", lens)
		return
	}

	// The report pass reads everything from the one variant directory
	base := filepath.Join(os.Args[1], lens)

	// ENOENT means no answer, including reference sites in variants-only calls.
	genotype, present := leaf(base, "genotype")
	if !present {
		fmt.Printf("## Cilantro or soap?\n\n")
		fmt.Printf("No genotype at `%s`. Silence isn't two reference alleles.\n", rsID)
		return
	}

	// This known SNP needs only a simple split; keep the original text for display.
	copies, missing := 0, false
	for _, allele := range strings.FieldsFunc(genotype, func(r rune) bool { return r == '/' || r == '|' }) {
		if allele == "." || allele == "*" {
			missing = true
		}
		if allele == soapyAllele {
			copies++
		}
	}

	fmt.Printf("## Cilantro or soap?\n\n")
	fmt.Printf("Your genotype at `%s` is `%s`.\n\n", rsID, genotype)
	if missing {
		fmt.Println("An allele is missing or deleted, so the soap count is inconclusive.")
	} else {
		switch copies {
		case 0:
			fmt.Printf("You carry no copies of the soapy `%s` allele. Cilantro is more likely to taste fresh and herby.\n", soapyAllele)
		case 1:
			fmt.Printf("You carry one copy of the soapy `%s` allele. Cilantro may have a faint soapy note.\n", soapyAllele)
		default:
			fmt.Printf("You carry %d copies of the soapy `%s` allele. Cilantro is more likely to taste like dish soap.\n", copies, soapyAllele)
		}
	}

	chrom, hasChrom := leaf(base, "chromosome")
	pos, hasPos := leaf(base, "position")
	reference, hasRef := leaf(base, "reference")
	if hasChrom && hasPos && hasRef {
		fmt.Printf("\nLocus `%s:%s`, reference allele `%s`.\n", chrom, pos, reference)
	}
}

// Generated files hold exactly their value; only ENOENT means no answer.
func leaf(base, name string) (string, bool) {
	data, err := os.ReadFile(filepath.Join(base, name))
	if errors.Is(err, fs.ErrNotExist) {
		return "", false
	}
	if err != nil {
		fmt.Fprintf(os.Stderr, "Could not read %s %s: %v\n", rsID, name, err)
		os.Exit(1)
	}
	return string(data), true
}
