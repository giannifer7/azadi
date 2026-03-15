---
name: planned feature — macro-level back-propagation and token tracing
description: Plan for tracing output lines back through macro expansion to the original literate source, with on-demand token annotation
type: project
---

## Context

`project_backpropagate.md` describes the noweb-level apply-back tool.
That tool traces `gen/` output lines → literate source chunks via `noweb_map`.

But the `noweb_map.src_line` points to a line in the **macro-expanded** intermediate
text, not in the original `.md` file.  A change at output line L came from:

```
gen/out.rs : line L
    ↓  noweb_map
driver.md (expanded) : line E   ← this is what noweb_map currently tracks
    ↓  macro_map  (NEW)
driver.md (original) : line S, col C, via %def or literal
```

Without the second level, the apply-back tool can only say "the change came from
expanded line E of driver.md" — but that line may be the result of a macro call
that looks nothing like the expanded text.  The user needs to know: was this literal
text, a macro body, or an argument at a call site?

---

## Two-level source map

### Level 1: `noweb_map` (already built)

```
(out_file, out_line_0indexed)  →  NowebMapEntry {
    src_file:   String,   // driver file path (the filename passed to clip.read)
    chunk_name: String,
    src_line:   u32,      // 0-indexed line in the expanded text of src_file
    indent:     String,
}
```

### Level 2: `macro_map` (new redb table)

```
("{driver_file}\x00{expanded_line:010}")  →  MacroMapEntry {
    src_file:  String,    // original .md file that contributed this text
    src_line:  u32,       // 0-indexed line in the original file
    src_col:   u32,       // 0-indexed column (byte offset into the line)
    kind:      SpanKind,
}

enum SpanKind {
    Literal,
    // Text came from a %def / %rhaidef body definition
    MacroBody {
        macro_name: String,
        def_file:   String,
        def_line:   u32,
    },
    // Text came from an argument value at a macro call site
    MacroArg {
        macro_name: String,
        param_name: String,
        call_file:  String,
        call_line:  u32,
        call_col:   u32,
    },
}
```

Both tables live in `_azadi_work/azadi.db` alongside `gen_baselines`,
`noweb_map`, and `src_snapshots`.

---

## On-demand tracing (not always-on)

Token annotation through a full macro expansion is non-trivial overhead.
The plan discussed: **skip it during normal `azadi` runs; compute it only when
`azadi backprop` needs it**.

Two options for on-demand computation:

### Option A — Re-run in tracing mode (preferred)

When `azadi backprop` is invoked:
1. Read the list of driver files from the db (or accept them as arguments)
2. Re-run `azadi-macros` in **tracing mode** on each driver
3. Populate `macro_map` in the db
4. Proceed with apply-back using both levels of the map

The original `src_snapshots` provide the merge base for step 4 even if the user
has since edited the source.

### Option B — Always populate (simpler code, measure overhead first)

Add `macro_map` population to the normal `azadi` run.  If the overhead turns out
to be acceptable (likely for typical literate files), remove the on-demand path.

---

## `EvalOutput` trait — generic output accumulator

The evaluator currently renders directly to `Vec<u8>` / `String`.
To support both fast and tracing modes without duplicating the evaluator, introduce
a generic output accumulator:

```rust
pub trait EvalOutput {
    /// Append `text` that originated at `span` in the source.
    fn push_str(&mut self, text: &str, span: SourceSpan);
    /// Finalise and return the rendered bytes.
    fn finish(self) -> Vec<u8>;
}

pub struct SourceSpan {
    pub file_idx: usize,   // index into evaluator's file_names vec
    pub line:     u32,
    pub col:      u32,
    pub kind:     SpanKind,
}

// Fast path — ignores span info
pub struct PlainOutput { pub buf: Vec<u8> }

// Tracing path — records one SpanEntry per push_str call
pub struct TracingOutput {
    pub buf:   Vec<u8>,
    pub spans: Vec<SpanEntry>,   // sorted by out_offset
}

pub struct SpanEntry {
    pub out_offset: u32,   // byte offset in buf where this span starts
    pub out_len:    u32,   // length in bytes
    pub span:       SourceSpan,
}
```

The evaluator becomes generic: `fn eval<O: EvalOutput>(&mut self, node: &Node, out: &mut O)`.

Normal `azadi` run: instantiate `PlainOutput`.
Tracing run: instantiate `TracingOutput`, then convert `spans` into per-line
`MacroMapEntry` records and write to the db.

---

## Token location tracking in the lexer/parser

Currently tokens carry no location.  Changes needed in `azadi-macros`:

1. **Lexer**: add `(file_idx, line, col)` to each `Token` (or a parallel `TokenSpan` vec)
2. **Parser**: propagate spans from tokens into AST nodes
3. **Evaluator**:
   - When outputting literal text from a `Text` node: `out.push_str(text, span_of_node)`
   - When substituting a variable/parameter: `out.push_str(value, call_site_span)` with `kind = MacroArg { ... }`
   - When expanding a `%def` body: each literal piece in the body carries the `def_line` span; argument substitutions carry the call-site span
   - When outputting `%if`/`%case`/`%for` results: propagate spans from the branch taken

This is the most invasive change.  The lexer already has line-counting logic for
error messages; adding `col` and threading spans through the AST is incremental work.

---

## What back-propagation looks like with full two-level trace

Given a change at `gen/out.rs` output line L:

```
1.  entry1 = noweb_map[out.rs, L]
    → src_file="driver.md", chunk="@file out.rs", src_line=42, indent="    "

2.  entry2 = macro_map["driver.md", 42]
    → src_file="macros.md", src_line=7, kind=MacroBody { macro_name="tag", def_line=5 }

3.  original_text = src_snapshots["macros.md"][line 7]
4.  current_text  = read("macros.md")[line 7]

5.  If current_text == original_text:
        patch macros.md line 7 with the new content (minus indent)
    elif current_text == desired:
        idempotent — skip
    else:
        3-way conflict → present to user:
          base    = original_text      (from snapshot)
          theirs  = current_text       (what's in macros.md now)
          mine    = desired_text       (from gen/ edit)
```

For `kind = MacroArg { macro_name="tag", param_name="value", call_file, call_line }`:
- The right place to patch is the *argument at the call site* in `call_file:call_line`,
  not the macro body.
- Present to user: "This value came from the argument `value` in the call to `%tag`
  at macros.md:23.  Apply there? [y/N]"

---

## The "experimental edit" full workflow

1. User runs `azadi` → `gen/foo.rs` generated, `src_snapshots` written
2. User edits `gen/foo.rs` to try something out
3. User runs `azadi` → `ModifiedExternally` abort
4. User runs `azadi backprop --trace`:
   - Tool re-runs macro expansion in tracing mode → populates `macro_map`
   - Computes diff of `gen/foo.rs` vs `gen_baselines["foo.rs"]`
   - For each changed line, follows two-level map to original source location
   - Reports: "line 42 of gen/foo.rs → chunk `@file foo.rs` in driver.md (expanded
     line 15) → macro body of `%tag` defined at macros.md:8"
   - Prompts for confirmation; applies patch to macros.md
5. `azadi backprop` resets the baseline for foo.rs so next run proceeds
6. User runs `azadi` → regenerates cleanly from the patched macros.md

---

## Scope and explicit non-goals

**In scope:**
- Literal text, macro body text, argument-value text — all three `SpanKind`s
- `%def` and `%rhaidef` (Rhai body: source span points to the `%rhaidef` call line)
- `%include` / `%import`: spans cross file boundaries — the file_idx in `SourceSpan`
  handles this naturally
- On-demand mode (re-run) to avoid adding overhead to normal builds

**Deferred / out of scope:**
- Tracing through Rhai *code execution* (i.e., `%rhaidef` body at expression level)
  — the span would point to the `%rhaidef` definition line, not sub-expressions
- Column-level precision in the apply-back (line-level is sufficient for the
  `azadi backprop` use case)
- IDE integration (showing source locations inline) — that comes after the CLI tool works

---

## Implementation order (suggested)

1. Add `(line, col)` to `Token` in the lexer (non-breaking; col can be 0 initially)
2. Propagate line/col through the AST nodes that matter: `Text`, `MacroCall`,
   `VarRef`, `MacroBody`
3. Introduce `EvalOutput` trait + `PlainOutput` (should be zero-overhead for the
   normal path)
4. Wire `TracingOutput` into the evaluator; test with small examples
5. Add `macro_map` table to `AzadiDb`; write entries from `TracingOutput.spans`
6. Integrate into `azadi backprop --trace` (extend `project_backpropagate.md` plan)
7. Measure overhead; decide whether to always-on or keep on-demand
