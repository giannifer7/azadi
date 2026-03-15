# Azadi

Azadi is a literate-programming toolchain. Write your source code inside
Markdown (or any text file), expand macros, extract named chunks, and let
the tool write the real files.

The recommended entry point is the combined `azadi` command:

```bash
azadi source.md --gen src
```

Under the hood it runs two passes in sequence:

1. **azadi-macros** — expands `%macro(...)` calls in the source document
2. **azadi-noweb** — extracts `<[@file ...]>` chunks and writes them to disk

Both passes run in-process; no intermediate files or subprocesses are needed.
The individual `azadi-macros` and `azadi-noweb` binaries are also available for
advanced or step-by-step use.

---

## Installation

### Arch Linux

```bash
paru -S azadi-bin   # or: yay -S azadi-bin
```

### Nix

```bash
nix profile install github:giannifer7/azadi
```

Or in a flake input:

```nix
inputs.azadi.url = "github:giannifer7/azadi";
environment.systemPackages = [ inputs.azadi.packages.x86_64-linux.default ];
```

### Pre-built binaries (Linux, Windows)

Download from the [latest release](https://github.com/giannifer7/azadi/releases/latest):

| File | Platform | Notes |
|------|----------|-------|
| `azadi-x86_64-linux.tar.gz` | Linux x86_64 | glibc build — tarball with all three binaries |
| `azadi-glibc` / `azadi-macros-glibc` / `azadi-noweb-glibc` | Linux x86_64 | glibc, individual binaries |
| `azadi-musl` / `azadi-macros-musl` / `azadi-noweb-musl` | Linux x86_64 | musl, individual binaries — see note below |
| `azadi-fedora` / `azadi-macros-fedora` / `azadi-noweb-fedora` | Fedora/RHEL | Fedora build |
| `*.deb` | Debian/Ubuntu | install with `sudo dpkg -i` |
| `*.rpm` | Fedora/RHEL | install with `sudo rpm -i` |
| `azadi.exe` / `azadi-macros.exe` / `azadi-noweb.exe` | Windows x86_64 | native build |
| `azadi-mingw64.exe` / ... | Windows x86_64 | MinGW cross-compiled build |

**musl vs glibc binaries**

The musl builds are fully statically linked — no shared library dependencies
at all, as confirmed by `ldd azadi-musl` reporting *statically linked*. This
makes them suitable for:

- Any Linux distro regardless of glibc version (old RHEL/CentOS, Alpine, etc.)
- Minimal or scratch containers
- Environments where you just want to `curl` and run without worrying about
  library compatibility

The glibc builds are dynamically linked against glibc and are the better
choice on standard Debian/Ubuntu/Fedora systems where glibc is already
present, as glibc's runtime is generally faster.

The `azadi-macros` and `azadi-noweb` binaries are available in the release for
advanced pipeline use, but `azadi` alone covers the typical workflow.

**Quick install (musl, no package manager):**

```bash
curl -sL https://github.com/giannifer7/azadi/releases/latest/download/azadi-musl \
     -o /usr/local/bin/azadi && chmod +x /usr/local/bin/azadi
```

---

## Quick start

```bash
cargo build --release
# binaries: target/release/azadi  target/release/azadi-macros  target/release/azadi-noweb
```

Run the built-in example:

```bash
cd examples/c_enum
azadi status.md --gen .
```

---

## `azadi` — combined tool

### Usage

```bash
# Process explicit files
azadi [OPTIONS] <INPUTS>...

# Auto-discover all driver files in a directory
azadi [OPTIONS] --dir <DIR>
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--input-dir <PATH>` | `.` | Base directory prepended to every input path |
| `--special <CHAR>` | `%` | Macro invocation character |
| `--include <PATHS>` | `.` | Include search paths for `%include`/`%import` (colon-separated on Unix) |
| `--work-dir <PATH>` | `_azadi_work` | Work directory for backups and private noweb files |
| `--gen <PATH>` | `gen` | Base directory for generated output files |
| `--open-delim <STR>` | `<[` | Chunk-open delimiter |
| `--close-delim <STR>` | `]>` | Chunk-close delimiter |
| `--chunk-end <STR>` | `@` | End-of-chunk marker |
| `--comment-markers <STR>` | `#,//` | Comment prefixes recognised before chunk delimiters (comma-separated) |
| `--formatter <EXT=CMD>` | | Run a formatter after writing a file (e.g. `rs=rustfmt`), repeatable |
| `--trace` | off | Record source-map data in the work database for use by `azadi where` / `azadi trace` |
| `--dump-expanded` | off | Print macro-expanded text to stderr before noweb processes it |
| `--dir <DIR>` | | Auto-discover and process driver files under this directory (mutually exclusive with positional inputs) |
| `--ext <EXT>` | `md` | File extension to scan in `--dir` mode; repeatable for multiple extensions |
| `--depfile <PATH>` | | Write a Makefile depfile listing every source file read |
| `--stamp <PATH>` | | Touch this file on success (build-system stamp) |
| `--allow-env` | off | Enable the `%env(NAME)` builtin (disabled by default) |

### Examples

```bash
# Run from the project root; inputs live in docs/
azadi notes.md --input-dir docs --gen src --formatter rs=rustfmt

# Debug: see what the macro expander produced
azadi notes.md --gen src --dump-expanded 2>expanded.txt
```

`--input-dir` lets you run from the project root while keeping your literate
sources in a subdirectory. Every input path is joined with `--input-dir` before
reading.

`--dump-expanded` writes the macro-expanded intermediate text to stderr, one
section per input file delimited by `=== expanded: <path> ===` / `=== end: <path> ===`
headers, so you can inspect exactly what azadi-noweb receives. This is the
first thing to check when a chunk cannot be found or expands unexpectedly.

### Directory mode

`--dir <DIR>` scans a directory tree recursively for files matching
`--ext` (default: `md`), automatically identifies which are *drivers*
(top-level files) versus *fragments* (files referenced by a `%include()` in
another file), and processes each driver. No changes are needed when new files
are added to the tree.

```bash
# Default: scan for .md files
azadi --dir src --include . --gen src

# Scan for AsciiDoc files instead
azadi --dir src --ext adoc --include . --gen src

# Multiple extensions at once
azadi --dir src --ext md --ext adoc --include . --gen src
```

The driver/fragment distinction is determined by a *discovery pass*: every
file is evaluated with macro expansion to resolve `%include`/`%import` paths
(including computed ones), and any file referenced from another is marked as a
fragment and excluded from standalone processing.

### Build-system integration (`--depfile` / `--stamp`)

`--depfile <PATH>` writes a Makefile-format depfile after a successful run.
In directory mode the depfile lists **all** files found in the tree matching
`--ext` (so adding a new file triggers a rebuild even before it is processed).
In explicit-input mode it lists only the files actually read by the evaluator.

`--stamp <PATH>` writes an empty file on success, suitable as a build-system
output marker.

Together they let a single build rule replace an entire list of per-file rules:

```meson
# meson.build — one rule for all .adoc files; no edits needed when adding new ones
custom_target('gen-nim',
  output  : ['gen.stamp'],
  depfile : 'gen.d',
  command : [azadi, '--dir',    meson.current_source_dir() / 'src',
                    '--ext',    'adoc',
                    '--include', meson.current_source_dir(),
                    '--stamp',  '@OUTPUT0@',
                    '--depfile', '@DEPFILE@',
                    ...other flags...],
)
```

> **Note:** list only the stamp in `output`, never the `.d` file.
> Ninja consumes the depfile into its internal database (`.ninja_deps`)
> after the first run; if the `.d` file is also declared as an output,
> ninja sees it as permanently missing and reruns the target on every build.

### Source tracing (`azadi where` / `azadi trace`)

`azadi` can record a source map during every run and use it later to answer
the question *"where did this line in a generated file come from?"*

**Step 1 — record the source map:**

```bash
azadi status.md --gen . --trace
```

`--trace` stores two levels of provenance in `_azadi_work/azadi.db`:

- **noweb level** — which literate chunk and source line produced each output line
- **macro level** — which macro call (and which argument or body) generated each
  expanded line

**Step 2 — query:**

```bash
# noweb level only: chunk name + source file/line
azadi where <out_file> <line>

# full two-level trace: chunk + macro body/arg origin
azadi trace <out_file> <line>
```

Both commands print JSON to stdout and accept the same `--work-dir` / `--gen`
flags as the main command. `<out_file>` is the path of a generated file as you
see it on disk; `<line>` is 1-indexed.

**Example** (using the c_enum sample):

```bash
cd examples/c_enum
azadi status.md --gen . --trace

azadi where src/status.c 6
```

```json
{
  "chunk": "string_cases",
  "expanded_file": "./status.md",
  "expanded_line": 44,
  "indent": "",
  "out_file": "src/status.c",
  "out_line": 6
}
```

```bash
azadi trace src/status.c 6
```

```json
{
  "chunk": "string_cases",
  "expanded_file": "./status.md",
  "expanded_line": 44,
  "indent": "",
  "kind": "MacroBody",
  "macro_name": "enum_val",
  "out_file": "src/status.c",
  "out_line": 6,
  "src_col": 45,
  "src_file": "/path/to/examples/c_enum/status.md",
  "src_line": 31
}
```

The `trace` output adds `src_file`, `src_line`, `src_col`, `kind`, and
(for macro body/arg spans) `macro_name`, giving the exact location in the
literate source that was expanded to produce the output line.

`kind` is one of:

| Value | Meaning |
|-------|---------|
| `Literal` | Text copied verbatim from the source |
| `MacroBody` | Text produced by expanding a macro body |
| `MacroArg` | Text produced from a macro argument value |
| `VarBinding` | Text from a `%set` variable |
| `Computed` | Text produced by a Rhai script or other computed source |

### MCP server (`azadi mcp`)

`azadi mcp` starts a [Model Context Protocol](https://modelcontextprotocol.io/)
server over stdin/stdout. It exposes a single tool — `azadi_trace` — that
IDE extensions and AI agents can call to trace generated-file locations back to
their literate source.

```bash
azadi --work-dir _azadi_work --gen . mcp
```

The server reads JSON-RPC 2.0 messages from stdin (one per line) and writes
responses to stdout. It implements the MCP 2024-11-05 protocol and handles
`initialize`, `notifications/initialized`, `tools/list`, and `tools/call`.

**Tool: `azadi_trace`**

```json
{
  "name": "azadi_trace",
  "inputSchema": {
    "type": "object",
    "properties": {
      "out_file": { "type": "string" },
      "out_line": { "type": "integer" }
    },
    "required": ["out_file", "out_line"]
  }
}
```

Returns the same JSON as `azadi trace` above, encoded as a text content item.
Returns an error if no mapping is found or if the database has not been
populated yet (run `azadi ... --trace` first).

**Claude Desktop configuration example:**

```json
{
  "mcpServers": {
    "azadi": {
      "command": "azadi",
      "args": ["--work-dir", "_azadi_work", "--gen", ".", "mcp"]
    }
  }
}
```

---

## azadi-macros

A macro expander. Reads source files, evaluates `%macro(...)` calls, and writes
the result to `--output`. Supports both explicit file lists and directory-mode
auto-discovery (same driver/fragment logic as the combined `azadi` tool).

### Usage

```bash
# Process explicit files
azadi-macros [OPTIONS] <INPUTS>...

# Auto-discover driver files in a directory
azadi-macros [OPTIONS] --dir <DIR>
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--output <PATH>` | `-` | Output file (`-` for stdout) |
| `--special <CHAR>` | `%` | Macro invocation character |
| `--work-dir <PATH>` | `_azadi_work` | Directory for backup / intermediate files |
| `--include <PATHS>` | `.` | Include search paths (separated by `--pathsep`) |
| `--pathsep <STR>` | `:` / `;` | Path separator (platform default) |
| `--input-dir <PATH>` | `.` | Base directory prepended to each input path |
| `--allow-env` | off | Enable the `%env(NAME)` builtin (disabled by default) |
| `--dir <DIR>` | | Auto-discover driver files under this directory (mutually exclusive with positional inputs) |
| `--ext <EXT>` | `md` | File extension to scan in `--dir` mode; repeatable |

### Syntax

#### Argument whitespace

Leading whitespace in every argument is stripped. This lets you align
multi-argument calls without spaces leaking into the expanded text:

```
%def(tag, name, value, %{<%(name)>%(value)</%(name)>%})
%tag( div,
      Hello world)
```
Output: `<div>Hello world</div>`

To include a literal leading space use a block:

```
%tag(%{ div%}, %{ Hello world%})
```
Output: `< div> Hello world</ div>`

#### Named arguments

Any argument may be given a name with `identifier = value` syntax. The named
argument is bound to the variable whose name matches the identifier, regardless
of position:

```
%def(greet, name, msg, %{Hello, %(name)! %(msg)%})
%greet(name = Alice, msg = %{Good morning%})
```
Output: `Hello, Alice! Good morning`

#### Comments

| Syntax | Scope | Style |
|--------|-------|-------|
| `%# …` | to end of line | shell / Python |
| `%// …` | to end of line | C++ / Rust |
| `%-- …` | to end of line | Lua / SQL |
| `%/* … %*/` | block, nestable | C |

Comments are discarded and produce no output. They can appear at the top level,
inside macro bodies, or inside argument lists:

```
%def(greet,
     %# formal parameters:
     name,   %// the person's name
     msg,    %/* optional message %*/
     Hello %(name)! %(msg))
```

#### Multi-line blocks and tags

`%{ … %}` delimits a block that may span multiple lines and may contain
commas and parentheses without triggering argument splitting.

An optional tag identifier makes matching pairs easier to spot — in an editor
and on a printed page:

```
%def(page, title, body, %page{
<!DOCTYPE html>
<html><head><title>%(title)</title></head>
<body>%(body)</body></html>
%page})
```

The syntax is `%tag_name{ … %tag_name}` where `tag_name` is any identifier.
Tags are purely documentary; they do not affect evaluation.

---

### Built-in macros

#### `%def` — define a macro

```
%def(name, param1, param2, ..., body)
```

All arguments except the first (name) and last (body) are formal parameters.
Wrap the body in `%{ %}` — it is required whenever the body contains commas or
parentheses, and good style otherwise:

```
%def(greet, name, %{Hello, %(name)!%})
%greet(World)
```
Output: `Hello, World!`

```
%def(square, x, %{
%(x) * %(x)
%})
%square(7)
```
Output: `7 * 7`

A macro body may call other macros:

```
%def(bold,   text, %{**%(text)**%})
%def(titled, name, body, %{
### %(name)

%bold(%(body))
%})
%titled(Introduction, %{This text will be bold%})
```
Output:
```
### Introduction

**This text will be bold**
```

#### Calling conventions

These rules apply to all macro kinds (`%def`, `%rhaidef`, `%pydef`).
Arguments follow Python-style rules:

- **Positional** args fill declared params left-to-right; extra positional args beyond the declared param count are silently ignored.
- **Named** args (`param = value`) bind by name and can appear in any order; an unknown name is an error (helps catch typos).
- Positional args must come **before** named args; a positional after a named arg is an error.
- Binding the same param both positionally and by name is an error.
- Missing args default to empty string.

```
%def(http_endpoint, method, path, handler, %{
%(method) %(path) → %(handler)
%})

%http_endpoint(
    method  = GET,
    path    = /api/users,
    handler = list_users)
```
Output: `GET /api/users → list_users`

Mixing positional and named — positionals first:

```
%http_endpoint(GET, path = /api/users, handler = list_users)
```
Output: `GET /api/users → list_users`

---

A macro body may also contain `%def` calls that define new macros. Those macros
are local to the invocation scope; use `%export` to promote them to the
caller's scope:

```
%def(make_pair, a, b, %{
%def(first,  %{%(a)%})
%def(second, %{%(b)%})
%export(first)
%export(second)
%})
%make_pair(hello, world)
(%first(), %second())
```
Output: `(hello, world)`

#### `%set` — set a variable

```
%set(version, 1.0.0)
Version: %(version)
```
Output: `Version: 1.0.0`

#### `%if` — conditional

```
%if(condition, then-branch, else-branch)
```

Empty string is falsy; any non-empty string is truthy.

```
%set(debug, yes)
%if(%(debug), [DEBUG MODE], )
```
Output: `[DEBUG MODE]`

#### `%equal` — equality test

Returns the value if both arguments are equal, otherwise empty string.

```
%equal(%(mode), release)
```

#### `%include` / `%import`

`%include` expands the included file inline. `%import` expands it but discards
the output (useful for loading macro definitions).

```
%import(macros/common.txt)
%my_macro(arg)
```

#### `%env` — read an environment variable

```
%env(NAME)
```

Expands to the value of environment variable `NAME`, or empty string if unset.
Requires `--allow-env` to be passed on the command line; without it the macro
raises an error, preventing templates from silently reading secrets.

```
azadi --allow-env notes.md --gen src
```

```
Prefix: %env(MY_PREFIX)_
```

#### `%capitalize`, `%decapitalize`, `%to_snake_case`, `%to_camel_case`, `%to_pascal_case`, `%to_screaming_case`

Case-conversion helpers.

```
%to_snake_case(MyFancyName)
```
Output: `my_fancy_name`

#### `%rhaidef` — define a Rhai-scripted macro

```
%rhaidef(name, param1, param2, ..., body)
```

The body is a [Rhai](https://rhai.rs) script. It is evaluated at call time; its
return value (converted to string) becomes the macro output.

All visible azadi scope variables are injected into the Rhai script as string
variables. The body **must** be wrapped in `%{ ... %}` whenever it contains
parentheses, so that azadi does not misparse them as argument separators.

**Registered Rhai helpers:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `parse_int(s)` | `String → i64` | Parse string to integer (returns 0 on error) |
| `parse_float(s)` | `String → f64` | Parse string to float (returns 0.0 on error) |
| `to_hex(n)` | `i64 → String` | Format integer as `0xHEX` |

Rhai does **not** support Rust turbofish syntax — use the helpers above instead
of `.parse::<i64>()`.

**Examples:**

```
%rhaidef(double, x, %{(parse_int(x) * 2).to_string()%})
%double(21)
```
Output: `42`

```
%rhaidef(offset, base, size, %{
  let b = parse_int(base);
  let s = parse_int(size);
  (b + s).to_string()
%})
%offset(256, 64)
```
Output: `320`

```
%rhaidef(as_hex, n, %{to_hex(parse_int(n))%})
%as_hex(255)
```
Output: `0xFF`

```
%rhaidef(factorial, n, %{
  fn fact(k) { if k <= 1 { 1 } else { k * fact(k - 1) } }
  fact(parse_int(n)).to_string()
%})
%factorial(10)
```
Output: `3628800`

Outer azadi scope variables are also available inside the script:

```
%set(prefix, item_)
%rhaidef(prefixed, name, %{
  prefix + name
%})
%prefixed(count)
```
Output: `item_count`

#### `%rhaiset`, `%rhaiget`, `%rhaiexpr` — Rhai store

A typed, persistent store that survives across all `%rhaidef` calls in a run.
Every store entry is injected into every `%rhaidef` script as a Rhai variable
with its native type (integer, float, string, array, map, …). Store keys take
priority over same-named azadi scope variables.

After each script runs, any store key that the script modified is automatically
written back — no explicit write-back syntax needed.

```
%rhaiset(key, value)      — store a string or number (auto-parsed to i64/f64)
%rhaiget(key)             — read a store value as string
%rhaiexpr(key, rhai_expr) — store the result of a Rhai expression (for typed
                            literals: [], #{}, 3.14, true, …)
```

**Example — counter with auto write-back:**

```
%rhaiset(counter, 0)
%rhaidef(tick, %{
  counter += 1;
  counter.to_string()
%})
%tick()
%tick()
%tick()
```
Output:
```
1
2
3
```

**Example — accumulating an array:**

```
%rhaiexpr(items, [])
%rhaidef(push_item, x, %{
  items.push(x);
  items.len().to_string()
%})
%push_item(apple)
%push_item(banana)
%push_item(cherry)
```
Output:
```
1
2
3
```

**Example — building and traversing a tree:**

```
%rhaiexpr(root, #{})
%rhaidef(build_tree, %{
  root = #{
    name: "root",
    children: [
      #{ name: "a", children: [] },
      #{ name: "b", children: [] }
    ]
  };
  ""
%})
%rhaidef(child_count, %{
  root.children.len().to_string()
%})
%build_tree()
Tree has %child_count() children.
```
Output: `Tree has 2 children.`

#### `%pydef` — define a Python-scripted macro

```
%pydef(name, param1, param2, ..., body)
```

The body is a Python script evaluated by [monty](https://github.com/pydantic/monty),
pydantic's pure-Rust sandboxed Python interpreter. It is evaluated at call time; its
return value (converted to string) becomes the macro output. No Python runtime is
required — monty is compiled into the binary.

Only the explicitly declared parameters are available inside the script — they arrive
as plain Python string variables. Store entries from `%pyset` are also injected (see
below). There is no implicit full azadi scope injection. The script must return a
value: either an explicit `return` or the value of the last expression.

The body **must** be wrapped in `%{ ... %}` whenever it contains parentheses or commas.

**Note:** monty supports a subset of Python: arithmetic, string ops, `re`, basic
control flow. No third-party libraries, no file I/O, no `print`. Top-level statements
must not be indented. See the [monty repository](https://github.com/pydantic/monty)
for the current feature set and known limitations.

**Examples:**

```
%pydef(double, x, %{str(int(x) * 2)%})
%double(21)
```
Output: `42`

```
%pydef(offset, base, size, %{
str(int(base) + int(size))
%})
%offset(256, 64)
```
Output: `320`

```
%pydef(greet, name, %{
"Hello, " + name + "!"
%})
%greet(world)
```
Output: `Hello, world!`

#### `%pyset` and `%pyget` — Python store

A key/value store that persists across all `%pydef` calls within a single run.
All store entries are automatically visible inside every `%pydef` script as plain
Python string variables. Declared parameters shadow any store key with the same name.

```
%pyset(key, value)   — write a string value into the store
%pyget(key)          — read a value from the store (empty string if absent)
```

Write-back is explicit: the script returns its result as usual, and the caller
captures it with `%pyset` if the store needs updating.

**Example — running sum:**

```
%pyset(total, 0)
%pydef(add, n, %{str(int(total) + int(n))%})
%pyset(total, %add(10))
%pyset(total, %add(20))
%pyset(total, %add(12))
Total: %pyget(total)
```
Output: `Total: 42`

**Example — shared prefix:**

```
%pyset(prefix, item_)
%pydef(tagged, name, %{prefix + name%})
%tagged(count)
%tagged(label)
```
Output:
```
item_count
item_label
```

#### `%eval` — indirect macro call

```
%eval(%(macro_name), arg1, arg2)
```

Calls the macro whose name is stored in a variable.

#### `%export` — export a variable or macro to the parent scope

```
%def(init, %{
  %set(x, 10)
  %export(x)
%})
%init()
x is: %(x)
```

#### `%here` — in-place expansion (modifies the source file)

Evaluates its argument and writes the result back into the source file at the
call site. Useful for one-time code generation.

---

### Macro redefinition and the X macro pattern

A `%def` with the same name as an existing macro silently replaces it. This
enables the [X macro](https://en.wikipedia.org/wiki/X_macro) idiom: define a
*list* macro whose body calls a configurable inner macro (`X`) for each entry,
then redefine `X` before each use to project the list onto a different shape.

**Example — generating an enum and a name table from one list:**

```
%def(Colors,
  %X(Red)
  %X(Green)
  %X(Blue)
)

%# Project 1: emit enum variants
%def(X, value, %{%(value),%})
typedef enum {
  %Colors()
} Color;

%# Project 2: emit a string table
%def(X, value, %{[%(value)] = "%(value)",%})
const char *color_names[] = {
  %Colors()
};
```

Output:
```c
typedef enum {
  Red,
  Green,
  Blue,

} Color;
const char *color_names[] = {
  [Red] = "Red",
  [Green] = "Green",
  [Blue] = "Blue",

};
```

The trailing blank line before `}` comes from the final newline in the `Colors`
body; a formatter such as `clang-format` removes it.
Note that `X` need not be defined before `Colors` is defined — only before it
is called. The definition of `Colors` is not evaluated until `%Colors()` is
invoked, at which point whatever `X` is currently bound to is used.

The list is written once; each projection only defines what `X` means for that
context. Adding a new entry to `COLORS` automatically propagates to every
projection.

This composes with `%rhaidef` for projections that require computation, and
with noweb chunks for projections that emit to different output files
simultaneously.

---

## azadi-noweb

A noweb-style chunk extractor. Reads literate source files, resolves chunk
references, and writes output files.

### Usage

```bash
azadi-noweb [OPTIONS] <FILES>...
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--gen <PATH>` | `gen` | Base directory for generated output files |
| `--priv-dir <PATH>` | `_azadi_work` | Private work directory |
| `--output <PATH>` | stdout | Output for `--chunks` extraction |
| `--chunks <NAMES>` | | Comma-separated chunk names to extract to stdout |
| `--open-delim <STR>` | `<[` | Chunk-open delimiter |
| `--close-delim <STR>` | `]>` | Chunk-close delimiter |
| `--chunk-end <STR>` | `@` | End-of-chunk marker |
| `--comment-markers <STR>` | `#,//` | Comment prefixes (comma-separated) |
| `--formatter <EXT=CMD>` | | Run a formatter after writing a file (e.g. `rs=rustfmt`) |

### Chunk syntax

Comment markers (`#` or `//` by default) are stripped before the delimiters
are recognised, so chunks blend naturally into any host language's comment
syntax.

```rust
// <[@file src/hello.rs]>=
fn main() {
    // <[greeting]>
}
// @

// <[greeting]>=
println!("Hello, world!");
// @
```

- `<[@file path]>=` declares a file output chunk. The path may begin with `~/`
  to write to the home directory.
- `<[name]>=` declares a named chunk.
- `<[name]>` inside a chunk body references (expands) another chunk, preserving
  indentation.
- A line matching `// @` (or `# @`) ends the current chunk.

Modifiers go **before** the chunk name, inside the delimiters.

**`@replace`** — discards all prior definitions of that chunk and starts a new
one:

```rust
// <[@replace @file src/main.rs]>=
… new content …
// @
```

**`@reversed`** — on a reference line: expands the referenced chunk's
accumulated definitions in reverse order (last-defined first). Useful for
stack / LIFO patterns.

```rust
// <[@reversed items]>
```

---

## License

MIT OR Apache-2.0
