// weaveback-tangle/src/block_parser/adoc.rs
// I'd Really Rather You Didn't edit this generated file.

use super::utils::{build_line_table, byte_to_line};

/// Parse an AsciiDoc document using ACDC, falling back to the simple line
/// scanner if parsing fails.
pub(in crate::block_parser) fn parse_adoc_raw(source: &str) -> Vec<(u32, u32, &'static str, String)> {
    if has_unclosed_adoc_fence(source) {
        return parse_adoc_raw_simple(source);
    }

    match parse_adoc_with_parser(source) {
        Some(blocks) if !blocks.is_empty() => blocks,
        _ => parse_adoc_raw_simple(source),
    }
}

fn parse_adoc_with_parser(source: &str) -> Option<Vec<(u32, u32, &'static str, String)>> {
    let line_of_byte = build_line_table(source);
    let masked = mask_preprocessor_directives(source);
    let doc = acdc_parser::Parser::new(&masked).parse().ok()?;
    let mut blocks = Vec::new();
    collect_adoc_blocks(&doc.blocks, source, &line_of_byte, &mut blocks);
    if blocks.is_empty() {
        let n = source.lines().count().max(1) as u32;
        blocks.push((1, n, "text", source.to_string()));
    }
    Some(blocks)
}

fn collect_adoc_blocks(
    blocks: &[acdc_parser::Block],
    source: &str,
    line_of_byte: &[usize],
    out: &mut Vec<(u32, u32, &'static str, String)>,
) {
    use acdc_parser::Block;

    for block in blocks {
        match block {
            Block::Section(s) => {
                // Emit the title line as "section", then recurse into children.
                let line = byte_to_line(line_of_byte, s.location.absolute_start);
                out.push((line, line, "section", source_line(source, line).to_string()));
                collect_adoc_blocks(&s.content, source, line_of_byte, out);
            }
            Block::DelimitedBlock(d) if is_code_delimited_block(d) => {
                push_location_block(source, line_of_byte, &d.location, "code", out);
            }
            other => {
                if let Some(location) = block_location(other) {
                    out.push(location_to_block(source, line_of_byte, location, "para"));
                }
            }
        }
    }
}

fn is_code_delimited_block(block: &acdc_parser::DelimitedBlock) -> bool {
    matches!(
        block.inner,
        acdc_parser::DelimitedBlockType::DelimitedListing(_)
            | acdc_parser::DelimitedBlockType::DelimitedLiteral(_)
            | acdc_parser::DelimitedBlockType::DelimitedPass(_)
    )
}

fn push_location_block(
    source: &str,
    line_of_byte: &[usize],
    location: &acdc_parser::Location,
    block_type: &'static str,
    out: &mut Vec<(u32, u32, &'static str, String)>,
) {
    out.push(location_to_block(source, line_of_byte, location, block_type));
}

fn location_to_block(
    source: &str,
    line_of_byte: &[usize],
    location: &acdc_parser::Location,
    block_type: &'static str,
) -> (u32, u32, &'static str, String) {
    let start = location.absolute_start.min(source.len());
    let end = location.absolute_end.saturating_add(1).min(source.len());
    let line_start = byte_to_line(line_of_byte, start);
    let line_end = byte_to_line(line_of_byte, end.saturating_sub(1));
    let content = if source.is_char_boundary(start) && source.is_char_boundary(end) && start <= end {
        source[start..end].to_string()
    } else {
        String::new()
    };
    (line_start, line_end, block_type, content)
}

fn block_location(block: &acdc_parser::Block) -> Option<&acdc_parser::Location> {
    use acdc_parser::Block;

    match block {
        Block::TableOfContents(block) => Some(&block.location),
        Block::Admonition(block) => Some(&block.location),
        Block::DiscreteHeader(block) => Some(&block.location),
        Block::DocumentAttribute(block) => Some(&block.location),
        Block::ThematicBreak(block) => Some(&block.location),
        Block::PageBreak(block) => Some(&block.location),
        Block::UnorderedList(block) => Some(&block.location),
        Block::OrderedList(block) => Some(&block.location),
        Block::CalloutList(block) => Some(&block.location),
        Block::DescriptionList(block) => Some(&block.location),
        Block::Section(block) => Some(&block.location),
        Block::DelimitedBlock(block) => Some(&block.location),
        Block::Paragraph(block) => Some(&block.location),
        Block::Image(block) => Some(&block.location),
        Block::Audio(block) => Some(&block.location),
        Block::Video(block) => Some(&block.location),
        Block::Comment(block) => Some(&block.location),
        _ => None,
    }
}

fn source_line(source: &str, line: u32) -> &str {
    source
        .lines()
        .nth(line.saturating_sub(1) as usize)
        .unwrap_or_default()
}

fn mask_preprocessor_directives(source: &str) -> String {
    let mut masked = String::with_capacity(source.len());

    for segment in source.split_inclusive('\n') {
        let line = segment.strip_suffix('\n').unwrap_or(segment);
        if is_preprocessor_directive(line) {
            masked.push_str(&neutralize_preprocessor_directive(line));
        } else {
            masked.push_str(line);
        }
        if segment.ends_with('\n') {
            masked.push('\n');
        }
    }

    masked
}

fn is_preprocessor_directive(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with('\\') || trimmed.starts_with("//") {
        return false;
    }

    let Some((name, rest)) = trimmed.split_once("::") else {
        return false;
    };
    matches!(
        name,
        "include"
            | "ifdef"
            | "ifndef"
            | "ifeval"
            | "endif"
            | "else"
            | "elsifdef"
            | "elsifndef"
    ) && rest.ends_with(']')
}

fn neutralize_preprocessor_directive(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut replaced = false;

    for ch in line.chars() {
        if !replaced && !ch.is_whitespace() {
            out.push('x');
            replaced = true;
        } else {
            out.push(ch);
        }
    }

    out
}

/// Fallback: simple line-by-line AsciiDoc scanner used when ACDC cannot parse
/// the file.  Splits on `----`/`....`/`++++` fences and `== …` section headers.
fn parse_adoc_raw_simple(source: &str) -> Vec<(u32, u32, &'static str, String)> {
    let mut blocks: Vec<(u32, u32, &'static str, String)> = Vec::new();

    let mut in_delim = false;
    let mut delim_start = 0u32;
    let mut delim_buf = String::new();

    let mut para_start = 0u32;
    let mut para_buf = String::new();

    let flush_para = |para_start: u32,
                      para_buf: &mut String,
                      current_line: u32,
                      blocks: &mut Vec<_>| {
        if !para_buf.is_empty() {
            let content = std::mem::take(para_buf);
            let end = current_line - 1;
            blocks.push((para_start, end.max(para_start), "para", content));
        }
    };

    for (idx, line) in source.lines().enumerate() {
        let lineno = idx as u32 + 1;

        if in_delim {
            delim_buf.push_str(line);
            delim_buf.push('\n');
            if is_adoc_fence(line) {
                let content = std::mem::take(&mut delim_buf);
                blocks.push((delim_start, lineno, "code", content));
                in_delim = false;
            }
            continue;
        }

        if is_adoc_fence(line) {
            flush_para(para_start, &mut para_buf, lineno, &mut blocks);
            in_delim = true;
            delim_start = lineno;
            delim_buf.push_str(line);
            delim_buf.push('\n');
            continue;
        }

        if is_adoc_section_header(line) {
            flush_para(para_start, &mut para_buf, lineno, &mut blocks);
            blocks.push((lineno, lineno, "section", line.to_string()));
            continue;
        }

        if line.trim().is_empty() {
            flush_para(para_start, &mut para_buf, lineno, &mut blocks);
            continue;
        }

        if para_buf.is_empty() {
            para_start = lineno;
        }
        para_buf.push_str(line);
        para_buf.push('\n');
    }

    let total_lines = source.lines().count() as u32;
    if in_delim && !delim_buf.is_empty() {
        blocks.push((delim_start, total_lines, "code", delim_buf));
    } else if !para_buf.is_empty() {
        blocks.push((para_start, total_lines, "para", para_buf));
    }

    blocks
}

pub(in crate::block_parser) fn is_adoc_fence(line: &str) -> bool {
    let t = line.trim_end();
    (t.starts_with("----") && t.chars().all(|c| c == '-'))
        || (t.starts_with("....") && t.chars().all(|c| c == '.'))
        || (t.starts_with("++++") && t.chars().all(|c| c == '+'))
}

pub(in crate::block_parser) fn is_adoc_section_header(line: &str) -> bool {
    let mut chars = line.chars();
    if chars.next() != Some('=') {
        return false;
    }
    let rest: String = chars.collect();
    let trimmed = rest.trim_start_matches('=');
    trimmed.starts_with(' ') || trimmed.is_empty()
}

pub(in crate::block_parser) fn has_unclosed_adoc_fence(source: &str) -> bool {
    let mut current: Option<&str> = None;

    for line in source.lines() {
        if !is_adoc_fence(line) {
            continue;
        }

        let fence = line.trim_end();
        match current {
            Some(open) if open == fence => current = None,
            Some(_) => {}
            None => current = Some(fence),
        }
    }

    current.is_some()
}
