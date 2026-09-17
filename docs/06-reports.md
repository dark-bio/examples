# Reports

A report is what an app prints in its run pass, and it is the only thing an app
can send out of the sandbox. The owner reads it first, on their phone, and
decides whether it goes any further. Whoever ran the app then gets its exact
bytes, in a terminal from `ark app run`, or rendered as Markdown. So a report is
read three ways, on a phone, as raw text and as a rendered page, and it has to
work in all of them. This page is a guideline for that, not a format the Ark
checks. The Ark returns whatever the app printed, up to 1 MiB.

## What a report has to do

The owner approved the app before seeing a word of it, so the title and the
finding fit in the first ten lines, and the owner sees the answer before
scrolling. Everything the finding rests on is in the report, each value beside
the coordinate it came from, so a reader can check it and a second app could
reproduce it. The Ark reports the app's name and version and the paths it
granted beside the report, so the report carries none of them. And the report
is readable to the last byte. A report is only as trustworthy as it is
readable.

Two things follow from the sandbox. The run is deterministic, so a report
carries no date and no run id. The same app over the same data prints the same
report, which is what lets last year's be diffed against this year's, and an
app that lists a directory sorts the listing before printing it, since listing
order isn't promised. And the output is capped, so a report summarises. It
lists the evidence for its finding and counts the rest, since output past the
cap is dropped without a mark.

An app that exits non-zero returns nothing. Without `develop = true` the Ark
withholds standard output on failure and never returns standard error, so the
owner is left with a blank screen after approving the run. A failure exit is
for data that couldn't be read. Everything an app can state, including that
it has no answer, is a report.

## The shape

A report follows one order of ideas. The answer, what it rests on, how it
follows, what it doesn't establish, and where it came from. The answer sits
straight under the title with no heading of its own, and the rest are sections
with these default names.

```
# <Subject>       what it answers, in Title Case, and then at once the finding,
                  the answer in plain language, a paragraph or one small table

## Evidence       the values the finding rests on, each with its coordinate
## Method         how the finding follows, and from which study or guideline
## Limitations    what the app doesn't establish, as facts
## Sources        the studies named in Method, in that order
```

The title and the finding are the minimum. The four sections appear when they
have something to say. A finding with several parts, the axes of a panel, gives
each a `##` of its own before Evidence. A report that isn't a trait finding, a
map, a scan or a quality check, keeps the order and names its sections for what
they hold.

The title is the subject, not the app's name, which the Ark reports beside the
report, so an app called `drunk-o-type` prints a report titled "How Your Body
Handles a Drink". The same shape holds one variant, a panel, a gene, a whole
call file, and the summary a study asked for.

[03-cilantro-soapiness](../apps/03-cilantro-soapiness) is a light app, and
its report in this shape:

```markdown
# 🌿 Cilantro Taste Test

**Soap detector** 🧼. You carry two copies of C at rs72921001, the genotype
most strongly tied to tasting cilantro as dish soap. Blame *OR6A2*, the
olfactory receptor next door. C is also the reference base here, so a call
file that lists only variants would have stayed quiet about it. Yours didn't.

## Evidence

| Variant | Nearest gene | Position |
| :-- | :-- | :-- |
| rs72921001 | *OR6A2* | chr11:6868417, GRCh38.p14 |

Your call file says `|C|C` here, and the reference base is C.

## Method

Some people love cilantro. Others think it tastes like dish soap. Eriksson et
al. (2012) went looking for why, in a genome-wide study of self-reported
preference among European-ancestry participants, and rs72921001 is what they
found. The app counts your copies of C. Two is the strongest association, one
is somewhere in between, and zero is none. More copies, more soap.

## Limitations

One variant, one modest effect. The study measured what people said they
preferred, not what they tasted, and taste is polygenic and shaped by diet,
culture and exposure besides. Two copies doesn't mean you hate cilantro, and
zero doesn't mean you love it. The study was mostly people of European
ancestry, so elsewhere the effect is less well known, and this app doesn't
know how common your genotype is.

## Sources

1. Eriksson N, et al. A genetic variant near olfactory receptor genes
   influences cilantro preference. Flavour. 2012;1:22.
   https://doi.org/10.1186/2044-7248-1-22
2. NCBI dbSNP, rs72921001. https://www.ncbi.nlm.nih.gov/snp/rs72921001
```

## The sections

**The finding** says what was found, in the owner's own terms, and never more
than the evidence supports. It says "carries" and "is associated with", never
"you have" or "you will". A finding is a sentence and a value. A verdict label
can lead it, "Soap detector" or "Tomato Mode", as long as the value it rests on
follows in the same breath, since a reader shouldn't have to scroll to learn
what a label means. An absent answer is a finding too, stated as one, and
never read as two reference alleles:

```markdown
# 🌿 Cilantro Taste Test

**No answer.** No genotype is available at rs72921001, so this app has no
result. An absent genotype is not a reference call. A call file that records
only variants has no record at a reference site, and none at a site it didn't
cover, and the two can't be told apart here.
```

**Evidence** holds the values the finding rests on, one row per variant, gene
or interval, with its coordinate. A genotype is printed exactly as the file
holds it, `|C|C` and all. A pipe can't survive a table cell, where it has to be
escaped and reads as `\|` raw, so a genotype holding one goes in prose or in
code, or a panel table carries one column per allele. Never swap a slash for a
pipe, that is a different genotype. Absent values stay in the table, marked
absent. A long list is cut, with a count of what was left out.

Coordinates name their assembly when the app can read it. It sits in
`v1/genome/reference/build`, and reading it means granting
`v1/genome/reference`, which puts the stored reference genome on the approval
screen. The demos take that trade. An app that doesn't leaves the coordinate
unlabelled rather than labelling it wrongly, since an Ark may hold a GRCh37
genome.

**Method** explains how the finding follows from the evidence, in plain words,
and names each study inline, as author and year, with the population it was
established in. An app may carry allele frequencies or effect sizes from the
literature, and cites where they came from like any other claim.

**Limitations** says what the app didn't read and what the finding doesn't
establish, as facts rather than reassurance. One variant rarely tells a whole
story, and this section says how much this one tells.

**Sources** lists the studies Method named, in that order, in full, author,
year, title and journal, with a URL where the work has a stable one. Links
appear only here, and each is the fixed public address of the work it cites.
Nothing in a URL comes from the data.

## Voice

Pick a voice and keep it from the first line to the last. A serious subject
gets a professional report throughout. A light subject can be light
throughout. What a report never does is switch, a joke verdict over a clinical
table, or a punchline wrapped around a finding that touches real risk. A light
report can carry a heavy fact, and when it does, that sentence is plain and
says so, the way a friend drops the joke for a moment.

The examples are toys, and they read that way, charming and friendly whatever
they count. The serious voice is for heavier subjects, and none of the examples
has one yet. So that its register is on record, the cilantro finding above in
the serious voice:

```markdown
You carry two copies of the C allele at rs72921001, the genotype most strongly
associated with perceiving cilantro as soapy. C is the reference base at this
position, so a call file that lists only variants may not record it.
```

Whichever voice, the evidence, the limitations and the sources are the same,
and the finding states the value in words. Emoji belong in prose and headings
in a light report, never in a table, where they break alignment in a terminal,
and never in place of the value.

Genomics vocabulary, not clinic vocabulary. An app reports an rsID, a gene, a
genotype and a coordinate on a named assembly. It has no patient, no specimen,
no test, no result that is positive or negative, no clinical significance and
no recommendation. It names what a study found and how strongly, and stops.

Second person for the owner's own data. Genes in italics, alleles, genotypes
and paths in code. Every number with its unit, every percentile with its
reference population. Title Case for the title, sentence case for headings.

## Form

Plain Markdown, and a small subset of it, because the subset is what keeps a
report readable everywhere. One `#`, the `##` sections above, and `###` to
group the rows of an Evidence section. Paragraphs, lists and
tables with as few columns as carry the evidence, in rows of about 80
characters, the values the owner needs in the leftmost columns since a phone
is narrower still. A paragraph is one line of output, since every renderer
wraps it. A text figure in a code fence when it earns its place. Nothing else.

No images, no HTML, no footnotes and no encoded content, since the owner can't
read them. No dates and no run ids, which the sandbox can't give and the
journal already has. Standard error is for the developer and never part of the
report.

The minis print a bare line on purpose. They demonstrate a read, and the full
app beside each one demonstrates the report. `ark help apps` covers the
manifest and the sandbox limits, and `ark help output` how the CLI returns a
report.
