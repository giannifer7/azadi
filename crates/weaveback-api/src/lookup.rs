// weaveback-api/src/lookup.rs
// I'd Really Rather You Didn't edit this generated file.

mod context;
mod span;
mod trace;
mod where_lookup;

pub use context::build_source_context_value;
pub use trace::{load_source_text, perform_trace, perform_trace_coarse};
pub use where_lookup::perform_where;

#[cfg(test)]
use context::append_source_context;
#[cfg(test)]
use span::append_def_locations;

use weaveback_tangle::db::WeavebackDb;
use weaveback_core::PathResolver;

#[derive(Debug, thiserror::Error)]
pub enum LookupError {
    #[error("{0}")]
    Db(#[from] weaveback_tangle::db::DbError),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    InvalidInput(String),
}
#[cfg(test)]
mod tests;
