use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use super::builtins::{default_builtins, BuiltinFn};
use crate::types::{ASTNode, NodeKind};

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("Undefined macro: {0}")]
    UndefinedMacro(String),

    #[error("Builtin error: {0}")]
    BuiltinError(String),

    #[error("Include not found: {0}")]
    IncludeNotFound(String),

    #[error("Circular include: {0}")]
    CircularInclude(String),

    #[error("Invalid usage: {0}")]
    InvalidUsage(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type EvalResult<T> = Result<T, EvalError>;

impl From<String> for EvalError {
    fn from(s: String) -> Self {
        EvalError::Runtime(s)
    }
}

#[derive(Debug, Clone)]
pub struct MacroDefinition {
    pub name: String,
    pub params: Vec<String>,
    pub body: ASTNode,
    pub is_python: bool,
}

#[derive(Debug, Default, Clone)]
pub struct ScopeFrame {
    pub variables: HashMap<String, String>,
    pub macros: HashMap<String, MacroDefinition>,
}

#[derive(Debug, Clone)]
pub struct EvalConfig {
    pub special_char: char,
    pub pydef: bool,
    pub include_paths: Vec<PathBuf>,
    pub backup_dir: PathBuf,
}

impl Default for EvalConfig {
    fn default() -> Self {
        EvalConfig {
            special_char: '%',
            pydef: false,
            include_paths: vec![PathBuf::from(".")],
            backup_dir: PathBuf::from("_azadi_work"),
        }
    }
}

#[derive(Debug)]
pub struct Evaluator {
    config: EvalConfig,
    scope_stack: Vec<ScopeFrame>,
    builtins: HashMap<String, BuiltinFn>,
    open_includes: HashSet<PathBuf>,
    current_file: PathBuf,
    source_files: Vec<Vec<u8>>,
    file_names: Vec<PathBuf>,
    sources_by_path: HashMap<PathBuf, usize>,
}

impl Evaluator {
    pub fn new(config: EvalConfig) -> Self {
        Evaluator {
            config,
            scope_stack: vec![ScopeFrame::default()],
            builtins: default_builtins(),
            open_includes: HashSet::new(),
            current_file: PathBuf::from(""),
            source_files: Vec::new(),
            file_names: Vec::new(),
            sources_by_path: HashMap::new(),
        }
    }

    pub fn add_source_if_not_present(&mut self, file_path: PathBuf) -> Result<i32, std::io::Error> {
        let file_path = file_path.canonicalize()?;
        if let Some(&src) = self.sources_by_path.get(&file_path) {
            return Ok(src as i32);
        }
        let content = std::fs::read(file_path.clone())?;
        let src = self.add_source_bytes(content, file_path.clone());
        Ok(src)
    }

    pub fn add_source_bytes(&mut self, content: Vec<u8>, path: PathBuf) -> i32 {
        let index = self.source_files.len();
        self.source_files.push(content);
        self.file_names.push(path.clone());
        self.sources_by_path.insert(path, index);
        index as i32
    }

    pub fn set_current_file(&mut self, file: PathBuf) {
        self.current_file = file;
    }

    pub fn get_current_file_path(&self) -> PathBuf {
        self.current_file.clone()
    }

    pub fn get_backup_dir_path(&self) -> PathBuf {
        self.config.backup_dir.clone()
    }

    pub fn evaluate(&mut self, node: &ASTNode) -> EvalResult<String> {
        let mut out = String::new();
        match node.kind {
            NodeKind::Text | NodeKind::Space | NodeKind::Ident => {
                let txt = self.node_text(node);
                out.push_str(&txt);
            }
            NodeKind::Var => {
                let var_name = self.node_text(node);
                let val = self.get_variable(&var_name);
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
        let src_idx = node.token.src as usize;
        if src_idx >= self.source_files.len() {
            println!("node_text: invalid src index");
            return "".into();
        }
        let source = &self.source_files[src_idx];
        let start = node.token.pos;
        let end = node.token.pos + node.token.length;
        if end > source.len() || start >= source.len() {
            println!("node_text: out of range");
            return "".into();
        }
        use crate::types::TokenKind::*;
        let slice = match node.token.kind {
            BlockOpen | BlockClose | Macro => {
                if end > start + 2 {
                    &source[(start + 1)..(end - 1)]
                } else {
                    &source[start..end]
                }
            }
            Var => {
                if end > start + 3 {
                    &source[(start + 2)..(end - 1)]
                } else {
                    &source[start..end]
                }
            }
            _ => &source[start..end],
        };
        String::from_utf8_lossy(slice).to_string()
    }

    pub fn evaluate_macro_call(&mut self, node: &ASTNode, name: &str) -> EvalResult<String> {
        if let Some(bf) = self.builtins.get(name) {
            return bf(self, node);
        }
        let mac = match self.get_macro(name) {
            Some(m) => m,
            None => return Err(EvalError::UndefinedMacro(name.into())),
        };
        let param_nodes: Vec<&ASTNode> = node
            .parts
            .iter()
            .filter(|p| p.kind == NodeKind::Param)
            .collect();
        self.push_scope();
        for (i, param_name) in mac.params.iter().enumerate() {
            let val = if let Some(param_node) = param_nodes.get(i) {
                self.evaluate(param_node)?
            } else {
                "".to_string()
            };
            self.set_variable(param_name, &val);
        }
        let out = self.evaluate(&mac.body)?;
        self.pop_scope();
        if mac.is_python {}
        Ok(out)
    }

    fn push_scope(&mut self) {
        self.scope_stack.push(ScopeFrame::default());
    }

    fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    fn current_scope_mut(&mut self) -> &mut ScopeFrame {
        self.scope_stack.last_mut().unwrap()
    }

    pub fn set_variable(&mut self, name: &str, value: &str) {
        self.current_scope_mut()
            .variables
            .insert(name.into(), value.into());
    }

    pub fn get_variable(&self, name: &str) -> String {
        for frame in self.scope_stack.iter().rev() {
            if let Some(val) = frame.variables.get(name) {
                return val.clone();
            }
        }
        "".to_string()
    }

    pub fn define_macro(&mut self, mac: MacroDefinition) {
        self.current_scope_mut()
            .macros
            .insert(mac.name.clone(), mac);
    }

    pub fn get_macro(&self, name: &str) -> Option<MacroDefinition> {
        for frame in self.scope_stack.iter().rev() {
            if let Some(m) = frame.macros.get(name) {
                return Some(m.clone());
            }
        }
        None
    }

    /*
    pub fn parse_string(&mut self, text: &str, path: &PathBuf) -> Result<ASTNode, EvalError> {
        let src = self
            .add_source_if_not_present(path.clone())
            .map_err(|e| EvalError::IoError(e))?;
        crate::evaluator::lexer_parser::lex_parse_content(
            text,
            self.config.special_char,
            src as i32,
        )
        .map_err(|e| EvalError::ParseError(e))
    }
    */

    pub fn parse_string(&mut self, text: &str, path: &PathBuf) -> Result<ASTNode, EvalError> {
        println!(
            "parse_string: called with text: {:?}, path: {:?}",
            text, path
        );
        let src = match fs::metadata(path) {
            Ok(md) if md.is_file() => {
                // The file actually exists -> read from disk
                self.add_source_if_not_present(path.clone())?
            }
            _ => {
                // File does not exist: store the in-memory string
                self.add_source_bytes(text.as_bytes().to_vec(), path.clone())
            }
        };
        println!("parse_string: added source, src index: {}", src);
        let result = crate::evaluator::lexer_parser::lex_parse_content(
            text,
            self.config.special_char,
            src as i32,
        );
        println!("parse_string: lex_parse_content result: {:?}", result);
        result.map_err(|e| EvalError::ParseError(e))
    }

    fn find_file(&self, filename: &str) -> EvalResult<PathBuf> {
        let p = Path::new(filename);
        if p.is_absolute() && p.exists() {
            return Ok(p.to_path_buf());
        }
        for inc in &self.config.include_paths {
            let candidate = inc.join(filename);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
        Err(EvalError::IncludeNotFound(filename.into()))
    }

    pub fn do_include(&mut self, filename: &str) -> EvalResult<String> {
        let path = self.find_file(filename)?;
        if self.open_includes.contains(&path) {
            return Err(EvalError::CircularInclude(path.display().to_string()));
        }
        self.open_includes.insert(path.clone());
        let content = std::fs::read_to_string(&path)
            .map_err(|_| EvalError::IncludeNotFound(filename.into()))?;
        let ast = self.parse_string(&content, &path)?;
        let out = self.evaluate(&ast)?;
        self.open_includes.remove(&path);
        Ok(out)
    }

    pub fn num_source_files(&self) -> usize {
        self.source_files.len()
    }
}
