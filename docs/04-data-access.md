# Data access patterns

An app should ask for the narrowest data that does the job. The owner sees the
request at approval, and "one variant" reads very differently from "the whole
variant file." This page walks the patterns from narrowest to broadest, each one
matched to an example you can run.

## Prefer a lens to the raw file

Almost every app wants a lens, not a slot. A lens hands back a plain value
computed for the exact site you asked about: a genotype, a coordinate, a
sequence. You declare one small path, the owner sees one small request, and your
code reads a string instead of parsing a genomic file format. Lens values are
also build-independent and already strand- and allele-resolved, so there is no
REF/ALT index to decode.

Reach for the raw slot only when you genuinely need the whole file, and expect
the broader request to show up at approval.

## The ladder

### One variant - the `rsids/` lens

The narrowest useful read. Declare a single `rsids/<rsid>/` directory and read
its `genotype` leaf. No parsing, no coordinates to manage.

```toml
datasets = ["v1/genome/rsids/rs72921001"]
```

See [cilantro-mini](../apps/03-cilantro-mini-rust) for the bare read and
[cilantro-soapiness](../apps/03-cilantro-soapiness) for the same read as a full
report.

### A panel of variants - the `rsids/` lens, repeated

A trait that depends on several SNPs declares one `rsids/` directory per variant
and reads each genotype. It is the single-variant pattern at scale; the owner
sees the exact list of sites.

See [drunk-o-type](../apps/04-drunk-o-type), which reads eleven.

### A whole gene - the `genes/` lens

Declare `genes/<symbol>/` to get the gene's metadata leaves (chromosome, start,
end, strand, biotype), its `reference` sequence, and a `changes/` directory of
the non-reference variants you carry within it.

See [genes-mini](../apps/05-genes-mini) and [bitter-meter](../apps/05-bitter-meter).

### An interval - the `regions/` lens

For an arbitrary stretch of a chromosome, declare `regions/<chr>/<start>-<end>/`.
It works like the gene lens: a `reference` sequence for the interval and a
`changes/` directory within it. Useful when you care about a coordinate range
rather than a named feature.

See [regions-mini](../apps/06-regions-mini) and
[powerhouse-of-the-cell](../apps/06-powerhouse-of-the-cell), which reads the
whole mitochondrial genome as one interval.

### Reference only - no genotype at all

Some apps read only reference sequence and never touch your genotype. The
`reference` leaf of a gene or region is just a string of bases to compute over.

See [motif-finder](../apps/08-motif-finder), which scans a gene's reference for
restriction sites and reads nothing personal.

### The whole variant file - the `snp-indel/` slot

When you really need every record (a quality scan, a custom analysis the lenses
do not cover), declare the `snp-indel/` slot and read its `vcf` view. This is the
broadest request, and the most work: you parse the format yourself. The
decompressed `vcf` view is plain text, so a simple app can scan it line by line,
and a real one can use a genomics library.

See [vcf-mini](../apps/07-vcf-mini) for a simple line scan and
[vcf-roll-call](../apps/07-vcf-roll-call) for a full scan with `noodles-vcf`.

## Least privilege

The Ark mounts only the paths an app declares, and nothing else. Declaring one
rsID grants that one directory, not the variant file it came from and not its
neighbors. An app cannot read past what it asked for, and the owner approves a
specific, legible list. [permissions](../apps/02-permissions) demonstrates the
boundary: it reads a declared path, then fails to read an undeclared one.
