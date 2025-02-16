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
    use lazy_static::lazy_static;
    use pyo3::prelude::*;
    use pyo3::types::{PyAny, PyDict};
    use std::ffi::CString;
    use std::sync::Mutex;

    lazy_static! {
        static ref SHARED_CONTEXT: Mutex<Option<Py<PyAny>>> = Mutex::new(None);
    }

    pub struct PyO3Evaluator {
        config: PythonConfig,
        current_source: Option<String>,
        current_macro: Option<String>,
        current_line: Option<usize>,
    }

    impl PyO3Evaluator {
        pub fn new(config: PythonConfig) -> PyResult<Self> {
            Python::with_gil(|py| {
                let mut global = SHARED_CONTEXT.lock().unwrap();
                if global.is_none() {
                    let setup_code = r#"
try:
    from munch import Munch
except ImportError:
    try:
        import pip
        pip.main(['install', 'munch'])
        from munch import Munch
    except Exception as e:
        raise ImportError(f"Could not import or install munch: {e}")

shared_context = Munch()
"#;
                    // Convert Rust &str to a C string for py.run
                    let code_cstr = CString::new(setup_code).map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e))
                    })?;

                    let locals = PyDict::new(py);
                    py.run(code_cstr.as_c_str(), None, Some(&locals))?;

                    // get_item(...) returns Result<Option<Bound<'_, PyAny>>, PyErr> in your env
                    let maybe_obj = locals.get_item("shared_context");
                    let shared_context = match maybe_obj {
                        Ok(Some(obj)) => obj,
                        Ok(None) => {
                            return Err(pyo3::exceptions::PyKeyError::new_err(
                                "shared_context not found",
                            )
                            .into())
                        }
                        Err(err) => return Err(err.into()),
                    };

                    // Instead of .into_pyobject(py), we do .extract::<Py<PyAny>>()
                    let py_obj = shared_context.extract::<Py<PyAny>>()?;
                    *global = Some(py_obj);
                }

                Ok(Self {
                    config,
                    current_source: None,
                    current_macro: None,
                    current_line: None,
                })
            })
        }

        fn eval_with_context(
            &self,
            py: Python,
            code: &str,
            locals: &Bound<'_, PyDict>,
        ) -> PyResult<String> {
            let guard = SHARED_CONTEXT.lock().unwrap();
            let shared_ctx = guard.as_ref().expect("shared_context not set");

            // Insert the shared_context object into locals
            locals.set_item("shared_context", shared_ctx)?;

            // Capture stdout
            let io = py.import("io")?;
            let sys = py.import("sys")?;
            let string_io = io.getattr("StringIO")?.call0()?;
            let old_stdout = sys.getattr("stdout")?;
            sys.setattr("stdout", &string_io)?;

            // Convert user code to a C string
            let code_cstr = CString::new(code)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
            py.run(code_cstr.as_c_str(), None, Some(locals))?;

            // Restore stdout
            sys.setattr("stdout", &old_stdout)?;
            let output = string_io.call_method0("getvalue")?.extract::<String>()?;

            if !sys.getattr("stdout")?.is(&old_stdout) {
                sys.setattr("stdout", &old_stdout)?;
            }

            Ok(output)
        }
    }

    impl From<PyErr> for PyEvalError {
        fn from(err: PyErr) -> Self {
            PyEvalError::Execution(format!("Python error: {}", err))
        }
    }

    impl PythonEvaluator for PyO3Evaluator {
        fn evaluate(
            &self,
            code: &str,
            variables: std::collections::HashMap<String, String>,
        ) -> Result<String, PyEvalError> {
            Python::with_gil(|py| {
                let locals = PyDict::new(py);
                for (k, v) in variables {
                    locals.set_item(k, v)?;
                }

                let output = self.eval_with_context(py, code, &locals)?;
                Ok(output)
            })
        }

        fn validate_code(
            &self,
            _code: &str,
            _security_level: SecurityLevel,
        ) -> Result<(), PyEvalError> {
            Ok(())
        }
    }
}
