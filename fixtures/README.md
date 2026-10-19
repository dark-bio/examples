# Fixtures

These files stand in for what an Ark mounts, so `make run` works without one.
The default root models one invented genome on GRCh38.p14. Its 31-record small
variant file agrees with every rsID, gene and region answer beside it. The
genotypes, copy numbers and structural variant calls are made up, while the
coordinates, reference alleles and sequences are public data.

## Roots

| Root | Used by | What it models |
| :-- | :-- | :-- |
| `fixtures` | every app | typical calls, including phased, haploid and reference calls, no copies of UGT2B17, and the ACE insertion on both copies |
| `fixtures/no-call` | the rsID, copy number and structural apps | a no-call, `./.`, at every panel variant, a copy number segment without a count, and the ACE insertion with a copy uncalled |
| `fixtures/unanswered` | the rsID apps | no genotypes at the panel variants, as at reference sites in a variants-only call file, and no copy number or structural variant upload, so the `07` and `08` apps never start |

The rsID apps are `02-permissions`, the cilantro apps and `11-fortune-cookie`,
the copy number apps are the `07` ones, and the structural apps the `08` ones.
Pick a root with `FIXTURES`:

```sh
make run APP=04-cilantro-mini-rust FIXTURES=fixtures/no-call
```

The two smaller roots hold only the data those apps need, plus the reference
build they name coordinates on, so any other app stops at its missing grants
there.

## Editing them

Keep every file exactly as an Ark would serve it.

- A file holds only its value, with no trailing newline. Leave a file out to
  model an absent answer, and never leave one empty.
- `files/reference-genome/build` names the assembly.
- A `reference` holds exactly end - start + 1 forward-strand bases, with repeats
  in lowercase.
- `changes` holds one directory per alternate allele the genome carries, named
  after the span it changes. That's why the mitochondrial no-call at position
  16519 has none.
- Git can't store an empty directory, so each root lists its empty `changes`,
  `copies` and `datasets` folders in its `directories` file, and `make run`
  creates them.
- A place holds only the leaves an app reads, so UGT2B17 and the ACE window
  have no `reference`.
- `files/small-variants/vcf` is ordinary VCF text and keeps its newlines. Keep
  it in agreement with the files beside it.

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
- The UGT2B17 coordinates come from NCBI's
  [GRCh38.p14 annotation](https://www.ncbi.nlm.nih.gov/datasets/genome/GCF_000001405.40/).
  Its copy number segment is the deletion DRAGEN 4.4.7 calls over the gene in
  the 1000 Genomes samples.
- The ACE insertion's anchor and bases come from DRAGEN 4.4.7's structural
  variant calls for the 1000 Genomes samples, where 148 of the 150 carriers
  share one insertion.

UCSC counts from zero with an exclusive end, while the fixtures use 1-based,
inclusive coordinates. The sources were checked on 2026-09-15, and the copy
number and structural ones on 2026-10-19.
[docs/05-running.md](../docs/05-running.md) lists what local runs can't
reproduce.
