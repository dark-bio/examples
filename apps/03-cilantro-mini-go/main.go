// Cilantro taste, from one variant.
//
// Some people taste cilantro as soap. The SNP rs72921001, near the OR6A2
// olfactory receptor, tracks the trait: the more copies of the C allele you
// carry, the more soapy cilantro tends to taste (Eriksson et al., 2012).
//
// It is the same program as ../03-cilantro-mini-rust, in Go. The rsids/ lens hands
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
	// The manifest pass: name the app and ask for the one variant directory.
	if len(os.Args) < 2 {
		fmt.Printf("[package]\n"+
			"name = \"cilantro-mini\"\n"+
			"version = \"0.1.0\"\n"+
			"datasets = [\"%s\"]\n", lens)
		return
	}

	// The run pass: everything comes from the one lens directory.
	base := filepath.Join(os.Args[1], lens)

	// ENOENT means no answer, including reference sites in variants-only calls.
	genotype, present := leaf(base, "genotype")
	if !present {
		fmt.Printf("## Cilantro taste\n\n")
		fmt.Printf("No genotype answer for `%s`. Absence does not imply two reference alleles.\n", rsID)
		return
	}

	// This known SNP needs only a simple split; keep the original text for display.
	copies, missing := 0, false
	for _, allele := range strings.FieldsFunc(genotype, func(r rune) bool { return r == '/' || r == '|' }) {
		if allele == "." {
			missing = true
		}
		if allele == soapyAllele {
			copies++
		}
	}

	fmt.Printf("## Cilantro taste\n\n")
	fmt.Printf("Your genotype at `%s` is `%s`.\n\n", rsID, genotype)
	if missing {
		fmt.Println("The copy count is inconclusive because an allele is missing (`.`).")
	} else {
		switch copies {
		case 0:
			fmt.Printf("You carry no copies of the soapy `%s` allele. Cilantro probably tastes fresh and herby.\n", soapyAllele)
		case 1:
			fmt.Printf("You carry one copy of the soapy `%s` allele. Cilantro may have a faint soapy note.\n", soapyAllele)
		default:
			fmt.Printf("You carry %d copies of the soapy `%s` allele. Cilantro likely tastes like dish soap.\n", copies, soapyAllele)
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
