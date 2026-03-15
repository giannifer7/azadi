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

Azadi protects generated files from accidental overwriting. After each
successful run it stores a baseline copy of every file it wrote in
`_azadi_work/__old__/`. On the next run, before writing, it compares the
current `gen/` file against that baseline:

- **File unchanged since last run** — azadi overwrites it with the new
  content as usual.
- **File modified externally** — azadi stops with a `ModifiedExternally`
  error and leaves the file untouched. The message names the file so you
  can decide what to do:
  - To accept the regenerated version: restore the file from version
    control (or delete it) and rerun azadi.
  - To keep your manual change: edit the literate source to match your
    intent and rerun azadi.

If you need to suppress this protection (e.g. in a CI pipeline that always
regenerates from scratch), pass `--allow-overwrites`.

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
