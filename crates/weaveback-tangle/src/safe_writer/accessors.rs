// weaveback-tangle/src/safe_writer/accessors.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;
use std::path::Path;

impl SafeFileWriter {
    pub fn get_config(&self) -> &SafeWriterConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: SafeWriterConfig) {
        self.config = config;
    }

    pub fn db(&self) -> &WeavebackDb {
        &self.db
    }

    pub fn db_mut(&mut self) -> &mut WeavebackDb {
        &mut self.db
    }

    pub fn finish(self, target: &Path) -> Result<(), SafeWriterError> {
        self.db.merge_into(target).map_err(SafeWriterError::DbError)?;
        Ok(())
    }

    pub fn get_gen_base(&self) -> &Path {
        &self.gen_base
    }

    /// Retrieve the stored baseline bytes for a relative path (test helper).
    #[cfg(test)]
    pub fn get_baseline_for_test(&self, path: &str) -> Option<Vec<u8>> {
        self.db.get_baseline(path).ok().flatten()
    }
}
