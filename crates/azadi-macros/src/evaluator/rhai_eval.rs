// crates/azadi-macros/src/evaluator/rhai_eval.rs

use rhai::{Dynamic, Engine, Scope};
use std::collections::HashMap;

pub struct RhaiEvaluator {
    engine: Engine,
}

impl Default for RhaiEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl RhaiEvaluator {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.set_max_operations(100_000);

        engine.register_fn("parse_int", |s: &str| -> i64 {
            s.trim().parse::<i64>().unwrap_or(0)
        });
        engine.register_fn("parse_float", |s: &str| -> f64 {
            s.trim().parse::<f64>().unwrap_or(0.0)
        });
        engine.register_fn("to_hex", |n: i64| -> String { format!("0x{:X}", n) });

        Self { engine }
    }

    pub fn evaluate(
        &self,
        code: &str,
        variables: &HashMap<String, String>,
        name: Option<&str>,
    ) -> Result<String, String> {
        let mut scope = Scope::new();
        for (k, v) in variables {
            scope.push_dynamic(k, Dynamic::from(v.clone()));
        }
        let result: Dynamic = self
            .engine
            .eval_with_scope(&mut scope, code)
            .map_err(|e| format!("rhaidef '{}': {}", name.unwrap_or("?"), e))?;
        Ok(dynamic_to_string(result))
    }
}

fn dynamic_to_string(d: Dynamic) -> String {
    if d.is::<String>() {
        d.cast::<String>()
    } else if d.is_unit() {
        String::new()
    } else {
        d.to_string()
    }
}
