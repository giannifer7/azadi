// src/pipeline/error_macros.rs

#[macro_export]
macro_rules! try_read {
    ($path:expr) => {
        std::fs::read_to_string($path).map_err(|e| PipelineError::ReadError {
            path: $path.to_path_buf(),
            source: e,
            backtrace: Backtrace::capture(),
        })
    };
}

#[macro_export]
macro_rules! try_write {
    ($path:expr, $content:expr) => {{
        let path = $path;
        // Ensure the parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| PipelineError::CreateDirError {
                path: parent.to_path_buf(),
                source: e,
                backtrace: Backtrace::capture(),
            })?;
        }
        std::fs::write(path, $content).map_err(|e| PipelineError::WriteError {
            path: path.to_path_buf(),
            source: e,
            backtrace: Backtrace::capture(),
        })
    }};
}

#[macro_export]
macro_rules! try_mkdir {
    ($path:expr) => {
        std::fs::create_dir_all($path).map_err(|e| PipelineError::CreateDirError {
            path: $path.to_path_buf(),
            source: e,
            backtrace: Backtrace::capture(),
        })
    };
}

#[macro_export]
macro_rules! try_macro {
    ($result:expr) => {
        $result.map_err(|e| PipelineError::MacroError {
            source: e,
            backtrace: Backtrace::capture(),
        })
    };
}

#[macro_export]
macro_rules! try_noweb {
    ($result:expr) => {
        $result.map_err(|e| PipelineError::NowebError {
            source: e,
            backtrace: Backtrace::capture(),
        })
    };
}
