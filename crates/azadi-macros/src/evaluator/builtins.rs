// <[@file crates/azadi-macros/src/evaluator/builtins.rs]>=
// crates/azadi-macros/src/evaluator/builtins.rs

use std::collections::{HashMap, HashSet};

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

/// Helper: Checks that a Param node contains exactly one identifier child
/// (ignoring spaces/comments), then returns that identifier’s text.
fn single_ident_param(eval: &Evaluator, param_node: &ASTNode, desc: &str) -> EvalResult<String> {
    // Must be a `Param` node
    if param_node.kind != NodeKind::Param {
        return Err(EvalError::InvalidUsage(format!(
            "{desc} must be a Param node"
        )));
    }
    // In your AST, `param_node.name != None` means there's a "name=" portion;
    // we disallow that here, requiring a single plain identifier.
    if param_node.name.is_some() {
        return Err(EvalError::InvalidUsage(format!(
            "{desc} must be a single identifier (found an '=' style param?)"
        )));
    }

    // Filter out space/comment children
    let nonspace: Vec<_> = param_node
        .parts
        .iter()
        .filter(|child| {
            !matches!(
                child.kind,
                NodeKind::Space | NodeKind::LineComment | NodeKind::BlockComment
            )
        })
        .collect();

    // Must have exactly one child, and that child must be an Ident
    if nonspace.len() != 1 {
        return Err(EvalError::InvalidUsage(format!(
            "{desc} must be a single identifier"
        )));
    }
    let ident_node = &nonspace[0];
    if ident_node.kind != NodeKind::Ident {
        return Err(EvalError::InvalidUsage(format!(
            "{desc} must be a single identifier"
        )));
    }

    // Read raw text from that Ident node:
    let text = eval.node_text(ident_node);
    if text.trim().is_empty() {
        return Err(EvalError::InvalidUsage(format!("{desc} cannot be empty")));
    }
    Ok(text)
}

struct DefMacroConfig {
    min_params_error: String,
    name_param_context: String,
    formal_param_context: String,
    duplicate_param_error: String,
    is_python: bool,
}

fn define_macro(
    eval: &mut Evaluator,
    node: &ASTNode,
    config: DefMacroConfig,
) -> EvalResult<String> {
    if node.parts.len() < 2 {
        return Err(EvalError::InvalidUsage(config.min_params_error));
    }

    let macro_name = single_ident_param(eval, &node.parts[0], &config.name_param_context)?;
    let body_node = node.parts.last().unwrap().clone();

    let mut seen = HashSet::new();
    let param_list = node.parts[1..(node.parts.len() - 1)].iter().try_fold(
        Vec::new(),
        |mut acc, param_node| {
            let param_name = single_ident_param(eval, param_node, &config.formal_param_context)?;
            if !seen.insert(param_name.clone()) {
                return Err(EvalError::InvalidUsage(format!(
                    "{}: parameter '{param_name}' already used",
                    config.duplicate_param_error
                )));
            }
            acc.push(param_name);
            Ok(acc)
        },
    )?;

    let mac = MacroDefinition {
        name: macro_name,
        params: param_list,
        body: body_node,
        is_python: config.is_python,
    };
    eval.define_macro(mac);
    Ok("".into())
}

pub fn builtin_def(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    define_macro(
        eval,
        node,
        DefMacroConfig {
            min_params_error: "def requires at least (name, body)".into(),
            name_param_context: "macro name".into(),
            formal_param_context: "formal parameter".into(),
            duplicate_param_error: "def".into(),
            is_python: false,
        },
    )
}

pub fn builtin_pydef(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    define_macro(
        eval,
        node,
        DefMacroConfig {
            min_params_error: "pydef requires at least (name, body)".into(),
            name_param_context: "pydef name".into(),
            formal_param_context: "pydef parameter".into(),
            duplicate_param_error: "pydef".into(),
            is_python: true,
        },
    )
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
