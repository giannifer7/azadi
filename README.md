# Azadi

Azadi is a workspace containing tools for literate programming and macro expansion in Rust.

## Components

### azadi-noweb

`azadi-noweb` is a Rust implementation of a noweb-style literate programming tool. It allows extracting code chunks from literate source files.

#### Usage

```bash
azadi-noweb [OPTIONS] <FILES>...
```

#### Options

- `--output <PATH>`: Output file for `--chunks` (default: stdout).
- `--chunks <NAMES>`: Names of chunks to extract (comma separated).
- `--priv-dir <PATH>`: Private work directory (default: `_azadi_work`).
- `--gen <PATH>`: Base directory of generated files (default: `gen`).
- `--open-delim <STRING>`: Delimiter used to open a chunk (default: `<<`).
- `--close-delim <STRING>`: Delimiter used to close a chunk definition (default: `>>`).
- `--chunk-end <STRING>`: Delimiter for chunk-end lines (default: `@`).
- `--comment-markers <STRINGS>`: Comment markers, comma separated (default: `#,//`).

### azadi-macros

`azadi-macros` is a macro translator and evaluator.

#### Usage

```bash
```bash
azadi-macros [OPTIONS] <INPUTS>...
```

#### Options

- `--output <PATH>`: Output path (default: current directory).
- `--special <CHAR>`: Special character for macros (default: `%`).
- `--work-dir <PATH>`: Working directory for backups (default: `_azadi_work`).
- `--include <PATHS>`: List of include paths separated by the path separator (default: `.`).
- `--pathsep <STRING>`: Path separator (default: system specific, `:` or `;`).
- `--python-path <PATH>`: Path to Python executable or venv directory.
- `--pydef`: Enable Python macros (default: false).
- `--input-dir <PATH>`: Base directory for input files (default: `.`).

## Building

To build the project, run:

```bash
cargo build --release
```

## License

This project is licensed under MIT or Apache-2.0.
