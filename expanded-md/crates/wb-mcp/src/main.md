# wb-mcp

`wb-mcp` exposes the weaveback MCP (Model Context Protocol) server as a
standalone binary.  It reads JSON-RPC requests from stdin and writes
responses to stdout, making it usable directly in MCP client configurations
without requiring the full `weaveback` binary.

## CLI

Generated from `cli-spec/wb-mcp-cli.adoc`.

```rust
// <[wb-mcp-cli]>=
mod cli_generated;
use cli_generated::Cli;
use clap::Parser;
use miette::Diagnostic;
use thiserror::Error;
// @
```


## Main

```rust
// <[wb-mcp-main]>=
#[derive(Debug, Error, Diagnostic)]
enum Error {
    #[error("MCP server failed")]
    #[diagnostic(code(weaveback::mcp))]
    Mcp {
        #[from]
        #[source]
        source: std::io::Error,
    },
}

fn default_pathsep() -> String {
    if cfg!(windows) { ";".to_string() } else { ":".to_string() }
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();

    let pathsep = default_pathsep();
    let include_paths: Vec<std::path::PathBuf> = cli.include
        .split(&pathsep)
        .map(std::path::PathBuf::from)
        .collect();

    let eval_config = weaveback_macro::evaluator::EvalConfig {
        sigil: cli.sigil,
        include_paths,
        allow_env: cli.allow_env,
        ..Default::default()
    };

    let project_root = std::env::current_dir().unwrap_or_else(|_| ".".into());
    weaveback_api::mcp::run_mcp(std::io::stdin().lock(), std::io::stdout(), cli.db, cli.gen_dir, project_root, eval_config)
        .map_err(Error::from)?;
    Ok(())
}
// @
```


## Assembly

```rust
// <[@file wb-mcp/src/main.rs]>=
// wb-mcp/src/main.rs
// I'd Really Rather You Didn't edit this generated file.

// <[wb-mcp-cli]>
// <[wb-mcp-main]>

// @
```

