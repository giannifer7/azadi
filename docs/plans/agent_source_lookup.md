# Agent source lookup — read-only map query tool

## Motivation

When an AI agent is fixing compilation errors in a literate programming project,
the compiler points at a line in `gen/` but the fix must be made in the literate
source.  Without tooling the agent must search through chunks heuristically.
With the two-level source map already in `azadi.db`, the lookup is exact and O(1).

This use case does **not** require `azadi backprop` (no external edit to apply
back).  It needs only a lightweight **read-only query tool** separate from the
apply-back flow.

---

## The agent workflow

```
1. Compiler: gen/foo.rs:42:7: error[E0308]: mismatched types

2. Agent calls: azadi where gen/foo.rs 42
   → noweb_map lookup:
     src_file="driver.md", chunk="impl Foo", src_line=15, indent="    "

3. Agent calls: azadi where-macro driver.md 15   (if macro_map populated)
   → macro_map lookup:
     src_file="macros.md", pos=1234, kind=MacroBody{macro_name="impl_foo"}

4. Agent reads macros.md at that position, understands context, edits the chunk

5. Agent runs: azadi && cargo build
   → verifies fix compiles
```

Step 3 is optional — if the error is in literal chunk text rather than a macro
expansion, step 2 is sufficient.

---

## Proposed CLI interface

```sh
# Noweb level: which chunk produced output line L of file F?
azadi where <out-file> <line>

# Macro level: which source location produced expanded line L of driver file F?
azadi where-macro <driver-file> <expanded-line>

# Full chain in one call (combines both levels if macro_map is populated):
azadi trace <out-file> <line>
```

Output (machine-readable, JSON or plain):

```json
{
  "out_file":      "gen/foo.rs",
  "out_line":      42,
  "chunk":         "impl Foo",
  "expanded_file": "driver.md",
  "expanded_line": 15,
  "indent":        "    ",
  "src_file":      "macros.md",
  "src_pos":       1234,
  "src_line":      8,
  "src_col":       4,
  "kind":          "MacroBody",
  "macro_name":    "impl_foo"
}
```

`--json` flag for structured output; plain text by default for human reading.

---

## Why this is more useful than grep for agents

- **Exact**: no false positives from chunk names that appear in multiple places
- **Handles indentation**: the `indent` field tells the agent how much whitespace
  was added during expansion — it can strip it to compare with the source
- **Handles macro expansion**: a line that looks like `let x = foo(bar, baz);` in
  `gen/` may have come from `%(impl_call)(x, foo, bar, baz)` in the source —
  grep on the output text would never find the definition
- **Works across file boundaries**: `%include` chains are transparent; `src_file`
  always points to the actual file the agent should edit
- **No heuristics**: the agent doesn't need to understand azadi syntax at all to
  use this tool

---

## Relationship to azadi backprop

`azadi where` / `azadi trace` are the **read path** of the same infrastructure.
`azadi backprop` is the **write path** (applies edits back to the source).

They share the same db tables and can share most of the lookup code.
Implementation order: build `azadi where` first (simpler, no conflict resolution
needed), then extend to `azadi backprop`.

---

## MCP server variant (for IDE/agent integration)

For agents running inside an IDE or via the Model Context Protocol, the same
lookup can be exposed as an MCP tool:

```
tool: azadi_trace
input: { out_file: string, out_line: integer }
output: SourceLocation (same fields as JSON above)
```

This lets the agent call the lookup as a function rather than shelling out.
The MCP server would open `_azadi_work/azadi.db` read-only and serve queries.
This is a thin wrapper over the same redb lookups — very little additional code.

---

## Notes for implementation

- `azadi where` reads `noweb_map` only — no need for `macro_map` to be populated
- `azadi trace` falls back gracefully if `macro_map` is empty (macro tracing not
  yet computed) — just returns the noweb-level result
- The db path should default to `_azadi_work/azadi.db` with `--db` override
- Line numbers in the CLI are **1-indexed** (matching compiler output);
  internally `noweb_map` keys are 0-indexed — subtract 1 before lookup
