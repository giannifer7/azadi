---
name: Weaveback Literate Programming
description: Guidelines and patterns for working in codebases that use the weaveback literate programming toolchain.
---

# Weaveback Literate Programming

Weaveback is a literate programming toolchain. The source of truth is a set of
annotated documents (`.wvb`, `.md`, `.adoc`, etc.) containing prose, macro calls,
and named noweb chunks. Generated Rust/Python/TOML/etc. files are products of
the literate sources, not the primary edit target unless you intentionally use
apply-back.

## CLI

Use the focused tools:

```bash
wb-tangle                     # run the project configuration from weaveback.toml
wb-tangle path/to/file.md     # tangle a specific driver
wb-tangle --dir src --ext md --gen gen
wb-query trace gen/out.rs 42 1
wb-query cargo clippy --all-targets -- -D warnings
wb-tangle apply-back --dry-run
wb-serve --watch
wb-mcp
```

Responsibilities:

| Tool | Responsibility |
| --- | --- |
| `wb-tangle` | Build-side operations: macro expansion, tangling, source-map DB writes, and `apply-back`. |
| `wb-query` | Read/query-side operations: trace, where, attribute, cargo annotation, search, graph, coverage, lint, tags, LSP. |
| `wb-serve` | Local docs server. |
| `wb-mcp` | MCP server for editor/agent integrations. |
| `weaveback-macro` | Advanced standalone macro-expansion/debugging tool. Not the normal project entrypoint. |

## Two-Pass Documents

Newer sources may use `.wvb` plus markup prelude macros. The first pass expands
markup-neutral `¤...` macros into concrete `.md` and/or `.adoc` documents. The
second pass tangles the expanded document into generated source files.

Do not edit expanded output just because it exists. Prefer editing the `.wvb`
canonical source when present, then run `wb-tangle`.

## Chunk Syntax

The standard delimiters are `<[`, `]>`, and `@`.

Use the host-language comment marker before delimiters when the chunk body is
source code. Use `//` for Rust/C-like languages, `#` for Python/shell/TOML, and
no marker for plain weaveback text.

```rust
// <[@file src/lib.rs]>=
// <[module declarations]>
// @

// <[module declarations]>=
pub mod utils;
// @
```

```text
<[@file config/default.toml]>=
[server]
port = <[server-port]>
@

<[server-port]>=
8080
@
```

Rules:

| Form | Meaning |
| --- | --- |
| `<[@file path]>=` | Declares an output file chunk. |
| `<[name]>=` | Declares a named chunk. |
| `<[name]>` | Expands a named chunk inline, preserving indentation. |
| `// @`, `# @`, `@` | Ends the current chunk. The comment marker must match the opener style. |

When adding a generated file, declare one `@file` chunk and compose it from
small named chunks. Keep chunk names semantic, not mechanical.

## Macro Language

The macro language is strict, eager, string-valued, and intentionally small.
The default sigil is `%`; a run may choose any single UTF-8 scalar as sigil
with `--sigil`.

Core forms:

```text
%def(name, params..., body)     constant macro binding in the current frame
%redef(name, params..., body)   rebindable macro binding in the current frame
%(name)                         variable reference in the current frame
%set(name, value)               set variable in the current frame
%env(NAME)                      read environment, only when enabled
%if(cond, then, else)           conditional; only the selected branch expands
%match(v, default, r, x...)     regex dispatch; only the selected branch expands
%include(path)                  include and emit another file
%import(path)                   include another file for definitions, discard output
%pydef(name, params..., body)   Python/monty escape hatch
```

### Definitions

`%def` creates a constant binding in the current frame. It errors if the name
already exists in that frame as either constant or rebindable, or if the name
is a builtin.

`%redef` creates or replaces a rebindable binding in the current frame. It may
replace an existing rebindable binding, but it may not replace a `%def`
constant or a builtin.

Use `%redef` for deliberate X-macro or multi-pass rebinding patterns. Use
`%def` everywhere else.

### Blocks

Use quoted argument blocks for macro-active bodies:

```text
%def(greet, name, %{Hello, %(name)!%})
```

Use verbatim blocks for literal bodies:

```text
%pydef(greet, name, %[ "hello " + name %])
```

Block forms:

| Form | Meaning |
| --- | --- |
| `%{ ... %}` / `%tag{ ... %tag}` | One argument, still macro-active, nestable. |
| `%[ ... %]` / `%tag[ ... %tag]` | Opaque verbatim block, no macro parsing inside, nestable. |
| `%/* ... %*/` | Block comment, nestable. |

Both quoted and verbatim blocks preserve initial whitespace. A bare blank
argument between commas is an empty argument; `%{%}` is the explicit empty
argument form.

For `%match`, wrap non-trivial regexes in verbatim blocks and wrap branches
that read captures in quoted blocks:

```text
%match(issue-42, unknown,
       %[^(?P<prefix>[a-z]+)-(\d+)$%],
       %{kind=%(prefix), number=%(match_2)%})
```

Capture variables are branch-local: `%(match_0)` is the whole match,
`%(match_1)` etc. are numbered captures, and named captures are available by
their capture name.

### Strict Evaluation

Arguments are fully expanded in the caller scope before the callee frame is
pushed. Values do not become lazy expressions.

Important invariants:

| Rule | Consequence |
| --- | --- |
| Missing variables are errors. | Typos do not silently become empty strings. |
| Missing parameters are errors. | Calls must pass required values explicitly. |
| Extra positional arguments are errors. | Arity mistakes are caught. |
| Unknown named arguments are errors. | Misspelled parameter names are caught. |
| `%set(...)` is forbidden in argument position. | Arguments are values, not assignment sites. |
| `%if` and `%match` are lazy in their branches. | Non-selected branches can contain otherwise-invalid calls without firing. |
| Variables are looked up only in the current top frame. | Outer `%set` bindings do not leak into macro bodies. Pass values as parameters. |
| Macro lookup walks outward through frames. | Inner macro definitions shadow outer ones. |

`%set` belongs to the variable namespace. `%def`, `%redef`, `%alias`, and
`%pydef` belong to the macro namespace. Variables and macros are distinct.

## Markup Prelude Macros

Project-local markup prelude macros use `¤` by convention. They are intended to
hide Markdown/AsciiDoc differences and emit standard file/chunk structure.

Prefer prelude macros for common shapes when they exist, for example:

```text
¤rust_file(path/to/file.rs, ¤[
// <[module prelude]>
// <[module body]>
¤])

¤rust_chunk(module body, ¤[
pub fn helper() {}
¤])
```

Do not hand-normalize generated `.md`/`.adoc` if the `.wvb` source can express
the intent once through prelude macros.

## Source Tracing

`wb-tangle` records source maps in `weaveback.db`. Use `wb-query trace` to map
generated code back to the literate source.

```bash
wb-query trace gen/out.rs 42
wb-query trace gen/out.rs 42 10
```

Line and column numbers are 1-indexed character positions.

Important trace fields:

| Field | Meaning |
| --- | --- |
| `src_file` | Literate source file to edit. |
| `src_line` | 1-indexed source line. |
| `src_col` | 1-indexed source column when available. |
| `kind` | `Literal`, `MacroBody`, `MacroArg`, `VarBinding`, or `Computed`. |
| `macro_name` | Macro responsible for `MacroBody`/`MacroArg`. |
| `param_name` | Parameter responsible for `MacroArg`. |
| `var_name` | Variable responsible for `VarBinding`. |
| `def_locations` | `%def`/`%redef`/`%pydef` definition locations. |
| `set_locations` | `%set` assignment locations. |
| `chunk` | Noweb chunk containing the generated line. |

Reading traces:

| Kind | Edit target |
| --- | --- |
| `Literal` | Edit `src_file` at `src_line`. |
| `MacroBody` | Edit the macro definition reported in `def_locations`. |
| `MacroArg` | Edit the argument at the call site. |
| `VarBinding` | Edit the `%set` assignment if the mapping is unambiguous. |
| `Computed` | Inspect manually; generated by nontrivial computation. |

## Apply-Back

`wb-tangle apply-back` is a reconciliation workflow for changes already made in
generated files. It diffs generated outputs against the stored baseline,
traces changed lines back to literate source, patches candidates, and
oracle-verifies the resulting output before writing.

```bash
wb-tangle apply-back
wb-tangle apply-back --dry-run
```

Use apply-back when generated files were edited directly by an IDE, language
tool, or manual intervention. In a normal agent workflow, prefer tracing to the
literate source first and editing the source directly.

## MCP Agent Workflow

Start the server with:

```bash
wb-mcp
```

Recommended order for agents:

1. Use `weaveback_chunk_context` when you know the chunk name and need prose,
   dependencies, reverse dependencies, and recent git history.
2. Use `weaveback_trace` when a generated-file line is the starting point.
3. Read the surrounding literate source before editing; prose is part of the
   source of truth.
4. Use `weaveback_apply_fix` for targeted oracle-verified source edits when
   available.
5. Use `weaveback_apply_back` only when generated files have already been
   edited and must be reconciled.
6. Retangle and run the relevant tests/lints.

## Agent Rules

* Edit canonical literate sources (`.wvb`, `.md`, `.adoc`) whenever possible.
* If a generated file says not to edit it, find the chunk or source document
  that generated it.
* After changing literate source, run `wb-tangle` or the project recipe that
  wraps it before testing generated code.
* Do not use `include!` as a shortcut for splitting Rust code. It destroys the
  provenance boundaries Weaveback is designed to preserve.
* Keep chunks small enough to review locally, but do not fragment files into
  meaningless one-line chunks.
* Update prose together with code. A correct generated file with stale
  literate explanation is incomplete.
* Run `git diff --check` before handoff; whitespace drift in literate sources
  propagates into expanded documents.
