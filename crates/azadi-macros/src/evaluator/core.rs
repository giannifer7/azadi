// crates/azadi-macros/src/evaluator/core.rs

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::builtins::{default_builtins, BuiltinFn};
use super::errors::{EvalError, EvalResult};
use super::rhai_eval::RhaiEvaluator;
use super::state::{EvalConfig, EvaluatorState, MacroDefinition, MAX_RECURSION_DEPTH};
use crate::types::{ASTNode, NodeKind, Token, TokenKind};

pub struct Evaluator {
    state: EvaluatorState,
    builtins: HashMap<String, BuiltinFn>,
    rhai_evaluator: RhaiEvaluator,
}

impl Evaluator {
    pub fn new(config: EvalConfig) -> Self {
        Evaluator {
            state: EvaluatorState::new(config),
            builtins: default_builtins(),
            rhai_evaluator: RhaiEvaluator::new(),
        }
    }

    pub fn define_macro(&mut self, mac: crate::evaluator::state::MacroDefinition) {
        self.state.define_macro(mac);
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        self.state.set_variable(name, value);
    }

    pub fn add_source_if_not_present(&mut self, file_path: PathBuf) -> Result<u32, std::io::Error> {
        self.state
            .source_manager
            .add_source_if_not_present(file_path)
    }

    pub fn add_source_bytes(&mut self, content: Vec<u8>, path: PathBuf) -> u32 {
        self.state.source_manager.add_source_bytes(content, path)
    }

    pub fn set_current_file(&mut self, file: PathBuf) {
        self.state.current_file = file;
    }

    pub fn get_current_file_path(&self) -> PathBuf {
        self.state.current_file.clone()
    }

    pub fn get_backup_dir_path(&self) -> PathBuf {
        self.state.config.backup_dir.clone()
    }

    pub fn get_special_char(&self) -> Vec<u8> {
        self.state.get_special_char()
    }

    pub fn set_early_exit(&mut self) {
        self.state.early_exit = true;
    }

    pub fn evaluate(&mut self, node: &ASTNode) -> EvalResult<String> {
        if self.state.early_exit {
            return Ok(String::new());
        }
        let mut out = String::new();
        match node.kind {
            NodeKind::Text | NodeKind::Space | NodeKind::Ident => {
                let txt = self.node_text(node);
                out.push_str(&txt);
            }
            NodeKind::Var => {
                let var_name = self.node_text(node);
                let val = self.state.get_variable(&var_name);
                out.push_str(&val);
            }
            NodeKind::Macro => {
                let name = self.node_text(node);
                let expansion = self.evaluate_macro_call(node, &name)?;
                out.push_str(&expansion);
            }
            NodeKind::Composite | NodeKind::Block | NodeKind::Param => {
                for child in &node.parts {
                    let s = self.evaluate(child)?;
                    out.push_str(&s);
                }
            }
            NodeKind::LineComment | NodeKind::BlockComment => {}
            _ => {
                for child in &node.parts {
                    let s = self.evaluate(child)?;
                    out.push_str(&s);
                }
            }
        }
        Ok(out)
    }

    pub fn node_text(&self, node: &ASTNode) -> String {
        if let Some(source) = self.state.source_manager.get_source(node.token.src) {
            let start = node.token.pos;
            let end = node.token.pos + node.token.length;
            if end > source.len() || start > source.len() {
                eprintln!(
                    "node_text: out of range - start: {}, end: {}, source len: {}",
                    start,
                    end,
                    source.len()
                );
                return "".into();
            }

            let slice = match node.token.kind {
                TokenKind::BlockOpen | TokenKind::BlockClose | TokenKind::Macro => {
                    if end > start + 2 {
                        &source[(start + 1)..(end - 1)]
                    } else {
                        &source[start..end]
                    }
                }
                TokenKind::Var => {
                    if end > start + 3 {
                        &source[(start + 2)..(end - 1)]
                    } else {
                        &source[start..end]
                    }
                }
                TokenKind::Special => {
                    if end > start + 1 {
                        &source[start..(end - 1)]
                    } else {
                        &source[start..end]
                    }
                }
                _ => &source[start..end],
            };
            String::from_utf8_lossy(slice).to_string()
        } else {
            eprintln!("node_text: invalid src index");
            "".into()
        }
    }

    pub fn extract_name_value(&self, name_token: &Token) -> String {
        if let Some(source) = self.state.source_manager.get_source(name_token.src) {
            let start = name_token.pos;
            let end = name_token.pos + name_token.length;

            // Bounds checking
            if end > source.len() || start > source.len() {
                eprintln!(
                    "extract_name_value: out of range - start: {}, end: {}, source len: {}",
                    start,
                    end,
                    source.len()
                );
                return "".into();
            }

            // Since we know it's an Identifier, we can extract directly
            String::from_utf8_lossy(&source[start..end]).to_string()
        } else {
            eprintln!("extract_name_value: invalid src index");
            "".into()
        }
    }

    pub fn evaluate_macro_call(&mut self, node: &ASTNode, name: &str) -> EvalResult<String> {
        if let Some(bf) = self.builtins.get(name) {
            return bf(self, node);
        }

        if self.state.call_depth >= MAX_RECURSION_DEPTH {
            return Err(EvalError::Runtime(format!(
                "maximum recursion depth ({}) exceeded in macro '{}'",
                MAX_RECURSION_DEPTH, name
            )));
        }

        let mac = match self.state.get_macro(name) {
            Some(m) => m,
            None => return Err(EvalError::UndefinedMacro(name.into())),
        };

        let param_nodes: Vec<&ASTNode> = node
            .parts
            .iter()
            .filter(|p| p.kind == NodeKind::Param)
            .collect();

        self.state.push_scope();

        // frozen_args are vars that are not parameters
        // and get their values at definition site
        for (var, frozen_val) in mac.frozen_args.iter() {
            self.state.set_variable(var, frozen_val);
        }

        // Handle parameter assignment with support for both positional and named arguments
        for (i, param_name) in mac.params.iter().enumerate() {
            let val = if let Some(param_node) = param_nodes.get(i) {
                // If the parameter has a name in the AST node, use that as the parameter name
                if let Some(name_token) = &param_node.name {
                    // Extract the actual name from the token
                    let name_value = self.extract_name_value(name_token);

                    // Evaluate the parameter value
                    let evaluated = self.evaluate(param_node)?;

                    // Store the named parameter value
                    self.state.set_variable(&name_value, &evaluated);
                    continue; // Skip the positional assignment since we've handled it as named
                }

                // Otherwise, evaluate it as a positional parameter
                self.evaluate(param_node)?
            } else {
                "".to_string()
            };

            // Set the variable with the positional parameter name
            self.state.set_variable(param_name, &val);
        }

        self.state.call_depth += 1;
        let body_result = self.evaluate(&mac.body);
        self.state.call_depth -= 1;
        let mut result = body_result?;

        if mac.is_rhai {
            // Collect all visible variables (outer scopes first so inner ones win)
            let mut variables = std::collections::HashMap::new();
            for frame in self.state.scope_stack.iter() {
                for (k, v) in &frame.variables {
                    variables.insert(k.clone(), v.clone());
                }
            }
            result = self
                .rhai_evaluator
                .evaluate(&result, &variables, Some(&mac.name))
                .map_err(EvalError::Runtime)?;
        }

        self.state.pop_scope();

        Ok(result)
    }

    pub fn export(&mut self, name: &str) {
        let stack_len = self.state.scope_stack.len();
        if stack_len <= 1 {
            return;
        }
        let parent_index = stack_len - 2;

        if let Some(val) = self
            .state
            .scope_stack
            .last()
            .unwrap()
            .variables
            .get(name)
            .cloned()
        {
            self.state
                .scope_stack
                .get_mut(parent_index)
                .unwrap()
                .variables
                .insert(name.to_string(), val);
        }

        if let Some(mac) = self
            .state
            .scope_stack
            .last()
            .unwrap()
            .macros
            .get(name)
            .cloned()
        {
            let frozen_mac = self.freeze_macro_definition(&mac);
            self.state
                .scope_stack
                .get_mut(parent_index)
                .unwrap()
                .macros
                .insert(name.to_string(), frozen_mac);
        }
    }

    pub fn freeze_macro_definition(&mut self, mac: &MacroDefinition) -> MacroDefinition {
        let mut frozen = HashMap::new();
        let keep: HashSet<String> = mac.params.iter().cloned().collect();
        self.collect_freeze_vars(&mac.body, &keep, &mut frozen);

        MacroDefinition {
            name: mac.name.clone(),
            params: mac.params.clone(),
            body: Arc::clone(&mac.body),
            is_rhai: mac.is_rhai,
            frozen_args: frozen,
        }
    }

    fn collect_freeze_vars(
        &mut self,
        node: &ASTNode,
        keep: &HashSet<String>,
        frozen: &mut HashMap<String, String>,
    ) {
        if node.kind == NodeKind::Var {
            let var_name = self.node_text(node).trim().to_string();
            if !keep.contains(&var_name) && !frozen.contains_key(&var_name) {
                let value = self.evaluate(node).unwrap_or_default();
                frozen.insert(var_name, value);
            }
        }
        for child in &node.parts {
            self.collect_freeze_vars(child, keep, frozen);
        }
    }

    pub fn parse_string(&mut self, text: &str, path: &PathBuf) -> Result<ASTNode, EvalError> {
        let src = match fs::metadata(path) {
            Ok(md) if md.is_file() => self.add_source_if_not_present(path.clone())?,
            _ => self.add_source_bytes(text.as_bytes().to_vec(), path.clone()),
        };

        let result = crate::evaluator::lexer_parser::lex_parse_content(
            text,
            self.state.config.special_char,
            src,
        );
        result.map_err(EvalError::ParseError)
    }

    fn find_file(&self, filename: &str) -> EvalResult<PathBuf> {
        let p = Path::new(filename);
        if p.is_absolute() && p.exists() {
            return Ok(p.to_path_buf());
        }
        for inc in &self.state.config.include_paths {
            let candidate = inc.join(filename);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
        Err(EvalError::IncludeNotFound(filename.into()))
    }

    pub fn do_include(&mut self, filename: &str) -> EvalResult<String> {
        let path = self.find_file(filename)?;
        if self.state.open_includes.contains(&path) {
            return Err(EvalError::CircularInclude(path.display().to_string()));
        }
        self.state.open_includes.insert(path.clone());
        let result = (|| {
            let content = std::fs::read_to_string(&path)
                .map_err(|_| EvalError::IncludeNotFound(filename.into()))?;
            let ast = self.parse_string(&content, &path)?;
            self.evaluate(&ast)
        })();
        // Always remove the path, whether the include succeeded or failed,
        // so that a reused evaluator does not permanently block future includes.
        self.state.open_includes.remove(&path);
        result
    }

    pub fn num_source_files(&self) -> usize {
        self.state.source_manager.num_sources()
    }
}
