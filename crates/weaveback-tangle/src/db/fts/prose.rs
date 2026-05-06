// weaveback-tangle/src/db/fts/prose.rs
// I'd Really Rather You Didn't edit this generated file.

use super::helpers::normalise_snapshot_path;
use super::*;

impl WeavebackDb {
    /// Rebuild the `prose_fts` index from `src_snapshots` + `source_blocks`.
    /// Drops and re-inserts all rows so the index is always consistent.
    /// `root` overrides the CWD for path normalization.
    pub fn rebuild_prose_fts(&mut self, root: Option<&std::path::Path>) -> Result<(), DbError> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM prose_fts", [])?;

        // Snapshot paths may be stored as "./rel", "rel", or absolute.
        // The files table uses plain relative paths.  Normalise here.
        let cwd = root.map(|p| p.to_path_buf()).unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // Load all snapshots; query block metadata per file.
        // Deduplicate after path normalisation: the same source file may be
        // stored under both `./rel/path` and `rel/path` from different passes.
        let snapshots: Vec<(String, String)> = {
            let mut stmt = tx.prepare(
                "SELECT path, content FROM src_snapshots",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?))
            })?;
            let mut seen = std::collections::HashSet::new();
            rows.filter_map(|r| r.ok())
                .filter_map(|(path, bytes)| {
                    let path = normalise_snapshot_path(&path, &cwd);
                    String::from_utf8(bytes).ok().map(|s| (path, s))
                })
                .filter(|(path, _)| seen.insert(path.clone()))
                .collect()
        };

        for (path, source) in &snapshots {
            let lines: Vec<&str> = source.lines().collect();
            let mut stmt = tx.prepare_cached(
                "SELECT DISTINCT sb.block_type, sb.line_start, sb.line_end, sb.block_index
                 FROM source_blocks sb JOIN files f ON f.id = sb.src_file
                 WHERE f.path = ?1
                   AND sb.block_type IN ('section', 'para')
                 ORDER BY sb.line_start",
            )?;
            let blocks: Vec<(String, u32, u32, u32)> = stmt
                .query_map(params![path], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, u32>(1)?,
                        row.get::<_, u32>(2)?,
                        row.get::<_, u32>(3)?,
                    ))
                })?
                .filter_map(|r| r.ok())
                .collect();

            let mut ins = tx.prepare_cached(
                "INSERT INTO prose_fts (content, tags, src_file, block_type, line_start, line_end)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            let mut tag_stmt = tx.prepare_cached(
                "SELECT bt.tags FROM block_tags bt
                 JOIN files f ON f.id = bt.src_file
                 WHERE f.path = ?1 AND bt.block_index = ?2",
            )?;
            for (btype, start, end, block_index) in blocks {
                let lo = (start as usize).saturating_sub(1);
                let hi = (end as usize).min(lines.len());
                if lo >= hi { continue; }
                let content = lines[lo..hi].join("\n");
                if content.trim().is_empty() { continue; }
                let tags: String = tag_stmt
                    .query_row(params![path, block_index], |row| row.get(0))
                    .unwrap_or_default();
                ins.execute(params![content, tags, path, btype, start, end])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    /// BM25-ranked full-text search over literate source prose.
    pub fn search_prose(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<FtsResult>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT src_file, block_type, line_start, line_end,
                    snippet(prose_fts, 0, '**', '**', '…', 16),
                    tags
             FROM prose_fts
             WHERE prose_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![query, limit as i64], |row| {
            Ok(FtsResult {
                src_file:   row.get(0)?,
                block_type: row.get(1)?,
                line_start: row.get(2)?,
                line_end:   row.get(3)?,
                snippet:    row.get(4)?,
                tags:       row.get(5)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }
}
