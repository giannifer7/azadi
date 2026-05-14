// weaveback-lsp/src/registry.rs
// I'd Really Rather You Didn't edit this generated file.

/// Returns (command, language_id) for a given file extension.
pub fn get_lsp_config(ext: &str) -> Option<(String, String)> {
    match ext {
        "rs"  => Some(("rust-analyzer".to_string(), "rust".to_string())),
        "nim" => Some(("nimlsp".to_string(), "nim".to_string())),
        "py"  => Some(("pyright-langserver --stdio".to_string(), "python".to_string())),
        _     => None,
    }
}
