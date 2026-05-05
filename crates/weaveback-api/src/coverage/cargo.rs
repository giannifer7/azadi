// weaveback-api/src/coverage/cargo.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

mod attribution;
mod emit;
mod summary;
mod types;

pub use attribution::{
    collect_cargo_attributions,
    collect_cargo_span_attributions,
};
pub(in crate::coverage) use attribution::trace_generated_location;
pub use emit::{
    emit_augmented_cargo_message,
    emit_cargo_summary_message,
};
pub use summary::{
    build_cargo_attribution_summary,
    build_location_attribution_summary,
};
pub use types::{
    CargoDiagnostic,
};
#[cfg(test)]
pub use types::CargoDiagnosticSpan;
pub(in crate::coverage) use types::CargoMessageEnvelope;

