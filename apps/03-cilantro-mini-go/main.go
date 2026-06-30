// Cilantro taste, from one variant.
//
// Some people taste cilantro as soap. The SNP rs72921001, near the OR6A2
// olfactory receptor, tracks the trait: the more copies of the C allele you
// carry, the more soapy cilantro tends to taste (Eriksson et al., 2012).
//
// It is the same program as ../03-cilantro-mini-rust, in Go. The rsids/ lens hands
// back the genotype and coordinate as plain files; the app never opens a
// variant file. See ../../docs/04-data-access.md.
package main

import (
	"fmt"
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

	// The genotype leaf exists only where your calls cover the site. A missing
	// file just means the site was not sequenced, so report that and stop.
	raw, err := os.ReadFile(filepath.Join(base, "genotype"))
	if err != nil {
		fmt.Printf("## Cilantro taste\n\n")
		fmt.Printf("Your data does not cover `%s`, so there is nothing to report.\n", rsID)
		return
	}
	genotype := strings.TrimSpace(string(raw))

	// The lens returns alleles as bases (A/C, or C|C when phased), so just count
	// copies of the soapy allele. There is no REF/ALT index to decode.
	copies := 0
	for _, allele := range strings.FieldsFunc(genotype, func(r rune) bool { return r == '/' || r == '|' }) {
		if allele == soapyAllele {
			copies++
		}
	}

	fmt.Printf("## Cilantro taste\n\n")
	fmt.Printf("Your genotype at `%s` is `%s`.\n\n", rsID, genotype)
	switch copies {
	case 0:
		fmt.Printf("You carry no copies of the soapy `%s` allele. Cilantro probably tastes fresh and herby.\n", soapyAllele)
	case 1:
		fmt.Printf("You carry one copy of the soapy `%s` allele. Cilantro may have a faint soapy note.\n", soapyAllele)
	default:
		fmt.Printf("You carry two copies of the soapy `%s` allele. Cilantro likely tastes like dish soap.\n", soapyAllele)
	}

	// The coordinate, read from the lens's other leaves, shown for context.
	chrom, pos, reference := leaf(base, "chromosome"), leaf(base, "position"), leaf(base, "reference")
	if chrom != "" && pos != "" {
		fmt.Printf("\nLocus `%s:%s`, reference allele `%s`.\n", chrom, pos, reference)
	}
}

// leaf reads a single lens file and trims it, returning "" if it is absent.
func leaf(base, name string) string {
	data, err := os.ReadFile(filepath.Join(base, name))
	if err != nil {
		return ""
	}
	return strings.TrimSpace(string(data))
}
