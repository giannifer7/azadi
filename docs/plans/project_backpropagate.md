---
name: planned feature — back-propagate edits to literate source
description: Detailed plan for tracing edits in generated files back to their origin in the literate source and applying them there
type: project
---

## The problem

When azadi detects `ModifiedExternally` (the user edited a file in `gen/` directly),
the run aborts. This is correct protection, but it's not helpful: the user now has a
useful edit in the generated file and no easy way to get it back into the literate
source where it belongs.

The goal is to build an **apply-back** tool (separate binary or `azadi apply-back`)
that:
1. Reads the diff between the generated file and its baseline in `gen_baselines`
2. Uses `noweb_map` to trace each changed output line back to its chunk in the
   literate source
3. Uses `src_snapshots` to see what the literate source looked like at generation time
4. Applies the changes to the current literate source, with care and prompting on
   ambiguity
5. Clears the baseline so the next `azadi` run can proceed

---

## Infrastructure already in place (as of v0.1.1)

### `_azadi_work/azadi.db` tables

**`gen_baselines`** — `rel_out_path → bytes`
The file content as azadi last wrote it.  Diff this against the current `gen/` file
to find what the user changed.

**`noweb_map`** — `"{out_file}\x00{out_line:010}" → postcard(NowebMapEntry)`
One entry per output line.  `NowebMapEntry`:
```rust
pub struct NowebMapEntry {
    pub src_file:   String,   // literate source file path
    pub chunk_name: String,   // chunk that produced this line
    pub src_line:   u32,      // 0-indexed line in src_file at generation time
    pub indent:     String,   // indentation prepended during expansion
}
```

**`src_snapshots`** — `src_path → bytes`
Snapshot of every input file at the time of the last run.  Lets us locate the
exact text in the source even if the source was since changed.

---

## Algorithm sketch

```
for each file F in gen/ where current_content != gen_baselines[F]:

    diff = line_diff(gen_baselines[F], current_content(F))

    for each changed hunk in diff:
        for each modified output line L:
            entry = noweb_map[F, L]          # NowebMapEntry
            snapshot_line = src_snapshots[entry.src_file][entry.src_line]

            # Strip the indent azadi added, to get back the raw chunk text
            original_chunk_line = strip_prefix(snapshot_line, entry.indent)

            new_chunk_line = strip_prefix(modified_output_line, entry.indent)

            # Locate the same line in the *current* literate source
            current_src = read(entry.src_file)
            candidate_line = current_src[entry.src_line]

            if candidate_line == original_chunk_line:
                # Clean match — apply directly
                patch current_src at entry.src_line with new_chunk_line
            elif candidate_line == new_chunk_line:
                # Already applied (idempotent) — skip
            else:
                # Conflict: the literate source was ALSO changed since last gen.
                # Present a 3-way diff to the user and ask what to do.
                ask_user(src_file, src_line, original_chunk_line,
                         candidate_line, new_chunk_line)
```

After all patches are applied:
- Write back the modified literate source files
- Delete (or reset) `gen_baselines[F]` so the next `azadi` run doesn't see it as
  modified externally
- Optionally re-run `azadi` automatically

---

## Complications to handle carefully

### Lines that have no map entry
Slot reference lines (chunk reference lines like `# <<other-chunk>>`) don't produce
output lines directly — they're replaced by the expansion of the referenced chunk.
Every output line **does** have a map entry because `expand_with_depth_impl` only
stores entries for plain (non-slot) lines.  So coverage is complete for actual content.

### Indentation
`NowebMapEntry.indent` records the indentation azadi prepended.  To recover the
original chunk text, strip that prefix from both the baseline output line and the
user's modified output line before applying the change to the source.

### Deleted lines
If the user deleted a line in the generated file, the corresponding source chunk
line should also be deleted.  Deletion in literate source may leave a dangling chunk
body — warn the user.

### Added lines
If the user inserted new lines in the generated file, there is no `noweb_map` entry
for them.  Best approach:
- Identify the surrounding map entries to find the chunk and approximate location
- Insert the new line(s) into the chunk body at that location
- Prompt the user to confirm the insertion point (could be ambiguous near chunk
  boundaries or multi-definition chunks)

### Multi-definition chunks (accumulated chunks)
A single logical chunk can have multiple definitions that are concatenated.
`NowebMapEntry.src_line` points to the right definition, so patching is unambiguous
as long as we use `src_file + src_line` together.

### `@reversed` chunks
Definitions are iterated in reverse, so line numbers are still correct (they point
into the original source), but the order of the applied patches must account for
reversal.  Track this via the natural ordering of `(out_file, out_line)` in the
noweb_map.

### Macro-expanded content
The `src_file` in `NowebMapEntry` is the *intermediate* file fed to azadi-noweb
(i.e., the output of azadi-macros, which is a string, not a real file).  For the
combined `azadi` binary the source files passed to `clip.read(text, filename)` use
the original driver path as the filename, so `src_file` points to the real literate
source.  For `azadi-noweb` run standalone, `src_file` is whatever was passed to
`--file` or stdin.

Macro-level back-propagation (tracing through `%def` expansion) is **not** in scope
for this feature and is explicitly deferred.

---

## Suggested implementation approach

1. **New binary `azadi-backprop`** (or subcommand `azadi backprop`)
   - Takes `--db _azadi_work/azadi.db`, `--gen gen/` as arguments (with sensible
     defaults matching `azadi`'s defaults)
   - Reads the db, diffs each baseline, applies patches

2. **Interactive mode** (default)
   - For unambiguous changes: print what it's about to do and apply after a short
     confirmation window (or `--yes` to skip)
   - For conflicts: show a 3-way diff (original snapshot / current source / desired
     target) and ask the user to choose or edit

3. **Dry-run mode** `--dry-run`
   - Print all patches without writing anything

4. **Dependencies needed**
   - `similar` crate (pure Rust, line diff) — already good fit
   - `redb` (already in workspace)
   - `postcard` (already in workspace)
   - `console` or `dialoguer` for interactive prompts (optional — could use plain
     stdin/stdout)

---

## Why this is valuable

- Allows "experiment in the generated file, then commit to the source" workflow
- Closes the loop on the always-on modification protection — protection is no longer
  just a wall, it becomes a guide back to the right place
- Together with `src_snapshots`, enables a soft "undo": even if the user modified
  both gen/ and the literate source, the snapshot gives a 3-way merge base

---

## What NOT to do (scope boundaries)

- Do not attempt to back-propagate through macro expansion (%def/%rhaidef bodies).
  Only noweb-level (chunk) back-propagation is in scope.
- Do not automatically reformat the literate source — only change the lines that
  were actually modified in the generated file.
- Do not silently overwrite conflicts — always ask.
