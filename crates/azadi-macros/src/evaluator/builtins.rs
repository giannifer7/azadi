// <[@file crates/azadi-macros/src/evaluator/builtins.rs]>=
// azadi/crates/azadi-macros/src/evaluator/builtins.rs

use std::collections::HashMap;

use super::evaluator::{EvalError, EvalResult, Evaluator, MacroDefinition};
use crate::types::{ASTNode, NodeKind};

/// Type for a builtin macro function: (Evaluator, node) -> String
pub type BuiltinFn = fn(&mut Evaluator, &ASTNode) -> EvalResult<String>;

/// Return the default builtins (def, pydef, include, if, equal, eval, here, upper, lower).
pub fn default_builtins() -> HashMap<String, BuiltinFn> {
    let mut map = HashMap::new();
    map.insert("def".to_string(), builtin_def as BuiltinFn);
    map.insert("pydef".to_string(), builtin_pydef as BuiltinFn);
    map.insert("include".to_string(), builtin_include as BuiltinFn);
    map.insert("if".to_string(), builtin_if as BuiltinFn);
    map.insert("equal".to_string(), builtin_equal as BuiltinFn);
    map.insert("eval".to_string(), builtin_eval as BuiltinFn);
    map.insert("here".to_string(), builtin_here as BuiltinFn);
    map.insert("upper".to_string(), builtin_upper as BuiltinFn);
    map.insert("lower".to_string(), builtin_lower as BuiltinFn);
    map
}

/// `%def(name, param1, param2, ..., body)`
fn builtin_def(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    let parts = &node.parts;
    if parts.len() < 2 {
        return Err(EvalError::InvalidUsage(
            "def requires (name, ...params..., body)".into(),
        ));
    }
    let macro_name = eval.evaluate(&parts[0])?;
    if macro_name.is_empty() {
        return Err(EvalError::InvalidUsage("def: empty macro name".into()));
    }
    let body_node = parts.last().unwrap();
    let param_nodes = &parts[1..(parts.len() - 1)];
    let mut param_list = Vec::new();
    for pn in param_nodes {
        let p = eval.evaluate(pn)?;
        if p.is_empty() {
            return Err(EvalError::InvalidUsage("def: empty param".into()));
        }
        param_list.push(p);
    }
    let definition = MacroDefinition {
        name: macro_name,
        params: param_list,
        body: body_node.clone(),
        is_python: false,
    };
    eval.define_macro(definition);
    Ok("".to_string())
}

/// `%pydef(...)` => same as `%def`, but is_python=true
fn builtin_pydef(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    let parts = &node.parts;
    if parts.len() < 2 {
        return Err(EvalError::InvalidUsage(
            "pydef requires (name, ...params..., body)".into(),
        ));
    }
    let macro_name = eval.evaluate(&parts[0])?;
    if macro_name.is_empty() {
        return Err(EvalError::InvalidUsage("pydef: empty macro name".into()));
    }
    let body_node = parts.last().unwrap();
    let param_nodes = &parts[1..(parts.len() - 1)];
    let mut param_list = Vec::new();
    for pn in param_nodes {
        let p = eval.evaluate(pn)?;
        if p.is_empty() {
            return Err(EvalError::InvalidUsage("pydef: empty param".into()));
        }
        param_list.push(p);
    }
    let definition = MacroDefinition {
        name: macro_name,
        params: param_list,
        body: body_node.clone(),
        is_python: true,
    };
    eval.define_macro(definition);
    Ok("".to_string())
}

/// `%include(filename)`
fn builtin_include(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    if node.parts.is_empty() {
        return Ok("".into());
    }
    let filename = eval.evaluate(&node.parts[0])?;
    if filename.trim().is_empty() {
        return Ok("".into());
    }
    eval.do_include(&filename)
}

/// `%if(condition, thenVal, elseVal)`
fn builtin_if(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    let parts = &node.parts;
    if parts.is_empty() {
        return Ok("".into());
    }
    let cond = eval.evaluate(&parts[0])?;
    if !cond.trim().is_empty() {
        if parts.len() > 1 {
            eval.evaluate(&parts[1])
        } else {
            Ok("".into())
        }
    } else {
        if parts.len() > 2 {
            eval.evaluate(&parts[2])
        } else {
            Ok("".into())
        }
    }
}

/// `%equal(a,b)` => returns `a` if they match, else ""
fn builtin_equal(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    let parts = &node.parts;
    if parts.len() != 2 {
        return Err(EvalError::InvalidUsage("equal: exactly 2 args".into()));
    }
    let a = eval.evaluate(&parts[0])?;
    let b = eval.evaluate(&parts[1])?;
    if a == b {
        Ok(a)
    } else {
        Ok("".into())
    }
}

/// `%eval(macroName, param1, param2, ...)`
fn builtin_eval(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    let parts = &node.parts;
    if parts.is_empty() {
        return Err(EvalError::InvalidUsage("eval requires macroName".into()));
    }
    let macro_name = eval.evaluate(&parts[0])?;
    if macro_name.trim().is_empty() {
        return Ok("".into());
    }
    let rest = if parts.len() > 1 {
        parts[1..].to_vec()
    } else {
        vec![]
    };
    let call_node = ASTNode {
        kind: NodeKind::Macro,
        src: node.src,
        token: node.token.clone(),
        end_pos: node.end_pos,
        parts: rest,
        name: None,
    };
    eval.evaluate_macro_call(&call_node, &macro_name)
}

/// `%here(...)`: modifies the current file at the node's position
fn builtin_here(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    if node.parts.is_empty() {
        return Ok("".into());
    }
    let expansion = eval.evaluate(&node.parts[0])?;
    let path = eval.get_current_file_path();
    let insertion_pos = node.token.pos;
    super::source_utils::modify_source(
        &path,
        &[(insertion_pos, expansion.into_bytes(), false)],
        Some(&eval.get_backup_dir_path()),
    )
    .map_err(|e| EvalError::Runtime(format!("`here` macro error: {}", e)))?;
    Ok("".into())
}

/// `%upper(...)` => uppercase first letter
fn builtin_upper(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    if node.parts.is_empty() {
        return Ok("".into());
    }
    let original = eval.evaluate(&node.parts[0])?;
    if original.is_empty() {
        return Ok("".into());
    }
    let mut chars = original.chars();
    let first = chars.next().unwrap().to_uppercase().to_string();
    Ok(format!("{}{}", first, chars.collect::<String>()))
}

/// `%lower(...)` => lowercase first letter
fn builtin_lower(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    if node.parts.is_empty() {
        return Ok("".into());
    }
    let original = eval.evaluate(&node.parts[0])?;
    if original.is_empty() {
        return Ok("".into());
    }
    let mut chars = original.chars();
    let first = chars.next().unwrap().to_lowercase().to_string();
    Ok(format!("{}{}", first, chars.collect::<String>()))
}
