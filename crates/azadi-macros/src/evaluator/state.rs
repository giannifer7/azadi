// crates/azadi-macros/src/evaluator/state.rs

use super::python::PythonConfig;
use crate::types::ASTNode;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct EvalConfig {
    pub special_char: char,
    pub pydef: bool,
    pub include_paths: Vec<PathBuf>,
    pub backup_dir: PathBuf,
    pub python: PythonConfig,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            special_char: '%',
            pydef: false,
            include_paths: vec![PathBuf::from(".")],
            backup_dir: PathBuf::from("_azadi_work"),
            python: PythonConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MacroDefinition {
    pub name: String,
    pub params: Vec<String>,
    pub body: ASTNode,
    pub is_python: bool,
    pub frozen_args: HashMap<String, String>,
}

#[derive(Debug, Default, Clone)]
pub struct ScopeFrame {
    pub variables: HashMap<String, String>,
    pub macros: HashMap<String, MacroDefinition>,
}

pub struct SourceManager {
    source_files: Vec<Vec<u8>>,
    file_names: Vec<PathBuf>,
    sources_by_path: HashMap<PathBuf, usize>,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
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

    pub fn get_source(&self, src: i32) -> Option<&[u8]> {
        self.source_files.get(src as usize).map(|v| v.as_slice())
    }

    pub fn num_sources(&self) -> usize {
        self.source_files.len()
    }
}

pub struct EvaluatorState {
    pub config: EvalConfig,
    pub scope_stack: Vec<ScopeFrame>,
    pub open_includes: HashSet<PathBuf>,
    pub current_file: PathBuf,
    pub source_manager: SourceManager,
}

impl EvaluatorState {
    pub fn new(config: EvalConfig) -> Self {
        Self {
            config,
            scope_stack: vec![ScopeFrame::default()],
            open_includes: HashSet::new(),
            current_file: PathBuf::from(""),
            source_manager: SourceManager::new(),
        }
    }

    pub fn push_scope(&mut self) {
        self.scope_stack.push(ScopeFrame::default());
    }

    pub fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    pub fn current_scope(&self) -> &ScopeFrame {
        self.scope_stack.last().unwrap()
    }

    pub fn current_scope_mut(&mut self) -> &mut ScopeFrame {
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

    pub fn get_special_char(&self) -> Vec<u8> {
        vec![self.config.special_char as u8]
    }
}
