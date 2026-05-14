---
title: |-
  CLI reference
---
# CLI reference
The public CLI is composed of four focused tools.

## Command map

<table>
  <tr><th>Tool</th><th>Responsibility</th></tr>
  <tr><td>`wb-tangle`</td><td>Build-side operations: single-pass tangling, multi-pass `weaveback.toml`<br>
runs, and `apply-back` reconciliation from `gen/` to literate source.</td></tr>
  <tr><td>`wb-query`</td><td>Read/query-side operations: `where`, `trace`, `attribute`, `impact`,<br>
`graph`, `search`, `lint`, `coverage`, wrapped `cargo`, `tags`, plus<br>
semantic `lsp` lookups and `tag` metadata generation.</td></tr>
  <tr><td>`wb-serve`</td><td>Local docs server with live reload, inline editing, and AI-assisted views.</td></tr>
  <tr><td>`wb-mcp`</td><td>MCP server for editor and agent integrations.</td></tr>
</table>

## Common workflows

### Build and regenerate outputs

```bash
wb-tangle --dir src --ext adoc --include . --gen src
wb-tangle
```

### Reconcile edits made in `gen/`

```bash
wb-tangle apply-back
wb-tangle --gen path/to/gen apply-back --dry-run
```

### Trace generated code back to literate source

```bash
wb-query where gen/out.rs 120
wb-query trace gen/out.rs 120 8 --include .
wb-query attribute gen/out.rs:120:8 --include .
```

### Coverage and diagnostics

```bash
wb-query cargo clippy --all-targets -- -D warnings
wb-query coverage --summary lcov.info
```

### Docs server and MCP

```bash
wb-serve --watch
wb-mcp
```

## Detailed references

For full flag-by-flag details, read the tool-specific pages:

* [`wb-tangle`](../crates/wb-tangle/src/main.md)
* [`wb-query`](../crates/wb-query/src/main.md)
* [`wb-serve`](../crates/wb-serve/src/main.md)
* [`wb-mcp`](../crates/wb-mcp/src/main.md)
* [`cli-spec/macros.adoc`](../cli-spec/macros.md) for the shared option
  families used by the generated CLI code
