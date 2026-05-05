// weaveback-tangle/src/db/fts/tags.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

impl WeavebackDb {
    /// List all tagged blocks, optionally filtered to a single source file.
    /// `file_filter` should be a plain relative path (no leading `./`).
    pub fn list_block_tags(
        &self,
        file_filter: Option<&str>,
    ) -> Result<Vec<TaggedBlock>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT f.path, bt.block_index, sb.block_type, sb.line_start, bt.tags
             FROM block_tags bt
             JOIN files f ON f.id = bt.src_file
             LEFT JOIN source_blocks sb
               ON sb.src_file = bt.src_file AND sb.block_index = bt.block_index
             WHERE (?1 IS NULL OR f.path = ?1)
             ORDER BY f.path, sb.line_start",
        )?;
        let rows = stmt.query_map(params![file_filter], |row| {
            Ok(TaggedBlock {
                src_file:    row.get(0)?,
                block_index: row.get(1)?,
                block_type:  row.get(2)?,
                line_start:  row.get(3)?,
                tags:        row.get(4)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Return all prose blocks that have no tag entry or whose content has
    /// changed since the last tagging run (hash mismatch).
    pub fn get_blocks_needing_tags(&self) -> Result<Vec<BlockForTagging>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT f.path, sb.block_index, sb.block_type, sb.line_start, sb.line_end, sb.content_hash
             FROM source_blocks sb
             JOIN files f ON f.id = sb.src_file
             LEFT JOIN block_tags bt
               ON bt.src_file = sb.src_file AND bt.block_index = sb.block_index
             WHERE sb.block_type IN ('section', 'para')
               AND (bt.src_file IS NULL OR bt.content_hash != sb.content_hash)",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(BlockForTagging {
                src_file:     row.get(0)?,
                block_index:  row.get(1)?,
                block_type:   row.get(2)?,
                line_start:   row.get(3)?,
                line_end:     row.get(4)?,
                content_hash: row.get(5)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Store LLM-generated tags for a block. Overwrites any previous entry.
    /// `tags` is a comma-separated string, e.g. `"fts,sqlite,search"`.
    pub fn set_block_tags(
        &mut self,
        src_file: &str,
        block_index: u32,
        content_hash: &[u8],
        tags: &str,
    ) -> Result<(), DbError> {
        let file_id = intern_file(&self.conn, src_file)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO block_tags (src_file, block_index, content_hash, tags)
             VALUES (?1, ?2, ?3, ?4)",
            params![file_id, block_index, content_hash, tags],
        )?;
        Ok(())
    }
}

