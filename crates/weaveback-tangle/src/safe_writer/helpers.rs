// weaveback-tangle/src/safe_writer/helpers.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;
use std::fs::{self, File};
use std::io::Read;
use std::io::{self, BufReader};
use std::path::Path;

impl SafeFileWriter {
    pub(in crate::safe_writer) fn atomic_copy<P: AsRef<Path>>(
        &self,
        source: P,
        destination: P,
    ) -> io::Result<()> {
        let destination = destination.as_ref();
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp_path = destination.with_extension("tmp");

        if temp_path.exists() {
            let _ = fs::remove_file(&temp_path);
        }

        {
            let mut source_file = fs::File::open(&source)?;
            let mut temp_file = fs::File::create(&temp_path)?;
            io::copy(&mut source_file, &mut temp_file)?;
            temp_file.sync_all()?;
        }

        fs::rename(temp_path, destination)?;
        Ok(())
    }

    pub(in crate::safe_writer) fn copy_if_different<P: AsRef<Path>>(
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
            let mut source_file =
                BufReader::with_capacity(self.config.buffer_size, File::open(source)?);
            let mut dest_file =
                BufReader::with_capacity(self.config.buffer_size, File::open(destination)?);

            let mut src_buf = vec![0u8; self.config.buffer_size];
            let mut dst_buf = vec![0u8; self.config.buffer_size];
            loop {
                let src_n = source_file.read(&mut src_buf)?;
                let dst_n = dest_file.read(&mut dst_buf)?;
                if src_n != dst_n || src_buf[..src_n] != dst_buf[..dst_n] {
                    break true;
                }
                if src_n == 0 {
                    break false;
                }
            }
        };

        if are_different {
            eprintln!("file {} changed", destination.display());
            self.atomic_copy(source, destination)?;
        }

        Ok(())
    }

    pub(in crate::safe_writer) fn run_formatter(
        &self,
        command: &str,
        file: &Path,
    ) -> Result<(), SafeWriterError> {
        let parts = shlex::split(command).ok_or_else(|| {
            SafeWriterError::FormatterError(format!(
                "could not parse formatter command: '{}'", command
            ))
        })?;
        if parts.is_empty() {
            return Err(SafeWriterError::FormatterError(
                "formatter command is empty".to_string(),
            ));
        }
        let status = std::process::Command::new(&parts[0])
            .args(&parts[1..])
            .arg(file)
            .status()
            .map_err(|e| {
                SafeWriterError::FormatterError(format!("could not run '{}': {}", command, e))
            })?;
        if !status.success() {
            return Err(SafeWriterError::FormatterError(format!(
                "'{}' exited with code {}",
                command,
                status.code().unwrap_or(-1)
            )));
        }
        Ok(())
    }

    pub(in crate::safe_writer) fn normalize_trailing_whitespace(&self, path: &Path) -> io::Result<()> {
        let content = fs::read(path)?;
        if let Ok(text) = std::str::from_utf8(&content) {
            let ends_with_newline = content.last() == Some(&b'\n');
            let mut lines: Vec<&str> = text.split('\n').collect();
            if ends_with_newline {
                lines.pop();
            }
            while lines
                .last()
                .is_some_and(|line| line.trim_end_matches([' ', '\t', '\r']).is_empty())
            {
                lines.pop();
            }
            let mut result = Vec::with_capacity(content.len());
            for line in lines {
                result.extend_from_slice(
                    line.trim_end_matches([' ', '\t', '\r']).as_bytes()
                );
                result.push(b'\n');
            }
            if (!ends_with_newline || result.len() == 1) && result.last() == Some(&b'\n') {
                result.pop();
            }
            fs::write(path, result)?;
        }
        Ok(())
    }
}
