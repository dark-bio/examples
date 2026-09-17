# Fixtures

These files stand in for what an Ark mounts, so `make run` works without one.
The default root models one invented genome on GRCh38.p14. Its 31-record call
file agrees with every rsID, gene and region answer beside it. The genotypes are
made up, while the coordinates, reference alleles and sequences are public data.

## Roots

| Root | Used by | What it models |
| :-- | :-- | :-- |
| `fixtures` | every app | typical calls, including phased, haploid and reference calls |
| `fixtures/no-call` | the rsID apps | a no-call, `./.`, at every panel variant |
| `fixtures/unanswered` | the rsID apps | coordinates and reference alleles but no genotypes, as at reference sites in a variants-only call file |

The rsID apps are `02-permissions`, the cilantro apps, `04-drunk-o-type` and
`09-fortune-cookie`. Pick a root with `FIXTURES`:

```sh
make run APP=03-cilantro-mini-rust FIXTURES=fixtures/no-call
```

The two smaller roots hold only the variant data those apps need, plus the
reference build they name coordinates on, so any other app stops at its missing
grants there.

## Editing them

Keep every file exactly as an Ark would serve it.

- A file holds only its value, with no trailing newline. Leave a file out to
  model an absent answer, and never leave one empty.
- `reference/build` names the assembly.
- A `sequence` holds exactly end - start + 1 forward-strand bases, with repeats
  in lowercase.
- `changes` lists only the positions where a record starting there holds an ALT
  allele. That's why the mitochondrial no-call at position 16519 has no entry.
- `snp-indel/vcf` is ordinary VCF text and keeps its newlines. Keep it in
  agreement with the files beside it.

## Sources

- The rsID placements on GRCh38 come from the
  [NCBI Variation API](https://api.ncbi.nlm.nih.gov/variation/v0/refsnp/72921001),
  queried for each identifier.
- The TAS2R38 sequence comes from
  [UCSC](https://api.genome.ucsc.edu/getData/sequence?genome=hg38;chrom=chr7;start=141972630;end=141973773).
- The mitochondrial sequence comes from
  [UCSC](https://api.genome.ucsc.edu/getData/sequence?genome=hg38;chrom=chrM;start=0;end=16569).
- The chr1 reference bases in the call file come from
  [UCSC](https://api.genome.ucsc.edu/getData/sequence?genome=hg38;chrom=chr1;start=100000;end=101800).

UCSC counts from zero with an exclusive end, while the fixtures use 1-based,
inclusive coordinates. The sources were checked on 2026-09-15.
[docs/05-running.md](../docs/05-running.md) lists what local runs can't
reproduce.
