# Reading data

An app reads its grants as ordinary files. This page covers the three things
every app has to get right, which are telling absence from failure, reading
sequences, and parsing genotypes. Then it walks the grants from narrowest to
broadest, each with an example to run.

## Absence and failure

Opening a path that has no answer fails with "not found". That is a normal
outcome, and the app reports it. Every other error means the data couldn't be
read, so the app should print why to standard error and exit with a failure,
instead of guessing an answer.

```rust
match fs::read_to_string(base.join("genotype")) {
    Ok(genotype) => Some(genotype),
    Err(err) if err.kind() == io::ErrorKind::NotFound => None,
    Err(err) => {
        eprintln!("could not read the genotype: {err}");
        std::process::exit(1);
    }
}
```

In Go, test for absence with `errors.Is(err, fs.ErrNotExist)`, and in C with
`errno == ENOENT`. A file holds exactly its value with no trailing newline, so
there's nothing to trim.

Keep in mind what absence means. A genotype can be absent for someone carrying
two reference alleles, since a variants-only call file has no records at
reference sites. An app must never read absence as a result.

## Reading sequences

A gene's or an interval's `sequence` holds exactly end - start + 1 bases, so
the file's size is the span's length and byte offset i is position start + i.
Repeats are soft-masked in lowercase, so uppercase bases before matching them.
A long sequence can outgrow an app's 100 MiB of memory, so read it in chunks
through a buffered reader rather than whole.

## Parsing genotypes

A genotype reads as the call file wrote it, and
[03-data-paths.md](03-data-paths.md#genotypes) lists every shape. The ones an
app meets most are `A/G` and `A|G`, a haploid `G`, a missing allele such as
`./.`, and a leading phase marker such as `|A|G`.

- **Strip a leading `/` or `|`** before splitting.
- **Treat a `.` allele as missing.** It makes a copy count inconclusive, never
  zero.
- **Count the alleles you get**, since not every call has two.

At a known SNP, splitting on `/` and `|` is enough. An app that walks `changes`
can meet symbolic alleles such as `<DEL>` and breakends, which contain those
characters themselves, so it has to split only outside them.
[05-bitter-meter](../apps/05-bitter-meter) shows how.

## From narrowest to broadest

The owner reads every requested path before approving, so ask for the least that
does the job.

### One variant

Grant one `rsids/<rsid>` directory and read its `genotype`, `chromosome`,
`position` and `reference`.

```toml
datasets = ["v1/genome/rsids/rs72921001"]
```

[03-cilantro-mini-rust](../apps/03-cilantro-mini-rust), with Go and C twins,
shows the bare read. [03-cilantro-soapiness](../apps/03-cilantro-soapiness)
turns it into a report.

### A panel of variants

Grant one `rsids/` directory per variant. The owner sees each site by name.
[04-drunk-o-type](../apps/04-drunk-o-type) reads eleven.

### One position

Grant `positions/<chr>/<pos>` to read the `reference` base and the `genotype` at
a coordinate, without going through dbSNP. Every position on a chromosome
exists, while its `genotype` answers only where a record covers it.

### A gene

Grant `genes/<gene>` to read its `chromosome`, `start`, `end`, `strand`,
`biotype` and `sequence`, and its `changes`, one directory per position where a
record starting inside the gene holds an ALT allele. A gene can have no
`changes` directory at all, and a very long or very variable one fails with
"file too large". [05-genes-mini](../apps/05-genes-mini) shows the bare read,
and [05-bitter-meter](../apps/05-bitter-meter) turns TAS2R38 into a report.

### An interval

Grant `regions/<chr>/<start>-<end>` for any stretch of a chromosome, with the
same `sequence` and `changes` as a gene. When `changes` fails with "file too
large", split the interval and list each half.
[06-regions-mini](../apps/06-regions-mini) shows the bare read, and
[06-powerhouse-of-the-cell](../apps/06-powerhouse-of-the-cell) reads the whole
mitochondrial genome as one interval.

A gene or interval grant includes its `changes`, even if the app never lists
them. [08-motif-finder](../apps/08-motif-finder) reads only a gene's sequence,
yet its grant still covers the gene's variants.

### The whole call file

Grant `snp-indel` to read the `vcf` itself, when no lens answers the question.
It is the broadest request there is, and the file runs to gigabytes, so read it
line by line. [07-vcf-mini](../apps/07-vcf-mini) counts records with a plain
line scan, and [07-vcf-roll-call](../apps/07-vcf-roll-call) parses the whole
file with `noodles-vcf`.

## Least privilege

The Ark mounts only what a manifest grants. Granting one rsID gives that
directory, not the call file behind it or its neighbours.
[02-permissions](../apps/02-permissions) reads a granted path, then shows that an
undeclared one can't be opened.
