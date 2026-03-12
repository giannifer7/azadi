// crates/azadi-macros/src/evaluator/monty_eval.rs

use monty::{MontyObject, MontyRun};

pub struct MontyEvaluator;

impl MontyEvaluator {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(
        &self,
        code: &str,
        params: &[String],
        args: &[String],
        name: Option<&str>,
    ) -> Result<String, String> {
        let macro_name = name.unwrap_or("pydef");
        let runner = MontyRun::new(
            code.to_owned(),
            &format!("{macro_name}.py"),
            params.to_vec(),
        )
        .map_err(|e| format!("pydef '{macro_name}': compile error: {e:?}"))?;

        let inputs: Vec<MontyObject> = args
            .iter()
            .map(|s| MontyObject::String(s.clone()))
            .collect();

        let result = runner
            .run_no_limits(inputs)
            .map_err(|e| format!("pydef '{macro_name}': runtime error: {e:?}"))?;

        Ok(monty_object_to_string(result))
    }
}

fn monty_object_to_string(obj: MontyObject) -> String {
    match obj {
        MontyObject::String(s) => s,
        MontyObject::Int(n) => n.to_string(),
        MontyObject::Float(f) => f.to_string(),
        MontyObject::Bool(b) => if b { "true".into() } else { "false".into() },
        MontyObject::None => String::new(),
        MontyObject::List(items) => items
            .into_iter()
            .map(monty_object_to_string)
            .collect::<Vec<_>>()
            .join(""),
        other => format!("{other:?}"),
    }
}
