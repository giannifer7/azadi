# Azadi

A minimalistic macro processor written in Rust that helps with text transformation and code generation.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Azadi is a simple macro processor that helps you automate repetitive text transformations. It supports basic macro definitions, file inclusion, conditional processing, and Python integration.

## Installation

Add Azadi to your project dependencies:

```toml
[dependencies]
azadi-macros = "0.1.0"
```

Or install the CLI tool:

```bash
cargo install --path crates/azadi-macros
# The binary is named 'azadi-macro'
```

## Quick Start

Here are some basic examples to get you started:

### Simple Macro Definition
```
%def(hello, World)
Hello %hello()!
```
Output: `Hello World!`

### Macro with Parameters
```
%def(greet, name, Hello, %(name)!)
%greet(Alice)
```
Output: `Hello, Alice!`

### Block Content
```
%def(template, content, %{
  ====
  %(content)
  ====
%})
%template(Some text here)
```
Output:
```
  ====
  Some text here
  ====
```

## Command Line Usage

The CLI tool is named `azadi-macro`.

```bash
azadi-macro [OPTIONS] <INPUT_FILES>...
```

### Options

| Option | Default | Description |
|--------|---------|-------------|
| `--out-file <PATH>` | `.` | Output directory or file path |
| `--special <CHAR>` | `%` | Special character for macros |
| `--work-dir <DIR>` | `_azadi_work` | Working directory for backups |
| `--include <PATHS>` | `.` | Colon-separated list of include paths |
| `--pathsep <SEP>` | `:` (Unix) / `;` (Win) | Path separator for include paths |
| `--input-dir <DIR>` | `.` | Base directory for input files |
| `--pydef` | `false` | Enable Python macro definitions |
| `--venv-path <PATH>` | - | Path to Python virtual environment |
| `--python-path <PATH>` | - | Path to Python executable |

### Basic Usage

```bash
# Process a file
azadi-macro input.txt

# Use stdin/stdout (use '-' for stdin)
cat input.txt | azadi-macro -

# Process multiple files
azadi-macro file1.txt file2.txt

# Specify output directory
azadi-macro --out-file dist/ input.txt
```

## Configuration

Configuration can be specified in several files:

### azadi.toml
```toml
input_dir = "src"
output_dir = "gen"
special = "%"
```

### pyproject.toml
```toml
[tool.azadi]
work_dir = "_work"
open_delim = "<["
close_delim = "]>"
```

## Real-World Examples

Here are some practical examples:

### 1. Nested Macro Calls
```
%def(inner, x, inner(%(x)))
%def(outer, y, outer(%inner(%(y))))
%outer(test)
```
Output: `outer(inner(test))`

### 2. Conditional Processing
```
%def(greet, name, %{
    %if(%(name), %{
        Hello, %(name)!
    %}, %{
        Hello, stranger!
    %})
%})
%greet(World)
%greet()
```
Output:
```
Hello, World!
Hello, stranger!
```

### 3. File Organization Template
Here's a template for creating Rust files with a specific structure:

```
%def(rust_tpl, name, dir, content, %{
    %def(the_file, name, dir, %{%if(%(dir),%(dir),.)/%(name).rs%})
[,rust]
----
// <[@file %the_file(%(name), %(dir))]>=
// %the_file(%(name), %(dir))
%(content)
// <[%the_file(%(name), %(dir))_content]>
// $$
// <[%the_file(%(name), %(dir))_content]>=
// $$
----
%})
```

Usage example:
```
%rust_tpl(my_module, src/modules, %{
pub fn hello() {
    println!("Hello from module!");
}
%})
```

### 4. Basic Text Templates
```
%def(bold, text, **%(text)**)
%def(greet, name, %{
    %bold(Hello), %(name)!
    Welcome to our project.
%})
%greet(Alice)
```

### 5. Variable Usage
```
%def(set_x, val, %(val))
%def(message, %{
    Value is: %(set_x(hello))
%})
%message()
```

### 6. Comments
```
%// Line comment
%/*
   Block comment
   %*/
%# Hash comment
%-- Dash comment

%def(example, %{
    %// This is inside a macro definition
    Some content
%})
```

### 7. Include Files
```
%// config.txt contains: PORT=8080
%include(config.txt)
Server port: %(PORT)
```

## Built-in Macros

Azadi provides several built-in macros. Leading spaces in macro parameters are ignored.

### %def(name, [params...], body)
Defines a new macro.
```
%def(greet, name, Hello %(name)!)
%greet(  World  )  // spaces before "World" are ignored
```

### %if(condition, then-part [, else-part])
Conditional processing. The condition is considered true if it's non-empty.
```
%def(debug, true)
%if(%(debug), 
    Debug mode is on,
    Debug mode is off
)
```

### %equal(a, b)
Returns a if a and b match exactly, otherwise returns empty string.
```
%equal(abc, abc)    // returns "abc"
%equal(abc, def)    // returns ""
```

### %eval(macro-name, [args...])
Evaluates a macro by name.
```
%def(say, Hello!)
%eval(say)        // same as %say()
```

### %include(filename)
Includes content from another file. Detects circular includes.
```
%include(header.txt)
```

### %here(macro_name, params...)
Modifies the current file at the macro's position by evaluating the macro with the given parameters, then terminates processing. Often used to insert generated content into the source file itself.
```
// File: example.rs
%def(struct_fields, type, %{
    name: String,
    count: %(type),
    enabled: bool
%})

struct Example {
    // Generated fields will go here:
    %here(struct_fields, i32)
}
```

After processing, the file will contain:
```rust
struct Example {
    // Generated fields will go here:
    %name: String,
    count: i32,
    enabled: bool
}
```
Note the special character (%) is automatically added before the inserted content.

### %capitalize(text)
Makes first letter uppercase.
```
%capitalize(hello)   // returns "Hello"
```

### %decapitalize(text)
Makes first letter lowercase.
```
%decapitalize(Hello)   // returns "hello"
```

## Features

- Basic macro processing
- Basic conditional processing with `%if`
- File inclusion
- Multiple comment styles
- Text blocks using `%{...%}`
- Standard input/output support
- Backup file generation
- UTF-8 text processing
- **Python Integration**: Define macros in Python (requires `--pydef` and optional configuration).

## Development

### Building from Source

1. Clone the repository
2. Build: `cargo build`
3. Run tests: `cargo test`

### Project Structure
```
.
├── Cargo.toml
└── crates
    └── azadi-macros
        ├── src/
        └── tests/
```

## License

This project is licensed under the MIT License - see the [LICENSE-MIT](LICENSE-MIT) file for details.
