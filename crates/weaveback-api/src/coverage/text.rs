// weaveback-api/src/coverage/text.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

mod attribution;
mod cargo_run;
mod query;

pub use cargo_run::{run_cargo_annotated, run_cargo_annotated_to_writer};
pub use query::{run_graph, run_impact, run_search, run_tags, run_trace};
pub(in crate::coverage) use attribution::{collect_text_attributions, emit_text_attribution_message};
