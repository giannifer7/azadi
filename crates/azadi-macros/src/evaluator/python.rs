// crates/azadi-macros/src/evaluator/python.rs

use super::errors::PyEvalError;
use clap::ValueEnum;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum SecurityLevel {
    None,
    Basic,
    Strict,
}

#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum PyBackend {
    None,
    Subprocess,
    #[cfg(feature = "pyo3")]
    PyO3,
}

impl std::fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityLevel::None => write!(f, "none"),
            SecurityLevel::Basic => write!(f, "basic"),
            SecurityLevel::Strict => write!(f, "strict"),
        }
    }
}

impl std::fmt::Display for PyBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PyBackend::None => write!(f, "none"),
            PyBackend::Subprocess => write!(f, "subprocess"),
            #[cfg(feature = "pyo3")]
            PyBackend::PyO3 => write!(f, "pyo3"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PythonConfig {
    pub enabled: bool,
    pub venv_path: Option<PathBuf>,
    pub python_path: Option<PathBuf>,
    pub security_level: SecurityLevel,
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            venv_path: None,
            python_path: None,
            security_level: SecurityLevel::Basic,
        }
    }
}

pub trait PythonEvaluator: Send + Sync {
    fn validate_code(&self, code: &str, security_level: SecurityLevel) -> Result<(), PyEvalError>;
    fn evaluate(
        &self,
        code: &str,
        variables: HashMap<String, String>,
    ) -> Result<String, PyEvalError>;
}

pub struct SubprocessEvaluator {
    config: PythonConfig,
}

impl SubprocessEvaluator {
    pub fn new(config: PythonConfig) -> Self {
        Self { config }
    }

    fn create_execution_script(
        &self,
        code: &str,
        variables: &HashMap<String, String>,
    ) -> Result<NamedTempFile, PyEvalError> {
        let mut script = NamedTempFile::new()
            .map_err(|e| PyEvalError::Environment(format!("Failed to create temp file: {}", e)))?;

        // Write output capture setup
        writeln!(script, "import sys, io")?;
        writeln!(script, "_output = io.StringIO()")?;
        writeln!(script, "sys.stdout = _output")?;

        // Write error handling
        writeln!(script, "try:")?;

        // Write variable definitions with indentation
        for (name, value) in variables {
            writeln!(script, "    {} = \"{}\"", name, value)?;
        }

        // Write the actual code with indentation
        for line in code.lines() {
            writeln!(script, "    {}", line)?;
        }

        // Write error handling and output management
        writeln!(script, "except Exception as e:")?;
        writeln!(script, "    print(f'Python error: {{str(e)}}')")?;
        writeln!(script, "finally:")?;
        writeln!(script, "    sys.stdout = sys.__stdout__")?;
        writeln!(script, "    print(_output.getvalue(), end='')")?;

        script.flush()?;
        Ok(script)
    }
}

impl PythonEvaluator for SubprocessEvaluator {
    fn validate_code(&self, code: &str, security_level: SecurityLevel) -> Result<(), PyEvalError> {
        match security_level {
            SecurityLevel::None => Ok(()),
            SecurityLevel::Basic => {
                let forbidden = ["os.system", "subprocess", "exec", "eval", "open", "file"];
                for term in forbidden {
                    if code.contains(term) {
                        return Err(PyEvalError::Security(format!(
                            "Forbidden term found: {}",
                            term
                        )));
                    }
                }
                Ok(())
            }
            SecurityLevel::Strict => {
                let forbidden = [
                    "os.",
                    "sys.",
                    "subprocess",
                    "exec",
                    "eval",
                    "open",
                    "file",
                    "__import__",
                    "importlib",
                ];
                for term in forbidden {
                    if code.contains(term) {
                        return Err(PyEvalError::Security(format!(
                            "Forbidden term found: {}",
                            term
                        )));
                    }
                }
                Ok(())
            }
        }
    }

    fn evaluate(
        &self,
        code: &str,
        variables: HashMap<String, String>,
    ) -> Result<String, PyEvalError> {
        // First validate
        self.validate_code(code, self.config.security_level)?;

        // Create the execution script
        let script = self.create_execution_script(code, &variables)?;

        // Build command with virtual env configuration
        let python_path = self
            .config
            .python_path
            .as_ref()
            .map(|p| p.as_path())
            .unwrap_or_else(|| Path::new("python3"));

        let mut cmd = Command::new(python_path);

        // Configure virtual environment if specified
        if let Some(venv) = &self.config.venv_path {
            if let Some(venv_str) = venv.to_str() {
                cmd.env("VIRTUAL_ENV", venv_str);

                #[cfg(unix)]
                {
                    let path = format!(
                        "{}/bin:{}",
                        venv_str,
                        std::env::var("PATH").unwrap_or_default()
                    );
                    cmd.env("PATH", path);
                }
                #[cfg(windows)]
                {
                    let path = format!(
                        "{}/Scripts;{}",
                        venv_str,
                        std::env::var("PATH").unwrap_or_default()
                    );
                    cmd.env("PATH", path);
                }
            }
        }

        // Execute and capture output/errors
        let output = cmd
            .arg(script.path())
            .output()
            .map_err(|e| PyEvalError::Execution(format!("Failed to execute Python: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if !output.status.success() {
            return Err(PyEvalError::Execution(format!(
                "Python execution failed: {}\nOutput: {}",
                stderr, stdout
            )));
        }

        // If there's stderr output but the process succeeded, it might be warnings
        if !stderr.is_empty() {
            eprintln!("Python warnings: {}", stderr);
        }

        Ok(stdout.into_owned())
    }
}

#[cfg(feature = "pyo3")]
pub mod pyo3_evaluator {
    use super::*;
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyList};
    use std::ffi::CString;

    pub struct PyO3Evaluator {
        config: PythonConfig,
    }

    impl PyO3Evaluator {
        pub fn new(config: PythonConfig) -> Self {
            Self { config }
        }
    }

    impl PythonEvaluator for PyO3Evaluator {
        fn validate_code(
            &self,
            code: &str,
            security_level: SecurityLevel,
        ) -> Result<(), PyEvalError> {
            // Use the same validation as subprocess
            let subprocess = SubprocessEvaluator::new(self.config.clone());
            subprocess.validate_code(code, security_level)
        }

        fn evaluate(
            &self,
            code: &str,
            variables: HashMap<String, String>,
        ) -> Result<String, PyEvalError> {
            Python::with_gil(|py| -> Result<String, PyEvalError> {
                // Create output sink list
                let sink = PyList::empty(py);

                // Create locals dict with variables
                let locals = PyDict::new(py);
                for (k, v) in variables {
                    locals.set_item(k, v).map_err(|e| {
                        PyEvalError::Execution(format!("Failed to set variable: {}", e))
                    })?;
                }

                // Add sink to locals - note we borrow the sink here
                locals
                    .set_item("_output_sink", &sink)
                    .map_err(|e| PyEvalError::Execution(format!("Failed to set sink: {}", e)))?;

                // Convert Python setup code to CString
                let setup_code = CString::new(
                    r#"
class _Write:
    def __call__(self, s):
        _output_sink.append(str(s))
class _WriteLine(_Write):
    def __call__(self, s):
        _output_sink.extend([str(s), '\n'])
write = _Write()
print = _WriteLine()
                "#,
                )
                .map_err(|e| {
                    PyEvalError::Execution(format!("Failed to create setup code: {}", e))
                })?;

                // Convert main code to CString
                let main_code = CString::new(code).map_err(|e| {
                    PyEvalError::Execution(format!("Failed to create main code: {}", e))
                })?;

                // Run setup code
                py.run(&setup_code, None, Some(&locals)).map_err(|e| {
                    PyEvalError::Execution(format!("Failed to run setup code: {}", e))
                })?;

                // Run main code
                py.run(&main_code, None, Some(&locals)).map_err(|e| {
                    PyEvalError::Execution(format!("Failed to run main code: {}", e))
                })?;

                // Collect output from sink
                let mut result = String::new();

                // Now we can use sink since we only borrowed it earlier
                for item in sink.iter() {
                    let item_str: String = item.extract().map_err(|e| {
                        PyEvalError::Execution(format!("Failed to extract string: {}", e))
                    })?;
                    result.push_str(&item_str);
                }

                Ok(result)
            })
        }
    }
}
