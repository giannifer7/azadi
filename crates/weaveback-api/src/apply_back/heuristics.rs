// weaveback-api/src/apply_back/heuristics.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

mod body;
mod macro_arg;
mod noweb;
mod ranking;
mod search;

pub(in crate::apply_back) use macro_arg::attempt_macro_arg_patch;
pub(in crate::apply_back) use noweb::resolve_noweb_entry;
pub(in crate::apply_back) use search::{
    search_macro_arg_candidate,
    search_macro_body_candidate,
    search_macro_call_candidate,
};

#[cfg(test)]
pub(in crate::apply_back) use body::attempt_macro_body_fix;
#[cfg(test)]
pub(in crate::apply_back) use ranking::{choose_best_candidate, rank_candidate};

