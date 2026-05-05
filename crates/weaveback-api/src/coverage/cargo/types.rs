// weaveback-api/src/coverage/cargo/types.rs
// I'd Really Rather You Didn't edit this generated file.

#[derive(Debug, serde::Deserialize)]
pub(in crate::coverage) struct CargoMessageEnvelope {
    pub(in crate::coverage) reason: String,
    pub(in crate::coverage) message: Option<CargoDiagnostic>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CargoDiagnostic {
    pub spans: Vec<CargoDiagnosticSpan>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CargoDiagnosticSpan {
    pub file_name: String,
    pub line_start: u32,
    pub column_start: u32,
    pub is_primary: bool,
}

