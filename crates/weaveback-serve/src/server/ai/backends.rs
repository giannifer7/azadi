// weaveback-serve/src/server/ai/backends.rs
// I'd Really Rather You Didn't edit this generated file.

mod anthropic;
mod claude;
mod gemini;
mod ollama;
mod openai;

pub(in crate::server::ai) use anthropic::call_anthropic_api;
pub(in crate::server::ai) use claude::call_claude_cli;
pub(in crate::server::ai) use gemini::call_gemini_api;
pub(in crate::server::ai) use ollama::call_ollama_api;
pub(in crate::server::ai) use openai::call_openai_api;
