# Reading data

An app reads its grants as ordinary files. This page covers the three things
every app has to get right, which are distinguishing absence from failure,
reading references, and parsing genotypes. Then it walks the grants from
narrowest to broadest, each with an example to run.

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

In Go, test for absence with `errors.Is(err, fs.ErrNotExist)`, in C with
`errno == ENOENT`, and in Python by catching `FileNotFoundError`. A file holds
exactly its value with no trailing newline, so there's nothing to trim.

Keep in mind what absence means. A genotype can be absent for someone carrying
two reference alleles, since a variants-only call file has no records at
reference sites. An app must never read absence as a result.

Absence can also mean the data was never uploaded. `v1/genome/datasets` lists
one empty file per dataset the Ark holds, and it's a public grant the owner
never has to approve. [07-doping-test](../apps/07-doping-test) reads it to tell
a genome without a copy number file from one whose caller left the gene out.
[08-sprint-or-marathon](../apps/08-sprint-or-marathon) reads it to tell a D/D
genome from one without a structural variant file.

## Reading references

A gene's or an interval's `reference` holds exactly end - start + 1 bases, so
the file's size is the span's length and byte offset i is position start + i.
Repeats are soft-masked in lowercase, so uppercase bases before matching them.
A long reference can outgrow an app's 128 MiB of memory, so read it in chunks
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
- **Read a `*` as a copy a deletion removed.** It carries neither the reference
  letter nor the alternate one.

At a known SNP, splitting on `/` and `|` is enough. A genotype can still hold
symbolic alleles such as `<DEL>` and breakends, which contain those characters
themselves, so split only outside them. An app that walks `changes` reads each
entry's `type`, `bases` and `carried` instead.
[05-bitter-meter](../apps/05-bitter-meter) shows how.

## From narrowest to broadest

The owner reads every requested path before approving, so ask for the least that
does the job.

### One variant

Grant one `rsids/<rsid>` directory and read its `genotype`, `chromosome`,
`position`, `end` and `reference`.

```toml
[reads]
paths = ["v1/genome/rsids/rs72921001"]
```

[04-cilantro-mini-rust](../apps/04-cilantro-mini-rust), with its Go, C and
Python siblings, shows the bare read.
[04-cilantro-soapiness](../apps/04-cilantro-soapiness) turns it into a report.

### A panel of variants

Grant one `rsids/` directory per variant. The owner sees each site by name.
[11-fortune-cookie](../apps/11-fortune-cookie) reads three.

### One position

Grant `positions/<chr>/<pos>` to read the `reference` base and the `genotype` at
a coordinate, and the `changes` and `copies` reaching it, without going through
dbSNP. Every position on a chromosome exists, while its `genotype` answers only
where a record covers it.

### A gene

Grant `genes/<gene>` to read its `chromosome`, `start`, `end`, `strand`,
`biotype` and `reference`, and its `changes`, one directory per alternate
allele the owner carries that reaches the gene. A very long or very variable
gene fails with "file too large". [05-genes-mini-rust](../apps/05-genes-mini-rust) shows the bare read,
and [05-bitter-meter](../apps/05-bitter-meter) turns TAS2R38 into a report.

Its `copies` lists the segments of the owner's copy number file that reach the
gene, each with a `count`. A whole gene can go missing, and only a copy number
file shows it. [07-copies-mini-rust](../apps/07-copies-mini-rust) shows the bare
read, and [07-doping-test](../apps/07-doping-test) turns UGT2B17 into a report.

### An interval

Grant `regions/<chr>/<start>-<end>` for any stretch of a chromosome, with the
same `reference` and `changes` as a gene. When `changes` fails with "file too
large", split the interval and list each half.
[06-regions-mini-rust](../apps/06-regions-mini-rust) shows the bare read, and
[06-powerhouse-of-the-cell](../apps/06-powerhouse-of-the-cell) reads the whole
mitochondrial genome as one interval.

A structural variant sits in `changes` like any other change, with `dataset`
naming its file, so a short interval is enough to find one.
[08-structural-mini-rust](../apps/08-structural-mini-rust) lists the ACE
insertion from a 101-base window, and
[08-sprint-or-marathon](../apps/08-sprint-or-marathon) turns it into a report.

A gene or interval grant includes its `changes`, `copies` and positions, even
if the app never reads them. [10-motif-finder](../apps/10-motif-finder) reads
only a gene's reference, yet its grant still covers the gene's variants.

### A whole file

Grant a folder under `files/`, such as `files/small-variants`, to read its
`vcf` itself, when no place answers the question. It is the broadest request
there is, and a small variant file runs to gigabytes, so read it line by line,
through a large buffer. Every refill leaves the sandbox, so a small default
buffer can cost ten times the scan time.
[09-vcf-mini-rust](../apps/09-vcf-mini-rust) counts records with a plain line
scan, and [09-vcf-roll-call](../apps/09-vcf-roll-call) parses the whole file
with `noodles-vcf`.

## Least privilege

The Ark mounts only what a manifest grants. Granting one rsID gives that
directory, not the call file behind it or its neighbours.

An optional grant the owner declined, or whose data the Ark lacks, isn't
mounted, so its directory is missing. An app reads that like any absent answer,
and it can't distinguish a decline from missing data. So an app checks for the
directory before reading an optional grant, and reports its absence as a
finding, never as an error.

[02-permissions](../apps/02-permissions) reads a granted path and an optional
one, then shows that an undeclared path can't be opened.
