// crates/azadi-macros/src/evaluator/python.rs

use super::errors::PyEvalError;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use tempfile::NamedTempFile;

#[derive(Debug, Clone, Copy)]
pub enum SecurityLevel {
    None,
    Basic,
    Strict,
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
            Python::with_gil(|py| {
                // Create sink list
                let sink = PyList::empty(py);

                // Create locals dict with variables
                let locals = PyDict::new(py);
                for (k, v) in variables {
                    locals.set_item(k, v)?;
                }

                // Add sink and print/write functions
                locals.set_item("sink", sink)?;

                // Define print/write functions
                py.run(
                    r#"
class Write:
    def __call__(self, s):
        sink.append(str(s))
class WriteLine(Write):
    def __call__(self, s):
        sink.extend([str(s), '\n'])
write = Write()
print = WriteLine()
                "#,
                    None,
                    Some(locals),
                )?;

                // Execute the code
                py.run(code, None, Some(locals))?;

                // Get output from sink
                let result: String = sink
                    .iter()?
                    .map(|item| item.extract::<String>())
                    .collect::<PyResult<Vec<_>>>()?
                    .join("");

                Ok(result)
            })
            .map_err(|e| PyEvalError::Execution(format!("PyO3 error: {}", e)))
        }
    }
}
