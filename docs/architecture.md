# Azadi architecture

Azadi is a literate programming toolchain. Source files are written as
annotated documents (Markdown, AsciiDoc, etc.) that contain both prose and
named code chunks. The `azadi` command processes them and writes the final
source files.

## Transformation pipeline

```
Literate source (.md / .adoc / …)
        │
        ▼
  azadi-macros          expands %def / %set / %if / %rhaidef / %pydef calls
        │               into an intermediate noweb document
        ▼
  azadi-noweb           extracts <[chunk]> references, assembles them,
        │               optionally runs a formatter (e.g. rustfmt)
        ▼
  gen/ (output files)   written only when content changes
```

The two passes run in-process when you invoke the combined `azadi` binary.
The separate `azadi-macros` and `azadi-noweb` binaries exist for pipeline
use but are not needed for normal work.

## Source of truth

The literate document is the **only** source of truth. Files under `gen/`
are derived artefacts — editing them directly is always wrong because the
next `azadi` run will overwrite them (or refuse to, and tell you why; see
below).

## Directory layout

```
project/
├── src/                   literate source files
├── gen/                   generated output files  ← do not edit
└── _azadi_work/
    ├── (scratch files)    intermediate private copies before formatting
    └── __old__/           last content azadi wrote  ← modification baseline
```

The `_azadi_work/` tree is private to the tool. Commit `gen/` to version
control; ignore `_azadi_work/`.

## Content-based writes

`azadi-noweb` compares the freshly generated content against what is already
in `gen/` before writing. If they are identical the file is left untouched,
keeping build-system timestamps stable and avoiding unnecessary recompilation.

## What happens when you edit a generated file

Azadi **always overwrites** `gen/` files with the freshly generated content.
Your edit is silently lost.

The `SafeFileWriter` API does support a `modification_check` mode that
compares each `gen/` file against the baseline in `_azadi_work/__old__/`
and returns a `ModifiedExternally` error instead of overwriting — but this
is not yet exposed as a CLI flag. When using azadi as a library you can
enable it via `SafeWriterConfig::modification_check`.

Either way, the right response to an accidental edit is:

- Restore the file from version control and rerun azadi, **or**
- Edit the literate source to match your intent and rerun azadi.

## Build-system integration

`--depfile` writes a Makefile-format depfile after each run; `--stamp`
touches a file on success. Together they let a single build rule cover an
entire directory tree:

```meson
custom_target('gen',
  output  : ['gen.stamp'],
  depfile : 'gen.d',
  command : [azadi,
             '--dir',    meson.current_source_dir() / 'src',
             '--ext',    'adoc',
             '--include', meson.current_source_dir(),
             '--gen',    meson.current_source_dir() / 'gen',
             '--stamp',  '@OUTPUT0@',
             '--depfile', '@DEPFILE@'],
)
```

> List only the stamp in `output`, never the `.d` file — Ninja consumes
> depfiles into its internal database and will rerun forever if the `.d`
> file is also declared as an output.

## Formatter hooks

`--formatter EXT=COMMAND` runs a formatter on each generated file with the
matching extension before it is compared and written. Example:

```
azadi --formatter rs=rustfmt src/main.adoc --gen gen
```

The formatter receives the private scratch copy; the formatted result is
then used for content comparison and written to `gen/`.
