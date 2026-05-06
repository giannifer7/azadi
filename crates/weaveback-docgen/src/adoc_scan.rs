// weaveback-docgen/src/adoc_scan.rs
// I'd Really Rather You Didn't edit this generated file.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AdocListingBlock {
    pub start: usize,
    pub end: usize,
    pub content: String,
}
pub(crate) fn collect_listing_blocks_by_language(
    source: &str,
    language: &str,
    label: &str,
) -> Vec<AdocListingBlock> {
    let masked = mask_preprocessor_directives(source);
    let doc = match acdc_parser::Parser::new(&masked).parse() {
        Ok(doc) => doc,
        Err(err) => {
            eprintln!(
                "adoc scan: {label}: ACDC failed while scanning for {language} blocks: {err}"
            );
            return Vec::new();
        }
    };

    let mut out = Vec::new();
    collect_from_blocks(&doc.blocks, source, language, &mut out);
    out
}
fn collect_from_blocks(
    blocks: &[acdc_parser::Block],
    source: &str,
    language: &str,
    out: &mut Vec<AdocListingBlock>,
) {
    use acdc_parser::{Block, DelimitedBlockType};

    for block in blocks {
        match block {
            Block::Section(section) => {
                collect_from_blocks(&section.content, source, language, out);
            }
            Block::DelimitedBlock(delimited) => {
                let is_verbatim = matches!(
                    delimited.inner,
                    DelimitedBlockType::DelimitedListing(_)
                        | DelimitedBlockType::DelimitedLiteral(_)
                        | DelimitedBlockType::DelimitedPass(_)
                );

                if is_verbatim && block_language(delimited).as_deref() == Some(language) {
                    if let Some(content) = delimited_content(source, delimited) {
                        let end = delimited
                            .location
                            .absolute_end
                            .saturating_add(1)
                            .min(source.len());
                        out.push(AdocListingBlock {
                            start: delimited.location.absolute_start,
                            end,
                            content,
                        });
                    }
                    continue;
                }

                match &delimited.inner {
                    DelimitedBlockType::DelimitedExample(inner)
                    | DelimitedBlockType::DelimitedOpen(inner)
                    | DelimitedBlockType::DelimitedSidebar(inner)
                    | DelimitedBlockType::DelimitedQuote(inner) => {
                        collect_from_blocks(inner, source, language, out);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
fn block_language(block: &acdc_parser::DelimitedBlock) -> Option<String> {
    let style = block.metadata.style.as_deref()?;
    if style != "source" {
        return Some(style.to_string());
    }

    for (name, value) in block.metadata.attributes.iter() {
        if matches!(value, acdc_parser::AttributeValue::None) {
            return Some(name.clone());
        }
    }

    block
        .metadata
        .attributes
        .get_string("language")
        .or_else(|| block.metadata.attributes.get_string("lang"))
}
fn delimited_content(source: &str, block: &acdc_parser::DelimitedBlock) -> Option<String> {
    let open = block.open_delimiter_location.as_ref()?;
    let close = block.close_delimiter_location.as_ref()?;
    let content_start = source[open.absolute_start.min(source.len())..]
        .find('\n')
        .map(|offset| open.absolute_start + offset + 1)
        .unwrap_or(open.absolute_end.min(source.len()));
    let content_end = close.absolute_start.min(source.len());

    if content_start > content_end || !source.is_char_boundary(content_start) {
        return None;
    }
    if !source.is_char_boundary(content_end) {
        return None;
    }

    Some(source[content_start..content_end].to_string())
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
#[cfg(test)]
mod tests;
