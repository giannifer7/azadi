use crate::AzadiError;
use blake3::Hasher;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

pub struct SafeFileWriter {
    gen_dir: PathBuf,
    priv_dir: PathBuf,
    old_dir: PathBuf,
    safe: bool,
}

impl SafeFileWriter {
    pub fn new<P: AsRef<Path>>(gen_base: P, private_dir: P, safe: bool) -> Self {
        let g = gen_base.as_ref().to_path_buf();
        let p = private_dir.as_ref().to_path_buf();
        let o = p.join("__old__");
        fs::create_dir_all(&g).ok();
        fs::create_dir_all(&p).ok();
        fs::create_dir_all(&o).ok();
        Self {
            gen_dir: g,
            priv_dir: p,
            old_dir: o,
            safe,
        }
    }

    fn check_path(&self, f: &Path) -> Result<(), AzadiError> {
        let s = f.to_string_lossy();
        if f.is_absolute() {
            return Err(AzadiError::SecurityViolation(
                "Absolute paths not allowed".to_string(),
            ));
        }
        if s.contains(':') {
            return Err(AzadiError::SecurityViolation(
                "Windows-style paths not allowed".to_string(),
            ));
        }
        if s.contains("..") {
            return Err(AzadiError::SecurityViolation(
                "Path traversal not allowed".to_string(),
            ));
        }
        Ok(())
    }

    fn compute_hash(f: &Path) -> Result<String, AzadiError> {
        let mut h = Hasher::new();
        let mut file = fs::File::open(f)?;
        let mut buf = vec![0; 8192];
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            h.update(&buf[..n]);
        }
        Ok(h.finalize().to_hex().to_string())
    }

    fn sidecar_path(old_path: &Path) -> PathBuf {
        let mut sc = old_path.to_path_buf();
        let fname = sc.file_name().unwrap_or_else(|| std::ffi::OsStr::new("x"));
        sc.set_file_name(format!("{}.hash", fname.to_string_lossy()));
        sc
    }

    fn read_or_write_old_hash(old_file: &Path) -> Result<String, AzadiError> {
        let sc = Self::sidecar_path(old_file);
        if sc.is_file() {
            let c = fs::read_to_string(&sc)?;
            let t = c.trim();
            if t.is_empty() {
                let h = Self::compute_hash(old_file)?;
                fs::write(&sc, &h)?;
                Ok(h)
            } else {
                Ok(t.to_string())
            }
        } else {
            let h = Self::compute_hash(old_file)?;
            fs::write(&sc, &h)?;
            Ok(h)
        }
    }

    pub fn before_write<P: AsRef<Path>>(&mut self, file_name: P) -> Result<PathBuf, AzadiError> {
        let f = file_name.as_ref();
        self.check_path(f)?;
        if !self.safe {
            let final_file = self.gen_dir.join(f);
            if let Some(par) = final_file.parent() {
                fs::create_dir_all(par)?;
            }
            return Ok(final_file);
        }
        let final_file = self.gen_dir.join(f);
        let old_file = self.old_dir.join(f);
        if final_file.is_file() && old_file.is_file() {
            let final_hash = Self::compute_hash(&final_file)?;
            let old_hash = Self::read_or_write_old_hash(&old_file)?;
            if final_hash != old_hash {
                return Err(AzadiError::ModifiedExternally(format!(
                    "{} was modified externally",
                    final_file.display()
                )));
            }
        }
        let priv_file = self.priv_dir.join(f);
        if let Some(par) = priv_file.parent() {
            fs::create_dir_all(par)?;
        }
        Ok(priv_file)
    }

    pub fn after_write<P: AsRef<Path>>(&mut self, file_name: P) -> Result<(), AzadiError> {
        let f = file_name.as_ref();
        self.check_path(f)?;
        if !self.safe {
            return Ok(());
        }
        let priv_file = self.priv_dir.join(f);
        let old_file = self.old_dir.join(f);
        let final_file = self.gen_dir.join(f);
        if final_file.exists() {
            fs::copy(&final_file, &old_file).map_err(|e| {
                AzadiError::IoError(io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to backup {} to {}",
                        final_file.display(),
                        old_file.display()
                    ),
                ))
            })?;
            fs::remove_file(&final_file)?;
        }
        if let Some(p) = final_file.parent() {
            fs::create_dir_all(p)?;
        }
        fs::rename(&priv_file, &final_file).map_err(|e| {
            AzadiError::IoError(io::Error::new(
                e.kind(),
                format!(
                    "Failed to move {} to {}",
                    priv_file.display(),
                    final_file.display()
                ),
            ))
        })?;
        if old_file.is_file() {
            let new_old_hash = Self::compute_hash(&old_file)?;
            let sc = Self::sidecar_path(&old_file);
            if let Some(pa) = sc.parent() {
                fs::create_dir_all(pa)?;
            }
            fs::write(&sc, &new_old_hash)?;
        } else {
            if let Some(pa) = old_file.parent() {
                fs::create_dir_all(pa)?;
            }
            fs::copy(&final_file, &old_file)?;
            let new_old_hash = Self::compute_hash(&old_file)?;
            let sc = Self::sidecar_path(&old_file);
            fs::write(&sc, &new_old_hash)?;
        }
        Ok(())
    }
}
