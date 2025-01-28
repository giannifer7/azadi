// crates/azadi-cli/src/lib.rs
#![feature(error_generic_member_access)]

pub mod pipeline;
pub use pipeline::{run_pipeline, Args, PipelineError};
