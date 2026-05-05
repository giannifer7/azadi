// weaveback-tangle/src/block_parser/markdown.rs
// I'd Really Rather You Didn't edit this generated file.

use super::utils::{build_line_table, byte_to_line};

/// Parse Markdown using pulldown-cmark's offset iterator.
///
/// Produces blocks of type:
/// * `"section"` — a heading
/// * `"code"`    — a fenced code block
/// * `"para"`    — a paragraph or other leaf element
pub(in crate::block_parser) fn parse_markdown_raw(source: &str) -> Vec<(u32, u32, &'static str, String)> {
    use pulldown_cmark::{Event, Parser, Tag, TagEnd};

    // Build a byte→line lookup table.
    let line_of_byte = build_line_table(source);

    let parser = Parser::new(source).into_offset_iter();
    let mut blocks: Vec<(u32, u32, &'static str, String)> = Vec::new();

    let mut depth = 0usize; // nesting depth so we skip inner events
    let mut cur_type: Option<&'static str> = None;
    let mut cur_start = 0usize;

    for (event, range) in parser {
        match event {
            Event::Start(tag) => {
                depth += 1;
                if depth == 1 {
                    let btype = match &tag {
                        Tag::Heading { .. } => "section",
                        Tag::CodeBlock(_) => "code",
                        _ => "para",
                    };
                    cur_type = Some(btype);
                    cur_start = range.start;
                }
            }
            Event::End(end_tag) => {
                if depth == 1
                    && let Some(btype) = cur_type.take() {
                        // Only emit code / section at depth == 1
                        let emit = matches!(end_tag, TagEnd::Heading(_) | TagEnd::CodeBlock)
                            || btype == "para";
                        if emit {
                            let byte_end = range.end;
                            let start_line = byte_to_line(&line_of_byte, cur_start);
                            let end_line = byte_to_line(&line_of_byte, byte_end.saturating_sub(1));
                            let content = source[cur_start..byte_end.min(source.len())].to_string();
                            blocks.push((start_line, end_line, btype, content));
                        }
                }
                depth = depth.saturating_sub(1);
            }
            _ => {}
        }
    }

    if blocks.is_empty() {
        let n = source.lines().count().max(1) as u32;
        blocks.push((1, n, "text", source.to_string()));
    }
    blocks
}

