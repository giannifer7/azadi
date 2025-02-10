// <[@file crates/azadi-macros/src/evaluator/builtins.rs]>=
// crates/azadi-macros/src/evaluator/builtins.rs

use std::collections::{HashMap, HashSet};

use super::evaluator::{EvalError, EvalResult, Evaluator, MacroDefinition, Terminate};
use crate::types::{ASTNode, NodeKind};

//use std::path::Path;
//use std::io;

/// Type for a builtin macro function: (Evaluator, node) -> String
pub type BuiltinFn = fn(&mut Evaluator, &ASTNode) -> EvalResult<String>;

/// Return the default builtins (def, pydef, include, if, equal, eval, here, capitalize, decapitalize).
pub fn default_builtins() -> HashMap<String, BuiltinFn> {
    let mut map = HashMap::new();
    map.insert("def".to_string(), builtin_def as BuiltinFn);
    map.insert("pydef".to_string(), builtin_pydef as BuiltinFn);
    map.insert("include".to_string(), builtin_include as BuiltinFn);
    map.insert(
        "include_silent".to_string(),
        builtin_include_silent as BuiltinFn,
    );
    map.insert("if".to_string(), builtin_if as BuiltinFn);
    map.insert("equal".to_string(), builtin_equal as BuiltinFn);
    map.insert("set".to_string(), builtin_set as BuiltinFn);
    map.insert("export".to_string(), builtin_export as BuiltinFn);
    map.insert("eval".to_string(), builtin_eval as BuiltinFn);
    map.insert("here".to_string(), builtin_here as BuiltinFn);
    map.insert("capitalize".to_string(), builtin_capitalize as BuiltinFn);
    map.insert(
        "decapitalize".to_string(),
        builtin_decapitalize as BuiltinFn,
    );
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
        frozen_args: HashMap::new(),
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

/// Helper: reads, parses, and evaluates the file specified in `node`,
/// returning the resulting output.
fn process_include_file(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    if node.parts.is_empty() {
        return Ok("".into());
    }
    let filename = eval.evaluate(&node.parts[0])?;
    if filename.trim().is_empty() {
        return Ok("".into());
    }
    // Call your existing do_include function to read, parse, and evaluate the file.
    eval.do_include(&filename)
}

/// `%include(filename)` - includes a file for both definitions and text output.
fn builtin_include(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    process_include_file(eval, node)
}

/// `%include_silent(filename)` - includes a file for definitions only;
/// its evaluated output is discarded.
fn builtin_include_silent(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    // Process the file as usual, but discard its output.
    let _ = process_include_file(eval, node)?;
    Ok("".into())
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

/// `%set(var_name, value)` => sets the variable var_name to value
fn builtin_set(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    let parts = &node.parts;
    if parts.len() != 2 {
        return Err(EvalError::InvalidUsage("set: exactly 2 args".into()));
    }
    let var_name = single_ident_param(eval, &node.parts[0], "var name".into())?;
    let value = eval.evaluate(&parts[1])?;
    eval.set_variable(&var_name, &value);
    Ok("".into())
}

/// `%export(var_or_macro)` => copies the var (parameter) or macro to the enclosing scope
fn builtin_export(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    let parts = &node.parts;
    if parts.len() != 1 {
        return Err(EvalError::InvalidUsage("export: exactly 1 arg".into()));
    }
    let name = single_ident_param(eval, &node.parts[0], "var name".into())?;
    eval.export(&name);
    Ok("".into())
}

/// `%eval(macroName, param1, param2, ...)`
fn builtin_eval(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    let parts = &node.parts;
    if parts.is_empty() {
        return Err(EvalError::InvalidUsage("eval requires macroName".into()));
    }
    let macro_name = eval.evaluate(&parts[0])?;
    let macro_name = macro_name.trim();
    if macro_name.is_empty() {
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

/// `%here(...)`: Modifies the current file at the node's position by inserting the evaluated content.
/// Terminates execution after modifying the file.
fn builtin_here(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
    // If the node has no parts, return an empty string
    if node.parts.is_empty() {
        return Ok("".into());
    }

    // Evaluate the content inside the `%here` macro
    let expansion = builtin_eval(eval, node)?;

    // Get the current file path and the node's position/length
    let path = eval.get_current_file_path();
    let start_pos = node.token.pos; // Start position of %here(...)

    // Prepare the first triplet: prepend the special character before %here
    let prepend_triplet = (start_pos, eval.get_special_char(), false);

    // Prepare the second triplet: append the expansion after %here(...)
    let append_triplet = (node.end_pos, expansion.into_bytes(), true);

    // Call `modify_source` with both triplets and backup directory
    super::source_utils::modify_source(
        &path,
        &[prepend_triplet, append_triplet],
        Some(&eval.get_backup_dir_path()),
    )?;

    // Terminate execution by returning a special "termination" signal
    Err(EvalError::Terminate(Terminate))
}

/// `%capitalize(...)` => uppercase first letter
fn builtin_capitalize(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
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

/// `%decapitalize(...)` => lowercase first letter
fn builtin_decapitalize(eval: &mut Evaluator, node: &ASTNode) -> EvalResult<String> {
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
