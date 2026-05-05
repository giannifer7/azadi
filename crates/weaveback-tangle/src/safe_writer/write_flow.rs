// weaveback-tangle/src/safe_writer/write_flow.rs
// I'd Really Rather You Didn't edit this generated file.

use super::paths::validate_filename;
use super::*;
use std::fs;
use std::path::{Path, PathBuf};

impl SafeFileWriter {
    pub fn before_write<P: AsRef<Path>>(
        &mut self,
        file_name: P,
    ) -> Result<PathBuf, SafeWriterError> {
        validate_filename(file_name.as_ref())?;
        let path = file_name.as_ref();

        let dest_dir = path.parent().unwrap_or_else(|| Path::new(""));
        fs::create_dir_all(self.gen_base.join(dest_dir))
            .map_err(|_| SafeWriterError::DirectoryCreationFailed(self.gen_base.join(dest_dir)))?;

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let suffix = if ext.is_empty() {
            String::new()
        } else {
            format!(".{ext}")
        };
        let tmp = tempfile::Builder::new()
            .suffix(&suffix)
            .tempfile()
            .map_err(SafeWriterError::IoError)?;
        let tmp_path = tmp.path().to_path_buf();
        self.staging.insert(path.to_string_lossy().into_owned(), tmp);
        Ok(tmp_path)
    }

    /// Run the post-write pipeline and return the final (possibly formatted)
    /// file content as bytes.  The caller can use these bytes directly for
    /// source-map remapping without re-reading the output file from disk.
    pub fn after_write<P: AsRef<Path>>(&mut self, file_name: P) -> Result<Vec<u8>, SafeWriterError> {
        validate_filename(file_name.as_ref())?;
        let key = file_name.as_ref().to_string_lossy().into_owned();
        let tmp = self
            .staging
            .remove(&key)
            .ok_or_else(|| SafeWriterError::BackupFailed(file_name.as_ref().to_path_buf()))?;
        let tmp_path = tmp.path().to_path_buf();
        let output_file = self.gen_base.join(file_name.as_ref());

        // Step 1: run formatter on temp copy if configured.
        let ext = file_name
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let mut formatted = false;
        if let Some(cmd) = self.config.formatters.get(ext).cloned() {
            let pre_size = fs::metadata(&tmp_path).map(|m| m.len()).unwrap_or(0);
            self.run_formatter(&cmd, &tmp_path)?;
            if pre_size > 0 {
                let post_size = fs::metadata(&tmp_path).map(|m| m.len()).unwrap_or(0);
                if post_size == 0 {
                    return Err(SafeWriterError::FormatterError(format!(
                        "formatter '{cmd}' produced an empty file (input was {pre_size} bytes)"
                    )));
                }
            }
            formatted = true;
        }

        if !formatted {
            self.trim_trailing_whitespace(&tmp_path)?;
        }

        // Step 2: content-based modification detection.
        // When a stored baseline exists, compare the on-disk file against it:
        // any difference means the file was hand-edited since the last tangle.
        // When no baseline exists (fresh checkout or reset db), compare against
        // what tangle is about to write: in a consistent literate project the
        // committed generated file should match the committed .adoc, so any
        // difference still indicates a hand-edit.
        if output_file.is_file() && !self.config.force_generated {
            let current = fs::read(&output_file)?;
            let reference = match self.db.get_baseline(&key)? {
                Some(b) => b,
                None => fs::read(&tmp_path)?,
            };
            if current != reference {
                return Err(SafeWriterError::ModifiedExternally(output_file));
            }
        }

        // Step 3: copy temp → output.
        // Normally skip the copy when content is identical (keeps build-system
        // timestamps stable).  When force_generated is set we always overwrite —
        // that is the whole point of the flag.
        if self.config.force_generated {
            self.atomic_copy(&tmp_path, &output_file)
                .map_err(SafeWriterError::from)?;
        } else {
            self.copy_if_different(&tmp_path, &output_file)?;
        }

        // Step 4: read the (possibly formatted) temp content for the baseline
        // and return it to the caller so they don't need a second disk read.
        let written = fs::read(&tmp_path)
            .map_err(|_| SafeWriterError::BackupFailed(tmp_path.clone()))?;
        self.db
            .set_baseline(&key, &written)
            .map_err(SafeWriterError::DbError)?;

        // tmp is dropped here, deleting the temp file.
        Ok(written)
    }
}

