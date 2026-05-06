// weaveback-tangle/src/db/fts/embeddings.rs
// I'd Really Rather You Didn't edit this generated file.

use super::helpers::{cosine_similarity, normalise_snapshot_path, prose_snippet};
use super::*;

impl WeavebackDb {
    /// Return all prose blocks that need embeddings for `model`.
    pub fn get_blocks_needing_embeddings(
        &self,
        model: &str,
    ) -> Result<Vec<BlockForEmbedding>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT f.path, sb.block_index, sb.block_type, sb.line_start, sb.line_end, sb.content_hash
             FROM source_blocks sb
             JOIN files f ON f.id = sb.src_file
             LEFT JOIN block_embeddings be
               ON be.src_file = sb.src_file AND be.block_index = sb.block_index
             WHERE sb.block_type IN ('section', 'para')
               AND (be.src_file IS NULL OR be.content_hash != sb.content_hash OR be.model != ?1)",
        )?;
        let rows = stmt.query_map(params![model], |row| {
            Ok(BlockForEmbedding {
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

    /// Store an embedding vector for a prose block. Overwrites any previous entry.
    pub fn set_block_embedding(
        &mut self,
        src_file: &str,
        block_index: u32,
        content_hash: &[u8],
        model: &str,
        vector: &[f32],
    ) -> Result<(), DbError> {
        let file_id = intern_file(&self.conn, src_file)?;
        let vector_json = serde_json::to_string(vector)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO block_embeddings
             (src_file, block_index, content_hash, model, vector_json)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![file_id, block_index, content_hash, model, vector_json],
        )?;
        Ok(())
    }

    /// Brute-force cosine search over stored prose-block embeddings.
    pub fn search_prose_by_embedding(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<SemanticResult>, DbError> {
        let cwd = std::env::current_dir().unwrap_or_default();
        let mut snapshot_cache: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let mut stmt = self.conn.prepare(
            "SELECT f.path, sb.block_type, sb.line_start, sb.line_end,
                    COALESCE(bt.tags, ''), be.vector_json
             FROM block_embeddings be
             JOIN files f ON f.id = be.src_file
             JOIN source_blocks sb
               ON sb.src_file = be.src_file AND sb.block_index = be.block_index
             LEFT JOIN block_tags bt
               ON bt.src_file = be.src_file AND bt.block_index = be.block_index
             WHERE sb.block_type IN ('section', 'para')",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, u32>(2)?,
                row.get::<_, u32>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;

        let mut results = Vec::new();
        for row in rows {
            let (src_file, block_type, line_start, line_end, tags, vector_json) = row?;
            let block_embedding: Vec<f32> = serde_json::from_str(&vector_json)?;
            let score = cosine_similarity(query_embedding, &block_embedding);
            if !score.is_finite() || score <= 0.0 {
                continue;
            }

            let snapshot = if let Some(cached) = snapshot_cache.get(&src_file) {
                cached.clone()
            } else {
                let bytes = self.get_src_snapshot(&src_file)?
                    .or_else(|| {
                        let alt = normalise_snapshot_path(&src_file, &cwd);
                        if alt == src_file {
                            None
                        } else {
                            self.get_src_snapshot(&alt).ok().flatten()
                        }
                    });
                let Some(bytes) = bytes else { continue; };
                let Ok(source) = String::from_utf8(bytes) else { continue; };
                snapshot_cache.insert(src_file.clone(), source.clone());
                source
            };

            let lines: Vec<&str> = snapshot.lines().collect();
            let lo = (line_start as usize).saturating_sub(1);
            let hi = (line_end as usize).min(lines.len());
            if lo >= hi {
                continue;
            }
            let content = lines[lo..hi].join("\n");
            results.push(SemanticResult {
                src_file,
                block_type,
                line_start,
                line_end,
                snippet: prose_snippet(&content),
                tags,
                score,
            });
        }

        results.sort_by(|lhs, rhs| rhs.score.partial_cmp(&lhs.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);
        Ok(results)
    }
}
