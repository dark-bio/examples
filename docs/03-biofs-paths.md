# BioFS paths

Everything an app reads is a plain file. The Ark exposes the owner's data as a
read-only file tree, and an app reads it with ordinary file operations: open,
read, list a directory. There is no query language and no API to call. This page
is the map of that tree.

There are two kinds of paths:

- **Slot files** are the ingested data itself, served from disk: the variant
  file, the reference genome, the gene annotations. They appear only once that
  data has been loaded onto the device.
- **Lenses** are answers synthesized on read. A lens path does not exist on
  disk; reading it makes the Ark compute the answer from the slots. The lenses
  hand back small, plain values (a genotype, a coordinate, a sequence) so an app
  never has to parse a genomic file format. Most apps should use a lens.

The tree is versioned. Everything lives under `/v1/`; a future layout would add
`/v2/` beside it without breaking `/v1/` readers. An app reaches these paths by
joining them onto the data directory it is given as its first argument (`/`).

## The tree

A path is present only when its inputs are. A lens needs its slots loaded; a
slot needs its data ingested. Absent paths simply do not appear.

```text
/
└── v1/
    ├── README.md                         generated map of what is loaded and what each lens answers
    └── genome/
        ├── build                         the reference assembly, e.g. GRCh38.p14
        │
        ├── snp-indel/                     SLOT: your variants (present when a VCF is ingested)
        │   ├── vcf                          decompressed, seekable
        │   ├── vcf.gz                        bgzf, as stored
        │   ├── vcf.gz.tbi                    tabix index
        │   └── vcf.gz.gzi                    gzip index
        │
        ├── reference/                     SLOT: the reference genome (present when a FASTA is ingested)
        │   ├── fa
        │   ├── fa.gz
        │   ├── fa.gz.fai
        │   └── fa.gz.gzi
        │
        ├── annotations/                   SLOT: gene annotations (present when a GFF is ingested)
        │   ├── original.gff                  original feature order
        │   ├── original.gff.gz
        │   ├── original.gff.gz.gzi
        │   ├── sorted.gff                    coordinate-sorted
        │   ├── sorted.gff.gz
        │   ├── sorted.gff.gz.tbi
        │   └── sorted.gff.gz.gzi
        │
        ├── positions/                     LENS: one site (needs reference + snp-indel)
        │   └── <chr>/
        │       ├── length                    the chromosome's length in bases
        │       └── <pos>/
        │           ├── reference             reference base(s) at the position
        │           └── genotype              your genotype there
        │
        ├── rsids/                         LENS: one variant by dbSNP id (needs dbSNP catalog + reference + snp-indel)
        │   └── <rsid>/
        │       ├── chromosome
        │       ├── position
        │       ├── reference
        │       └── genotype
        │
        ├── genes/                         LENS: one gene by symbol (needs annotations + reference + snp-indel)
        │   └── <gene>/
        │       ├── chromosome
        │       ├── start
        │       ├── end
        │       ├── strand
        │       ├── biotype
        │       ├── reference                 the gene's reference sequence
        │       └── changes/
        │           └── <pos>/
        │               ├── reference
        │               └── genotype
        │
        └── regions/                       LENS: an arbitrary interval (needs reference + snp-indel)
            └── <chr>/
                ├── length
                └── <start>-<end>/
                    ├── reference             the interval's reference sequence
                    └── changes/
                        └── <pos>/
                            ├── reference
                            └── genotype
```

## Notes

- **`<chr>` is a stable name** (`chr1`, `chrM`), not a build-specific accession,
  so a path that names a chromosome stays valid across assembly patches.
- **Lens values are plain.** A `genotype` is a string like `C/C` or `A|C` (`|`
  when phased, `.` for an uncalled allele). A `reference` is the base or
  sequence. There is no REF/ALT index to decode and no file format to parse;
  that is the point of a lens.
- **A `<pos>` or `<gene>` directory appears only if there is an answer there.**
  Reading `rsids/<rsid>/genotype` for a site your data does not cover returns a
  read error, not an empty file. Apps treat that as "not covered."
- **`/v1/README.md` is generated live** on the device from the current slot
  state, so it always reflects what is actually loaded. An app (or an LLM handed
  the file) can read it to discover the tree without prior knowledge.

## What the examples read

The examples touch the lenses and the variant slot:

- `rsids/` - [cilantro](../apps/03-cilantro-soapiness), [drunk-o-type](../apps/04-drunk-o-type), [fortune-cookie](../apps/09-fortune-cookie)
- `genes/` - [bitter-meter](../apps/05-bitter-meter), [genes-mini](../apps/05-genes-mini), [motif-finder](../apps/08-motif-finder)
- `regions/` - [powerhouse-of-the-cell](../apps/06-powerhouse-of-the-cell), [regions-mini](../apps/06-regions-mini)
- `snp-indel/` - [vcf-roll-call](../apps/07-vcf-roll-call), [vcf-mini](../apps/07-vcf-mini)

The `reference/`, `annotations/`, and `positions/` paths are real but no example
reads them directly; the lenses above are built from them.
