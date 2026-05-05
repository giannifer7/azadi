// weaveback-tangle/src/block_parser.rs
// I'd Really Rather You Didn't edit this generated file.

/// Sub-file block parsing for incremental build support.
///
/// Splits a source file into logical blocks (code blocks, section headers,
/// prose paragraphs) and computes a BLAKE3 hash for each block.  The hashes
/// are stored in the database so that unchanged blocks can be skipped on the
/// next run.
mod adoc;
mod markdown;
mod utils;

use adoc::parse_adoc_raw;
use markdown::parse_markdown_raw;

#[cfg(test)]
use adoc::{has_unclosed_adoc_fence, is_adoc_fence, is_adoc_section_header};

/// A parsed logical block with its line range and content hash.
#[derive(Debug, Clone)]
pub struct SourceBlockEntry {
    pub block_index: u32,
    pub block_type: String,
    pub line_start: u32, // 1-based, inclusive
    pub line_end: u32,   // 1-based, inclusive
    pub content_hash: [u8; 32],
}
/// Parse `source` into logical blocks based on its file `extension`.
///
/// Recognised extensions: `adoc`, `asciidoc` (AsciiDoc line scanner);
/// `md`, `markdown` (pulldown-cmark); everything else gets a single block.
pub fn parse_source_blocks(source: &str, extension: &str) -> Vec<SourceBlockEntry> {
    let raw = match extension {
        "adoc" | "asciidoc" => parse_adoc_raw(source),
        "md" | "markdown" => parse_markdown_raw(source),
        _ => {
            let n = source.lines().count().max(1) as u32;
            vec![(1, n, "text", source.to_string())]
        }
    };

    raw.into_iter()
        .enumerate()
        .map(|(i, (start, end, btype, content))| {
            let mut h = blake3::Hasher::new();
            h.update(content.as_bytes());
            SourceBlockEntry {
                block_index: i as u32,
                block_type: btype.to_string(),
                line_start: start,
                line_end: end,
                content_hash: *h.finalize().as_bytes(),
            }
        })
        .collect()
}
#[cfg(test)]
mod tests;

