use crate::evaluator::errors::PyEvalError;
use pyo3::prelude::*; // Basic PyO3 items (Python, PyResult, etc.)
use pyo3::types::PyModule; // for PyModule::from_code
use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct PythonConfig {
    pub enabled: bool,
    pub venv_path: Option<PathBuf>,
    pub python_path: Option<PathBuf>,
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            venv_path: None,
            python_path: None,
        }
    }
}

pub trait PythonEvaluator: Send + Sync {
    fn evaluate(
        &self,
        code: &str,
        variables: HashMap<String, String>,
    ) -> Result<String, PyEvalError>;

    fn evaluate_with_context(
        &self,
        code: &str,
        variables: HashMap<String, String>,
        name: Option<&str>,
        filename: Option<&str>,
        line: Option<u32>,
    ) -> Result<String, PyEvalError>;
}

/// Just storing config or any global data you like. We won't keep a 'globals' PyDict in this version.
struct PythonState {
    work_dir: Option<PathBuf>,
}

/// Single lazy `PythonState`.
static PYTHON_STATE: OnceLock<Mutex<PythonState>> = OnceLock::new();

impl PythonState {
    fn new() -> Self {
        Self { work_dir: None }
    }

    fn set_work_directory(&mut self, path: Option<PathBuf>) {
        self.work_dir = path;
        if let Some(ref d) = self.work_dir {
            let py_dir = d.join("python");
            if !py_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&py_dir) {
                    eprintln!("Warning: Failed to create Python work directory: {}", e);
                }
            }
        }
    }

    /// If logging is enabled, write `code` to a .py file.
    fn log_code(
        &self,
        code: &str,
        name: Option<&str>,
        filename: Option<&str>,
        line: Option<u32>,
    ) -> Result<(), std::io::Error> {
        if let Some(ref base) = self.work_dir {
            let py_dir = base.join("python");
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();

            let log_filename = match (name, filename, line) {
                (Some(n), Some(f), Some(l)) => {
                    format!("{}_{}_L{}_{}.py", n, f.replace("/", "_"), l, timestamp)
                }
                (Some(n), Some(f), None) => {
                    format!("{}_{}.py", n, f.replace("/", "_"))
                }
                (Some(n), None, None) => format!("{}_{}.py", n, timestamp),
                _ => format!("python_code_{}.py", timestamp),
            };

            let mut file_content = String::new();
            file_content.push_str("# Python code generated from:\n");
            if let Some(f) = filename {
                file_content.push_str(&format!("# File: {}\n", f));
            }
            if let Some(l) = line {
                file_content.push_str(&format!("# Line: {}\n", l));
            }
            if let Some(n) = name {
                file_content.push_str(&format!("# Block: {}\n", n));
            }
            file_content.push_str(&format!("# Timestamp: {}\n\n{}", timestamp, code));

            let out_path = py_dir.join(log_filename);
            fs::write(out_path, file_content)?;
        }
        Ok(())
    }
}

fn get_python_state() -> &'static Mutex<PythonState> {
    PYTHON_STATE.get_or_init(|| Mutex::new(PythonState::new()))
}

/// Minimal evaluator that only does "single-pass" code execution via `PyModule::from_code`.
pub struct PyO3Evaluator {
    config: PythonConfig,
}

impl PyO3Evaluator {
    pub fn new(config: PythonConfig) -> PyResult<Self> {
        let me = Self { config };
        // If you want to set up a virtualenv or do other steps, do so here.
        if let Some(ref venv) = me.config.venv_path {
            me.setup_virtualenv(venv)?;
        }
        Ok(me)
    }

    pub fn set_work_directory(&self, path: PathBuf) {
        let mut st = get_python_state().lock().unwrap();
        st.set_work_directory(Some(path));
    }

    /// If you want to unify a virtualenv site-packages, do so here
    fn setup_virtualenv(&self, _venv_path: &PathBuf) -> PyResult<()> {
        // ... same logic you had before ...
        Ok(())
    }

    /// Actually run user code in a single multi-line snippet that also captures stdout.
    /// We do it all at once with `PyModule::from_code(...)`.
    fn eval_with_context(
        &self,
        user_code: &str,
        variables: &HashMap<String, String>,
        name: Option<&str>,
        filename: Option<&str>,
        line: Option<u32>,
    ) -> PyResult<String> {
        // Possibly log to file
        {
            let st = get_python_state().lock().unwrap();
            if let Err(e) = st.log_code(user_code, name, filename, line) {
                eprintln!("Warn: cannot log python code: {}", e);
            }
        }

        Python::attach(|py| {
            // We'll build a Python snippet that:
            //   1) sets up a buffer for stdout
            //   2) defines or sets each variable
            //   3) runs the user's code
            //   4) restores stdout
            //   5) stashes the result in "__captured"

            let mut snippet = String::new();
            snippet.push_str(
                r#"
import sys, io

old_stdout = sys.stdout
_buf = io.StringIO()
sys.stdout = _buf
"#,
            );

            // Insert each variable. We can just define them at top-level:
            for (k, v) in variables {
                // naive approach: do var = repr(v)
                snippet.push_str(&format!("{k} = {val:?}\n", val = v));
            }

            // Insert user code verbatim
            snippet.push_str("\n# === user code ===\n");
            snippet.push_str(user_code);
            snippet.push_str("\n# === end user code ===\n");

            // restore stdout
            snippet.push_str(
                r#"
sys.stdout = old_stdout
__captured = _buf.getvalue()
"#,
            );

            // Now we compile+run this snippet into a new module
            let c_code = CString::new(snippet).unwrap();
            let c_filename = CString::new("in_memory.py").unwrap();
            let c_modname = CString::new("temp_module").unwrap();

            let module = PyModule::from_code(py, &c_code, c_filename.as_ref(), c_modname.as_ref())?;

            // Finally, read __captured if present
            if let Ok(captured_any) = module.getattr("__captured") {
                let output: String = captured_any.extract()?;
                Ok(output)
            } else {
                Ok(String::new())
            }
        })
    }
}

impl PythonEvaluator for PyO3Evaluator {
    fn evaluate(
        &self,
        code: &str,
        variables: HashMap<String, String>,
    ) -> Result<String, PyEvalError> {
        self.eval_with_context(code, &variables, None, None, None)
            .map_err(|e| PyEvalError::Execution(format!("Python error: {}", e)))
    }

    fn evaluate_with_context(
        &self,
        code: &str,
        variables: HashMap<String, String>,
        name: Option<&str>,
        filename: Option<&str>,
        line: Option<u32>,
    ) -> Result<String, PyEvalError> {
        self.eval_with_context(code, &variables, name, filename, line)
            .map_err(|e| PyEvalError::Execution(format!("Python error: {}", e)))
    }
}
