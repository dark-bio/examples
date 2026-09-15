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
  genes/ +                    one gene, by symbol
    <gene>/ +
      chromosome
      start
      end
      strand
      biotype
      sequence
      changes/
        <pos>/
          reference
          genotype
  regions/ +                  any interval
    <chr>/ +
      length
      <start>-<end>/ +
        sequence
        changes/
          <pos>/
            reference
            genotype
  rsids/ +                    one variant, by dbSNP identifier
    <rsid>/ +
      chromosome
      position
      reference
      genotype
  snp-indel/ +                the stored call file
    vcf  vcf.gz  vcf.gz.tbi  vcf.gz.gzi
  reference/ +                the stored reference genome
    build  fa  fa.gz  fa.gz.fai  fa.gz.gzi
  annotations/ +              the stored gene annotations
    sorted.gff  sorted.gff.gz  sorted.gff.gz.tbi  sorted.gff.gz.gzi
    original.gff  original.gff.gz  original.gff.gz.gzi
```

`+` marks a directory a manifest may grant. The four lenses, `positions`,
`genes`, `regions` and `rsids`, answer questions from the three stored datasets
beside them. A lens is usually what an app wants, since it hands back one small
value instead of a file format to parse.

## Rules every path follows

- **Coordinates** are 1-based and inclusive, on the assembly named in
  `v1/genome/reference/build`, such as `GRCh38.p14`. The main chromosomes are
  spelled `chr1` to `chr22`, `chrX`, `chrY` and `chrM`, exactly.
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
- **`changes` has limits.** Listing a gene's or an interval's `changes`, or
  looking up an entry in it, fails with "file too large" (EFBIG) when it spans
  more than 4000000 bases or holds more than 16384 positions. Split the interval
  and try again.
- **Sequences stream.** A `sequence` file holds exactly end - start + 1 bases,
  so its size is its length, and byte offset i is position start + i. Repeats
  are soft-masked in lowercase. Read long sequences as streams, since an app has
  100 MiB of memory.
- **The stored datasets are big.** `snp-indel/vcf` and `reference/fa` are
  decompressed on read and run to gigabytes, so read them line by line.
- **`available`** means the data a path needs is on the Ark, not that every gene
  or position has an answer.

## Genotypes

A `genotype` file holds the genotype of the call file's first sample, copied as
the record writes it.

- There is one allele per copy, so a haploid call, such as on chrM, reads `G`.
- `/` joins unphased copies and `|` phased ones, possibly mixed, and a leading
  phase marker is kept, as in `A/G`, `A|G` and `|A|G`.
- A missing allele stays a dot, as in `./.` or `A/.`. A bare `.` leaves the
  number of copies unknown.
- Inside a reference block, each reference allele reads as the reference base,
  as in `C/C`.
- Inside a longer reference allele, such as a deletion, each copy reads its
  letter at that position, or `*` where it has none, as in `T/*`.
- At a record's start, alleles read as written, so multi-base, `*` and symbolic
  alleles such as `<DEL>` appear.

Symbolic and breakend alleles can contain `/` or `|` themselves. Split a genotype
only outside matching angle brackets, and outside a breakend's mate position
between two `[` or two `]` characters.

A genotype is absent when no record covers its position, which in a variants-only
call file includes most reference sites. It's also absent when the record has no
genotype, when several records start at the position, and inside a symbolic
allele or a replacement of a different length. **Absence never means homozygous
reference.**

## Genes and variants at several places

A gene symbol or an rsID can sit at more than one place in the genome. Duplicate
placements count once. Copies on alternate contigs give way to one on a main
chromosome, and partial gene copies give way to a complete one. A copy on chrX
and one on chrY inside the same pseudoautosomal region count as the chrX copy,
and while the calls cover the chrY side, its `reference`, `genotype` and
`changes` read absent.

Anything still left at several places lists nothing. Read each place through
`positions/` or `regions/` instead.
