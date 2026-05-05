// weaveback-tangle/src/db/fts/types.rs
// I'd Really Rather You Didn't edit this generated file.

/// A single result from `search_prose`.
#[derive(Debug, Clone)]
pub struct FtsResult {
    pub src_file:   String,
    pub block_type: String,
    pub line_start: u32,
    pub line_end:   u32,
    /// Short excerpt with matched terms wrapped in `**...**`.
    pub snippet:    String,
    /// Comma-separated LLM-generated tags; empty if block has not been tagged.
    pub tags:       String,
}

/// A tagged block returned by `list_block_tags`.
#[derive(Debug, Clone)]
pub struct TaggedBlock {
    pub src_file:    String,
    pub block_index: u32,
    pub block_type:  String,
    pub line_start:  u32,
    pub tags:        String,
}

/// A block that needs embeddings (never embedded, content changed, or model changed).
#[derive(Debug, Clone)]
pub struct BlockForEmbedding {
    pub src_file:     String,
    pub block_index:  u32,
    pub block_type:   String,
    pub line_start:   u32,
    pub line_end:     u32,
    pub content_hash: Vec<u8>,
}

/// A semantic-search hit returned by `search_prose_by_embedding`.
#[derive(Debug, Clone)]
pub struct SemanticResult {
    pub src_file:   String,
    pub block_type: String,
    pub line_start: u32,
    pub line_end:   u32,
    pub snippet:    String,
    pub tags:       String,
    pub score:      f32,
}

