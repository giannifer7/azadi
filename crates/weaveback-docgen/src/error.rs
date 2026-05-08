// weaveback-docgen/src/error.rs
// I'd Really Rather You Didn't edit this generated file.

use super::render;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub(super) enum Error {
    #[error("documentation rendering failed")]
    #[diagnostic(code(weaveback::docgen::render))]
    Render {
        #[from]
        #[source]
        source: render::RenderError,
    },
    #[error("failed to run xref command '{cmd}'")]
    #[diagnostic(code(weaveback::docgen::xref_cmd_spawn))]
    XrefCommandSpawn {
        cmd: String,
        #[source]
        source: std::io::Error,
    },
    #[error("xref command '{cmd}' exited with status {code}")]
    #[diagnostic(code(weaveback::docgen::xref_cmd_status))]
    XrefCommandStatus { cmd: String, code: i32 },
    #[error("failed to parse JSON from xref command '{cmd}'")]
    #[diagnostic(code(weaveback::docgen::xref_cmd_json))]
    XrefCommandJson {
        cmd: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to serialise xref data")]
    #[diagnostic(code(weaveback::docgen::xref_json))]
    XrefJson {
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to write xref data to {path}")]
    #[diagnostic(code(weaveback::docgen::xref_write))]
    XrefWrite {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
