# Azadi

Azadi is a literate-programming toolchain. It consists of two tools that work together in
a pipeline:

```
azadi-macros source.md | azadi-noweb
```

1. **azadi-macros** — expands macro definitions embedded in a source document
2. **azadi-noweb** — extracts named code chunks and writes them to output files

---

## Quick start

```bash
cargo build --release
# binaries land in target/release/azadi-macros and target/release/azadi-noweb
```

---

## azadi-macros

A macro expander. Reads one or more source files, evaluates `%macro(...)` calls, and
writes the result to `--output`.

### Usage

```bash
azadi-macros [OPTIONS] <INPUTS>...
```

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--output <PATH>` | `.` | Output file (or directory when processing multiple inputs) |
| `--special <CHAR>` | `%` | Macro invocation character |
| `--work-dir <PATH>` | `_azadi_work` | Directory for backup / intermediate files |
| `--include <PATHS>` | `.` | Include search paths (separated by `--pathsep`) |
| `--pathsep <STR>` | `:` / `;` | Path separator (platform default) |
| `--input-dir <PATH>` | `.` | Base directory prepended to each input path |

### Syntax

#### Argument whitespace

Leading whitespace in every argument is stripped. This lets you align
multi-argument calls without spaces leaking into the expanded text:

```
%def(tag, name, value, <%(name)>%(value)</%(name)>)
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

`%include` expands the included file inline. `%import` expands it but discards the output
(useful for loading macro definitions).

```
%import(macros/common.txt)
%my_macro(arg)
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

The body is a [Rhai](https://rhai.rs) script. It is evaluated at call time; its return
value (converted to string) becomes the macro output.

All visible azadi scope variables are injected into the Rhai script as string variables.
The body **must** be wrapped in `%{ ... %}` whenever it contains parentheses, so that
azadi does not misparse them as argument separators.

**Registered Rhai helpers:**

| Function | Signature | Description |
|----------|-----------|-------------|
| `parse_int(s)` | `String → i64` | Parse string to integer (returns 0 on error) |
| `parse_float(s)` | `String → f64` | Parse string to float (returns 0.0 on error) |
| `to_hex(n)` | `i64 → String` | Format integer as `0xHEX` |

Rhai does **not** support Rust turbofish syntax — use the helpers above instead of
`.parse::<i64>()`.

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

Evaluates its argument and writes the result back into the source file at the call site.
Useful for one-time code generation.

---

## azadi-noweb

A noweb-style chunk extractor. Reads literate source files, resolves chunk references, and
writes output files.

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
| `--open-delim <STR>` | `<<` | Chunk-open delimiter |
| `--close-delim <STR>` | `>>` | Chunk-close delimiter |
| `--chunk-end <STR>` | `@` | End-of-chunk marker |
| `--comment-markers <STR>` | `#,//` | Comment prefixes (comma-separated) |
| `--formatter <EXT=CMD>` | | Run a formatter after writing a file (e.g. `rs=rustfmt`) |

### Chunk syntax

Comment markers (`#` or `//` by default) are stripped before the delimiters are
recognised, so chunks blend naturally into any host language's comment syntax.

```rust
// <<@file src/hello.rs>>=
fn main() {
    // <<greeting>>
}
// @

// <<greeting>>=
println!("Hello, world!");
// @
```

- `<<@file path>>=` declares a file output chunk.
- `<<name>>=` declares a named chunk.
- `<<name>>` inside a chunk body references (expands) another chunk, preserving indentation.
- A line matching `// @` (or `# @`) ends the current chunk.

Modifiers go **before** the chunk name, inside the delimiters.

**`@replace`** — on a definition line: discards all prior definitions of that chunk and
starts a new one.

```rust
// <<@replace @file src/main.rs>>=
… new content …
// @
```

**`@reversed`** — on a reference line: expands the referenced chunk's accumulated
definitions in reverse order (last-defined first). Useful for stack / LIFO patterns.

```rust
// <<@reversed items>>
```

### Full pipeline example

Given `src/app.md`:

````markdown
# My App

## Entry point

// <<@file src/main.rs>>=
fn main() {
    // <<setup>>
    // <<run>>
}
// @

// <<setup>>=
let config = Config::default();
// @

// <<run>>=
app::run(config);
// @
````

Run:

```bash
azadi-macros src/app.md | azadi-noweb --gen .
```

This writes `src/main.rs` with all chunks expanded in place.

---

## License

MIT OR Apache-2.0
