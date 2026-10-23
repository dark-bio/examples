# FHIR View

Reads genomic FHIR R4 JSON and renders its codes and components beside the
re-indented leaves. The stored JSON is compact; the display preserves key order
and content. The report counts resources without interpreting traits.

The app reads a site collection at rs671, the TAS2R38 collection and its change
resources, UGT2B17 copy resources, and a haploid site at chrM position 263.
The two genes and the haploid site are optional grants.
The public dataset directory distinguishes a missing copy number upload from
an empty answer. Public grants cannot be optional.

```sh
make run APP=12-fhir-view
make run APP=12-fhir-view FIXTURES=fixtures/no-call
make run APP=12-fhir-view FIXTURES=fixtures/unanswered
OPTIONAL=off make run APP=12-fhir-view
```

The last command declines the optional grants, leaving only rs671 and the
dataset directory. Ungranted and missing endpoints are reported as absent.
Other read errors stop the app.
The report reads at most three change and three copy endpoints, renders eight
resources per endpoint, and prints at most 12 KiB of each re-indented leaf.
Every cut includes the omitted count.

The [data path reference](../../docs/03-data-paths.md#fhir) describes the
resource shapes, and the [report guide](../../docs/06-reports.md) describes
the Markdown output.
