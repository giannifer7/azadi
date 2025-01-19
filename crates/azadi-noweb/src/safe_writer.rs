// src/safe_writer.rs
//
// Full rewriting logic.
// Marked unused imports with underscores or removed them.

use chrono::{DateTime, Local};
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::AzadiError;

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
    pub gen_base: PathBuf,
    private_dir: PathBuf,
    old_dir: PathBuf,
    old_timestamp: Option<DateTime<Local>>,
    config: SafeWriterConfig,
}

impl SafeFileWriter {
    pub fn new<P: AsRef<Path>>(base: P) -> Self {
        let b = base.as_ref().to_path_buf();
        let privp = b.join("_private_");
        let oldp = b.join("__old__");
        fs::create_dir_all(&b).ok();
        fs::create_dir_all(&privp).ok();
        fs::create_dir_all(&oldp).ok();
        Self {
            gen_base: b,
            private_dir: privp,
            old_dir: oldp,
            old_timestamp: None,
            config: SafeWriterConfig::default(),
        }
    }

    pub fn with_config<P: AsRef<Path>>(base: P, config: SafeWriterConfig) -> Self {
        let b = base.as_ref().to_path_buf();
        let privp = b.join("_private_");
        let oldp = b.join("__old__");
        fs::create_dir_all(&b).ok();
        fs::create_dir_all(&privp).ok();
        fs::create_dir_all(&oldp).ok();
        Self {
            gen_base: b,
            private_dir: privp,
            old_dir: oldp,
            old_timestamp: None,
            config,
        }
    }

    pub fn get_config(&self) -> &SafeWriterConfig {
        &self.config
    }
    pub fn set_config(&mut self, c: SafeWriterConfig) {
        self.config = c;
    }

    pub fn get_gen_base(&self) -> &PathBuf {
        &self.gen_base
    }

    fn check_path(&self, path: &Path) -> Result<(), AzadiError> {
        let s = path.to_string_lossy();
        if path.is_absolute() {
            return Err(AzadiError::SecurityViolation(
                "Absolute paths are not allowed".to_string(),
            ));
        }
        if s.contains(':') {
            return Err(AzadiError::SecurityViolation(
                "Windows-style paths are not allowed".to_string(),
            ));
        }
        if s.contains("..") {
            return Err(AzadiError::SecurityViolation(
                "Path traversal is not allowed".to_string(),
            ));
        }
        Ok(())
    }

    fn canonical_path(&self, path: &Path) -> Result<PathBuf, AzadiError> {
        self.check_path(path)?;
        Ok(self.gen_base.join(path))
    }

    pub fn before_write<P: AsRef<Path>>(&mut self, file_name: P) -> Result<PathBuf, AzadiError> {
        let p = file_name.as_ref();
        self.check_path(p)?;
        let finalp = self.private_dir.join(p);
        let oldp = self.old_dir.join(p);

        if oldp.is_file() {
            let meta = fs::metadata(&oldp)?;
            let systime = meta.modified()?;
            self.old_timestamp = Some(systime.into());
        } else {
            self.old_timestamp = None;
        }
        if let Some(dir) = finalp.parent() {
            fs::create_dir_all(dir).ok();
        }
        Ok(finalp)
    }

    pub fn after_write<P: AsRef<Path>>(&self, file_name: P) -> Result<(), AzadiError> {
        let p = file_name.as_ref();
        self.check_path(p)?;
        let privp = self.private_dir.join(p);
        let finalp = self.canonical_path(p)?;
        let oldp = self.old_dir.join(p);

        // backup
        if self.config.backup_enabled {
            let _ = fs::copy(&privp, &oldp);
        }

        // modification check
        if self.config.modification_check && finalp.is_file() {
            let meta = fs::metadata(&finalp)?;
            let st: SystemTime = meta.modified()?;
            let out_ts: DateTime<Local> = st.into();
            if let Some(old_ts) = self.old_timestamp {
                if out_ts > old_ts && !self.config.allow_overwrites {
                    return Err(AzadiError::ModifiedExternally(format!(
                        "{}",
                        finalp.display()
                    )));
                }
            }
        }

        // rewrite if different
        if !finalp.exists() {
            fs::copy(&privp, &finalp)?;
            return Ok(());
        }
        let mut sfile = BufReader::new(fs::File::open(&privp)?);
        let mut dfile = BufReader::new(fs::File::open(&finalp)?);
        let mut sbuf = Vec::new();
        let mut dbuf = Vec::new();
        sfile.read_to_end(&mut sbuf)?;
        dfile.read_to_end(&mut dbuf)?;
        if sbuf != dbuf {
            fs::copy(&privp, &finalp)?;
        }
        Ok(())
    }
}
