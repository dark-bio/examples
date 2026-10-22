# Data paths

An app reads the owner's data as plain, read-only files, with ordinary opens,
reads and directory listings. There's no query language and no API. This page
explains the tree and the rules every path follows. For the exact contract of
each path, ask the Ark itself:

```sh
ark data paths          # the tree, with grant and availability marks
ark data paths --json   # every path with its description, format and examples
```

The JSON reply is the reference to build against. Each entry carries:

- `path`, spelled the way a manifest names it
- `directory` and `grantable`
- `available`, whether the data the path needs is on this Ark now
- `description`, what it holds, when it is absent and when reading it fails
- `format`, the file's exact contents or what the directory lists
- `examples`, sample values with the most typical first

A path with a placeholder, such as `<gene>`, describes every value that fills it.

## The tree

```text
v1/genome/ +
  positions/ +                one position
    <chr>/ +
      length
      <pos>/ +
        reference
        genotype
        fhir
        changes/
        copies/
  genes/ +                    one gene, by symbol
    <gene>/ +
      chromosome
      start
      end
      strand
      biotype
      reference
      fhir
      changes/
      copies/
      positions/<pos>/        unlisted, a site inside the gene
  regions/ +                  any interval
    <chr>/ +
      length
      <start>-<end>/ +
        reference
        fhir
        changes/
        copies/
        positions/<pos>/      unlisted, a site inside the interval
  rsids/ +                    one variant, by dbSNP identifier
    <rsid>/ +
      chromosome
      position
      end
      reference
      genotype
      fhir
      changes/
      copies/
  datasets/ +                 which datasets are uploaded
    <name>/ +                 one empty folder per uploaded dataset
  files/
    small-variants/ +         the stored files, each its own grant
    structural-variants/ +
    copy-number-variants/ +
    reference-genome/ +
    gene-annotations/ +
```

`+` marks a directory a manifest may grant. The four places, `positions`,
`genes`, `regions` and `rsids`, answer everything known about a spot in the
genome, whichever file it came from. A place is usually what an app wants,
since it hands back small values instead of a file format to parse.

Every place holds the same two folders:

- `changes/` lists the alternate alleles the owner carries that reach the
  place, from the small variants and the structural variants alike.
- `copies/` lists the copy number segments that reach it.

A gene and an interval also hold every position inside them, under
`positions/<pos>/`, with the same leaves as a position of their own. These
aren't listed, so address them directly.

`datasets/` holds one empty folder per uploaded dataset, such as
`small-variants`, so an app can tell an empty answer from data the owner never
uploaded. Each folder is a public grant of its own, and granting one makes the
Ark refuse to run the app until that dataset is there. `files/` holds the stored files with their indexes, under the same
names, for an app that needs a whole file.

## Rules every path follows

- **Coordinates** are 1-based and inclusive, on the assembly named in
  `v1/genome/files/reference-genome/build`, such as `GRCh38.p14`. The main
  chromosomes are spelled `chr1` to `chr22`, `chrX`, `chrY` and `chrM`,
  exactly.
- **A file holds exactly its value**, with no trailing newline. An empty value,
  such as a gene without a biotype, is absent rather than an empty file.
- **Absent means no answer.** Opening a path that has no answer fails with "not
  found" (ENOENT). That's an ordinary outcome for an app to report.
- **Other errors mean the data couldn't be read.** Damaged stored data and
  malformed genotypes fail with an I/O error (EIO). An app should stop and say
  so, never treat it as absence.
- **Big sets aren't listed.** There are billions of positions, intervals and
  variants, so address them directly. `positions/<chr>/` and `regions/<chr>/`
  list only `length`, `genes/` lists every symbol, and `rsids/` lists nothing.
  Every position from 1 to its chromosome's length exists.
- **`changes` has limits.** Listing a place's `changes`, or looking up an entry
  in it, fails with "file too large" (EFBIG) when the place spans more than
  4000000 bases or holds more than 16384 entries. Split the interval and try
  again.
- **References stream.** A gene's or an interval's `reference` holds exactly
  end - start + 1 bases, so its size is its length, and byte offset i is
  position start + i. Repeats are soft-masked in lowercase. Read long ones as
  streams, since an app has 128 MiB of memory.
- **Stored files are big.** `files/small-variants/vcf` and
  `files/reference-genome/fa` are decompressed on read and run to gigabytes, so
  read them line by line.
- **Public grants never reach the owner.** `datasets` and its folders,
  `files/reference-genome` and `files/gene-annotations` say nothing about the
  owner's DNA, so the approval doesn't list them.
- **`available`** means the data a path needs is on the Ark, not that every
  gene or position has an answer.

## Changes

Each entry under `changes/` is one alternate allele the owner carries, named
`<start>-<end>` after the bases it changes. A change reaches a place when the
bases it changes overlap it, widened by any confidence intervals the caller
gave. So a large deletion lists under every position it may cover, under the
same name everywhere. Entries sharing a span on a chromosome add `-2`, `-3`
and so on.

An entry's leaves, each present only when it has a value:

- `type` is the kind of change: `SNV`, `MNV`, `DEL`, `INS` or `DELINS` for a
  spelled-out allele, the symbol for a symbolic one such as `DUP:TANDEM`, and
  `BND` for a breakend.
- `bases` are the bases that replace the changed span, after any padding base.
- `carried` is how many of the genotype's copies carry the allele, and
  `uncalled` how many have no call.
- `dataset` is the file it comes from, `small-variants` or
  `structural-variants`.
- `copy` and `phase-set` place a phased call's copies.
- `failed` lists the filters the record failed, and `imprecise` marks a call
  the caller flagged as imprecise.
- `start-range` and `end-range` give the possible first and last changed base.
- `id` is the record's ID, such as an rsID, and `length` a symbolic
  insertion's length.
- `count`, `evidence` and `event` carry the record's copy number, evidence
  and event.
- `mate`, `mate-id`, `joins` and `mate-joins` describe a breakend's join.
- `fhir` holds the change as a FHIR resource.

## Copies

Each entry under `copies/` is one copy number segment that reaches the place,
named `<start>-<end>` after its span, normal ones included. Its `count` is the
total copy number over the segment, `failed` lists the filters it failed, and
`start-range` and `end-range` give its uncertain edges. A count never says
gain or loss by itself, since what is normal depends on the chromosome and the
person's sex. `copies/` is empty while no copy number file is uploaded.
Its `fhir` leaf holds the segment as a FHIR resource when `count` has an answer.

## FHIR

Each `fhir` leaf is compact FHIR R4 4.0.1 JSON, with media type
`application/fhir+json` and no trailing newline. Its components follow the
[HL7 Genomics Reporting STU3 profiles](https://hl7.org/fhir/uv/genomics-reporting/STU3/).

| Endpoint | Resource |
| :-- | :-- |
| `changes/<entry>/fhir` | one Variant `Observation`, LOINC `69548-6` |
| `copies/<entry>/fhir` | one Variant `Observation` with a copy number component |
| a position or rsID's `fhir` | a `collection` `Bundle` containing a Genotype `Observation`, LOINC `84413-4`, and its variants |
| a gene or interval's `fhir` | a `collection` `Bundle` containing its changes and answered copy numbers |

Positions inside genes and intervals have the same site leaf. The genotype is
preserved exactly in `valueCodeableConcept.text`. Its `derivedFrom` references
one Variant per distinct non-reference sequence allele. Spanning deletions
(`*`), symbolic alleles and breakends remain in the genotype text but derive no
Variant here. Their full descriptions are in `changes/<entry>/fhir`.
A fully called reference site instead has one Variant valued `Absent`.
Missing and skipped alleles never imply `Absent`. Missing site answers and
missing copy counts have no `fhir` leaf. Empty span answers have an empty
collection.

Only known component values are included. Gene symbols are text because the
annotations do not hold HGNC identifiers. Copy counts do not infer gain or
loss. RefSeq accessions are included only for GRCh37 and GRCh38, with chrM
included only for GRCh38. Allelic state is included only for fully called
alternate alleles, never on chrM. Haploid calls carry it only on chrX and chrY.

The JSON contains no dates, `subject`, `specimen` or `performer`. Resource IDs
and `urn:uuid:` entry URLs use UUIDv5 over canonical paths in the URL namespace.
Site aliases use their `positions/<chr>/<pos>/fhir` path. The same data produces
identical JSON. Gene and interval leaves share the 4000000-base and
16384-entry limits; their entry count includes both changes and copies.
Every FHIR leaf also has a 16 MiB JSON limit. Exceeding a limit fails with
"file too large" (EFBIG). The fixture's TAS2R38 change at 141972731 occupies
2,551 bytes. Including collection overhead, about 6,400 changes of that size
fit in one leaf. Longer alleles and additional components reduce that count.

[12-fhir-view](../apps/12-fhir-view) renders these leaves beside their
re-indented JSON, preserving key order and content.

## Genotypes

A `genotype` file holds the genotype of the call file's first sample at one
position.

- There is one allele per copy, so a haploid call, such as on chrM, reads `G`.
- `/` joins unphased copies and `|` phased ones, possibly mixed, and a leading
  phase marker is kept, as in `A/G`, `A|G` and `|A|G`.
- A missing allele stays a dot, as in `./.` or `A/.`. A bare `.` leaves the
  number of copies unknown.
- Inside a reference block, each reference allele reads as the reference base,
  as in `C/C`.
- Inside a longer reference allele, each copy reads its letter at that
  position, or `*` where it has none.
- A deletion covering the position, from either variant file, turns each copy
  it removes into `*`, as in `T/*`. When the deletion sits in the call's own
  phase set, the copies keep their order and `|`, as in `T|*`.
- At a record's start, alleles read as written, so multi-base, `*` and symbolic
  alleles such as `<DEL>` appear.

Symbolic and breakend alleles can contain `/` or `|` themselves. Split a
genotype only outside matching angle brackets, and outside a breakend's mate
position between two `[` or two `]` characters.

A genotype is absent when no record covers its position, which in a
variants-only call file includes most reference sites. It's also absent when
the record has no genotype, when several records start at the position, inside
a symbolic allele or a replacement of a different length, and when overlapping
deletions leave the copies unsettled. **Absence never means homozygous
reference.**

## Genes and variants at several places

A gene symbol or an rsID can sit at more than one place in the genome.
Duplicate placements count once. Copies on alternate contigs give way to one on
a main chromosome, and partial gene copies give way to a complete one. A copy
on chrX and one on chrY inside the same pseudoautosomal region count as the
chrX copy, and while the calls cover the chrY side, its answers read absent.

Anything still left at several places lists nothing. Read each place through
`positions/` or `regions/` instead.
