// crates/azadi-cli/src/main.rs
use azadi_cli::{run_pipeline, Args, PipelineError};
use azadi_macros::evaluator::EvalError;
use clap::Parser;

fn main() {
    let args = Args::parse();

    match run_pipeline(args) {
        Ok(()) => {}
        Err(PipelineError::MacroError {
            source: EvalError::Terminate(_),
            ..
        }) => {
            eprintln!("%here macro executed successfully");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
