use chrono::{DateTime, Local};
use std::fs::{self, File};
use std::io::Read;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug)]
pub enum SafeWriterError {
    IoError(io::Error),
    DirectoryCreationFailed(PathBuf),
    BackupFailed(PathBuf),
    ModifiedExternally(PathBuf),
    SecurityViolation(String),
}

impl std::fmt::Display for SafeWriterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SafeWriterError::IoError(e) => write!(f, "IO error: {}", e),
            SafeWriterError::DirectoryCreationFailed(path) => {
                write!(f, "Failed to create directory: {}", path.display())
            }
            SafeWriterError::BackupFailed(path) => {
                write!(f, "Failed to create backup for: {}", path.display())
            }
            SafeWriterError::ModifiedExternally(path) => {
                write!(f, "File was modified externally: {}", path.display())
            }
            SafeWriterError::SecurityViolation(msg) => write!(f, "Security violation: {}", msg),
        }
    }
}

impl std::error::Error for SafeWriterError {}

impl From<io::Error> for SafeWriterError {
    fn from(err: io::Error) -> Self {
        SafeWriterError::IoError(err)
    }
}

#[derive(Debug, Clone)]
pub struct SafeWriterConfig {
    pub backup_enabled: bool,
    pub allow_overwrites: bool,
    pub modification_check: bool,
    pub buffer_size: usize,
}

impl Default for SafeWriterConfig {
    fn default() -> Self {
        SafeWriterConfig {
            backup_enabled: true,
            allow_overwrites: false,
            modification_check: true,
            buffer_size: 8192,
        }
    }
}

pub struct SafeFileWriter {
    gen_base: PathBuf,
    private_dir: PathBuf,
    old_dir: PathBuf,
    old_timestamp: Option<DateTime<Local>>,
    config: SafeWriterConfig,
}

impl SafeFileWriter {
    pub fn new<P: AsRef<Path>>(gen_base: P, private_dir: P) -> Self {
        Self::with_config(gen_base, private_dir, SafeWriterConfig::default())
    }

    pub fn with_config<P: AsRef<Path>>(
        gen_base: P,
        private_dir: P,
        config: SafeWriterConfig,
    ) -> Self {
        let (gen_base, private_dir) = Self::canonicalize_paths(&gen_base, &private_dir)
            .expect("Failed to initialize directories");
        let old_dir = private_dir.join("__old__");

        // Create all required directories
        fs::create_dir_all(&gen_base).expect("Failed to create gen_base directory");
        fs::create_dir_all(&private_dir).expect("Failed to create private directory");
        fs::create_dir_all(&old_dir).expect("Failed to create old directory");

        SafeFileWriter {
            gen_base,
            private_dir,
            old_dir,
            old_timestamp: None,
            config,
        }
    }

    fn canonicalize_paths<P: AsRef<Path>>(
        gen_base: P,
        private_dir: P,
    ) -> io::Result<(PathBuf, PathBuf)> {
        // Ensure directories exist before canonicalizing.
        fs::create_dir_all(gen_base.as_ref())?;
        fs::create_dir_all(private_dir.as_ref())?;

        let gen = gen_base.as_ref().canonicalize()?;
        let private = private_dir.as_ref().canonicalize()?;

        Ok((gen, private))
    }

    fn atomic_copy<P: AsRef<Path>>(&self, source: P, destination: P) -> io::Result<()> {
        let temp_path = destination.as_ref().with_extension("tmp");
        
        // Ensure temp file is removed if it exists
        if temp_path.exists() {
            let _ = fs::remove_file(&temp_path);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        {
            let mut source_file = fs::File::open(&source)?;
            let mut temp_file = fs::File::create(&temp_path)?;
            io::copy(&mut source_file, &mut temp_file)?;
            temp_file.sync_all()?;
        } // Handles are dropped here
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::rename(temp_path, destination)?;
        Ok(())
    }
    
    fn copy_if_different<P: AsRef<Path>>(
        &self,
        source: P,
        destination: P,
    ) -> Result<(), SafeWriterError> {
        let source = source.as_ref();
        let destination = destination.as_ref();
    
        if !destination.exists() {
            return self
                .atomic_copy(source, destination)
                .map_err(SafeWriterError::from);
        }
    
        let are_different = {
            let mut source_file = BufReader::with_capacity(self.config.buffer_size, File::open(source)?);
            let mut dest_file = BufReader::with_capacity(self.config.buffer_size, File::open(destination)?);
    
            let mut source_content = Vec::new();
            let mut dest_content = Vec::new();
    
            source_file.read_to_end(&mut source_content)?;
            dest_file.read_to_end(&mut dest_content)?;
    
            source_content != dest_content
        }; // Handles are dropped here
    
        if are_different {
            println!("file {} changed", destination.display());
            std::thread::sleep(std::time::Duration::from_millis(10)); // Allow Windows to release handles
            self.atomic_copy(source, destination)?;
        }
    
        Ok(())
    }

    fn prepare_write_file<P: AsRef<Path>>(&self, file_path: P) -> Result<PathBuf, SafeWriterError> {
        let path = file_path.as_ref();
        let dest_dir = path.parent().unwrap_or_else(|| Path::new(""));

        // Create all necessary directories
        let dirs = [
            self.gen_base.join(dest_dir),
            self.old_dir.join(dest_dir),
            self.private_dir.join(dest_dir),
        ];

        for dir in &dirs {
            fs::create_dir_all(dir)
                .map_err(|_| SafeWriterError::DirectoryCreationFailed(dir.clone()))?;
        }

        Ok(path.to_path_buf())
    }

    pub fn before_write<P: AsRef<Path>>(
        &mut self,
        file_name: P,
    ) -> Result<PathBuf, SafeWriterError> {
        validate_filename(file_name.as_ref())?;
        let path = self.prepare_write_file(&file_name)?;

        if self.config.backup_enabled {
            let old_file_name = self.old_dir.join(&path);
            if old_file_name.is_file() {
                let metadata = fs::metadata(&old_file_name)?;
                let system_time: SystemTime = metadata.modified()?;
                self.old_timestamp = Some(DateTime::from(system_time));
            } else {
                self.old_timestamp = None;
            }
        }

        Ok(self.private_dir.join(path))
    }

    pub fn after_write<P: AsRef<Path>>(&self, file_name: P) -> Result<(), SafeWriterError> {
        validate_filename(file_name.as_ref())?;
        let path = self.prepare_write_file(file_name)?;

        let private_file = self.private_dir.join(&path);
        let output_file = self.gen_base.join(&path);
        let old_file = self.old_dir.join(&path);

        // Create backup if enabled
        if self.config.backup_enabled {
            self.atomic_copy(&private_file, &old_file)
                .map_err(|_| SafeWriterError::BackupFailed(old_file.clone()))?;
        }

        if self.config.modification_check && output_file.is_file() {
            let system_time: SystemTime = fs::metadata(&output_file)?.modified()?;
            let out_timestamp: DateTime<Local> = DateTime::from(system_time);

            if let Some(old_timestamp) = self.old_timestamp {
                if out_timestamp > old_timestamp && !self.config.allow_overwrites {
                    return Err(SafeWriterError::ModifiedExternally(output_file));
                }
            }
        }

        self.copy_if_different(&private_file, &output_file)?;

        Ok(())
    }

    pub fn get_config(&self) -> &SafeWriterConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: SafeWriterConfig) {
        self.config = config;
    }

    #[cfg(test)]
    pub fn get_gen_base(&self) -> &Path {
        &self.gen_base
    }

    #[cfg(test)]
    pub fn get_old_dir(&self) -> &Path {
        &self.old_dir
    }

    #[cfg(test)]
    pub fn get_private_dir(&self) -> &Path {
        &self.private_dir
    }
}

/// Validate that the filename does not specify an absolute path or attempt directory traversal.
fn validate_filename(path: &Path) -> Result<(), SafeWriterError> {
    let filename = path.to_string_lossy();

    // Check for Unix-style absolute path
    if filename.starts_with('/') {
        return Err(SafeWriterError::SecurityViolation(format!(
            "Absolute paths are not allowed: {}",
            filename
        )));
    }

    // Check for Windows-style absolute paths, e.g., "C:" or "D:"
    if filename.len() >= 2 {
        let chars: Vec<char> = filename.chars().collect();
        if chars[1] == ':' && chars[0].is_ascii_alphabetic() {
            return Err(SafeWriterError::SecurityViolation(format!(
                "Windows-style absolute paths are not allowed: {}",
                filename
            )));
        }
    }

    // Check if filename contains '..'
    if filename.split('/').any(|component| component == "..") {
        return Err(SafeWriterError::SecurityViolation(format!(
            "Path traversal detected (..): {}",
            filename
        )));
    }

    Ok(())
}
