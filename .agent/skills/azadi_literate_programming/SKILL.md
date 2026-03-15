---
name: Azadi Literate Programming
description: Guidelines and patterns for working in codebases that use the azadi literate programming toolchain.
---

# Azadi Literate Programming

Azadi is a literate programming toolchain. Source files are written as
annotated documents (Markdown, AsciiDoc, etc.) containing macro calls and
named code chunks. The `azadi` command processes them and writes the final
source files.

## Typical invocation

```bash
azadi source.md --gen src
azadi --dir docs --ext adoc --include . --gen src
```

The `--dir` mode recursively discovers driver files (those not `%include`d
by another file) and processes each one. No command-line changes are needed
when new files are added.

## Two passes, one command

`azadi` runs two passes in sequence, in-process:

1. **azadi-macros** — expands `%macro(...)` calls into an intermediate
   noweb document
2. **azadi-noweb** — extracts `<[chunk]>` references and writes output files

The separate `azadi-macros` and `azadi-noweb` binaries exist for advanced
pipeline use but are not needed for normal work.

## Chunk syntax (defaults)

The comment marker before a delimiter matches the language of the chunk's
content. Use `//` for Rust, C, and similar; use `#` for Python, shell, TOML,
etc. For azadi source or plain text there is no host language, so omit the
comment marker entirely.

**Rust** (`//` comment marker):

```rust
// <[@file src/lib.rs]>=
pub mod utils;
// <[utils-module]>
// @

// <[utils-module]>=
pub fn helper() {}
// @
```

**Azadi / plain text** (no comment marker):

```azadi
<[@file config/default.toml]>=
[server]
port = <[server-port]>
@

<[server-port]>=
8080
@
```

- `<[@file path]>=` — declares a file output chunk; path may start with `~/`
- `<[name]>=` — declares a named chunk
- `<[name]>` inside a chunk body — expands that chunk inline, preserving indentation
- `// @` / `# @` / `@` — ends the current chunk (marker must match what precedes delimiters)
- Comment markers before delimiters are stripped automatically; defaults are
  `#` and `//` but any set can be configured via `--comment-markers`
  (e.g. `--comment-markers "--,;;"` for Lua/Scheme)

Delimiters are configurable: `--open-delim`, `--close-delim`, `--chunk-end`.

## Macro syntax

```
%def(name, param1, ..., body)   — define a macro
%(varname)                      — interpolate a variable
%set(name, value)               — set a variable
%if(cond, then, else)           — conditional
%include(path)                  — include another file
%import(path)                   — include but discard output (load definitions)
%rhaidef(name, params..., body) — Rhai-scripted macro (logic, arithmetic)
%pydef(name, params..., body)   — Python-scripted macro (via monty)
```

Always wrap macro bodies in `%{ ... %}` — required when they contain commas
or parentheses, and good style otherwise. Wrap non-trivial arguments too.
Leading whitespace inside `%{` is preserved:

```
%def(greet, name, %{Hello, %(name)!%})
%greet(%{World%})        → Hello, World!
%greet(%{ World%})       → Hello,  World!   ← note the extra leading space
```

## Build system integration

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
             '--gen',    meson.current_source_dir() / 'src',
             '--stamp',  '@OUTPUT0@',
             '--depfile', '@DEPFILE@'],
)
```

> List only the stamp in `output`, never the `.d` file — Ninja consumes
> depfiles into its internal database and will rerun forever if the `.d`
> file is also declared as an output.

## Guidelines for agents

- The literate document is the **source of truth**. Never edit generated
  files in `gen/` directly — changes will be overwritten on the next run.
- Use the Markdown/AsciiDoc structure to explain *why* the code is
  structured as it is. Chunk names should read as intent, not mechanics.
- When adding a new output file, declare it as a `<[@file ...]>=` chunk
  in the appropriate literate source, then reference named sub-chunks to
  keep each chunk short and focused.
- `azadi` writes output files only when content changes (content-based
  diffing). Rebuilds that produce identical output leave files untouched,
  keeping build system timestamps stable.
- Use `--formatter rs=rustfmt` (or the equivalent for the target language)
  to keep generated code formatted without manual intervention.
- `--dump-expanded` (stderr) shows the macro-expanded intermediate text —
  the first thing to check when a chunk is missing or expands unexpectedly.
