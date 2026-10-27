# Fixtures

These files stand in for what an Ark mounts, so `make run` works without one.
The default root models one invented genome on GRCh38.p14. Its 37-record small
variant file agrees with every rsID, gene and region answer beside it. The
genotypes, copy numbers and structural variant calls are made up, while the
coordinates, reference alleles and sequences are public data.

## Roots

| Root | Used by | What it models |
| :-- | :-- | :-- |
| `fixtures` | every app | typical calls, including phased, haploid and reference calls, no copies of UGT2B17, and the ACE insertion on both copies |
| `fixtures/no-call` | the rsID, copy number and structural apps | a no-call, `./.`, at every panel variant, a copy number segment without a count, and the ACE insertion with a copy uncalled |
| `fixtures/unanswered` | the rsID apps | no genotypes at the panel variants, as at reference sites in a variants-only call file, and no copy number or structural variant upload, so the `07` and `08` apps never start |

The rsID apps are `02-permissions`, the cilantro apps, `11-fortune-cookie`,
`12-pigment-variants` and the composed `12-eye-color`. The copy number apps
are the `07` ones, and the structural apps the `08` ones.
Pick a root with `FIXTURES`:

```sh
make run APP=04-cilantro-mini-rust FIXTURES=fixtures/no-call
```

The two smaller roots hold only the data those apps need, plus the reference
build they name coordinates on, so any other app stops at its missing grants
there.

## Eye color panel

The six invented genotypes below give the original IrisPlex model a 94.634%
blue probability, 3.525% intermediate and 1.840% brown. Coordinates and
reference bases come from dbSNP's GRCh38.p14 placement, checked on 2026-10-27.
Positions are 1-based, in bases, and alleles use the forward strand.

| Variant | Gene | Position on GRCh38.p14 | Reference | Genotype |
| :-- | :-- | :-- | :-- | :-- |
| rs12913832 | *HERC2* | chr15:28120472 | A | G/G |
| rs1800407 | *OCA2* | chr15:27985172 | C | C/C |
| rs12896399 | *SLC24A4* | chr14:92307319 | G | T/T |
| rs16891982 | *SLC45A2* | chr5:33951588 | C | G/G |
| rs1393350 | *TYR* | chr11:89277878 | G | G/G |
| rs12203592 | *IRF4* | chr6:396321 | C | C/C |

Each has a matching record in the small variant file, including the three
reference genotypes. The `no-call` root gives all six `./.`, and `unanswered`
omits their genotype files. Both keep their coordinates and reference bases.
The composed report gives no estimate for either root. The model's standalone
sample input holds the default genotypes, independent of the fixture root.

The [model README](../apps/12-eye-color-model) gives the coefficients and
their source. The [gatherer README](../apps/12-pigment-variants) describes
the text passed to it.

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
- The eye color panel uses dbSNP records
  [rs12913832](https://api.ncbi.nlm.nih.gov/variation/v0/refsnp/12913832),
  [rs1800407](https://api.ncbi.nlm.nih.gov/variation/v0/refsnp/1800407),
  [rs12896399](https://api.ncbi.nlm.nih.gov/variation/v0/refsnp/12896399),
  [rs16891982](https://api.ncbi.nlm.nih.gov/variation/v0/refsnp/16891982),
  [rs1393350](https://api.ncbi.nlm.nih.gov/variation/v0/refsnp/1393350) and
  [rs12203592](https://api.ncbi.nlm.nih.gov/variation/v0/refsnp/12203592).
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
inclusive coordinates. The other sources were checked on 2026-09-15, and the
copy number and structural ones on 2026-10-19.
[docs/05-running.md](../docs/05-running.md) lists what local runs can't
reproduce.
