// weaveback-api/src/process/markdown_normalize.rs
// I'd Really Rather You Didn't edit this generated file.

pub(super) fn is_markdown_ext(expanded_ext: Option<&str>) -> bool {
    matches!(
        expanded_ext.unwrap_or_default().trim_start_matches('.'),
        "md" | "markdown"
    )
}

pub(in crate::process::markdown_normalize) fn is_asciidoc_ext(expanded_ext: Option<&str>) -> bool {
    matches!(
        expanded_ext.unwrap_or_default().trim_start_matches('.'),
        "adoc" | "asciidoc"
    )
}
mod adoc_table;
mod explicit_table;
mod markdown_table;

pub(crate) use adoc_table::normalize_adoc_tables_for_markdown;

use explicit_table::normalize_explicit_table_blocks;

fn markdown_inline_link_re() -> &'static regex::Regex {
    static RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| {
        regex::Regex::new(r"(\]\()([^)#\s]+)\.(adoc|wvb)((?:#[^)]+)?\))").unwrap()
    })
}

pub(in crate::process::markdown_normalize) fn normalize_markdown_projection_links(input: &str) -> String {
    let re = markdown_inline_link_re();
    re.replace_all(input, |caps: &regex::Captures<'_>| {
        let target = caps.get(2).map(|m| m.as_str()).unwrap_or_default();
        if target.starts_with("http://")
            || target.starts_with("https://")
            || target.starts_with("mailto:")
        {
            caps.get(0).map(|m| m.as_str()).unwrap_or_default().to_string()
        } else {
            format!(
                "{}{}.md{}",
                caps.get(1).map(|m| m.as_str()).unwrap_or_default(),
                target,
                caps.get(4).map(|m| m.as_str()).unwrap_or_default(),
            )
        }
    })
    .into_owned()
}
pub(crate) fn normalize_expanded_document(expanded_ext: Option<&str>, expanded: &[u8]) -> String {
    let expanded = String::from_utf8_lossy(expanded);
    let expanded = normalize_explicit_table_blocks(expanded_ext, &expanded);
    if is_markdown_ext(expanded_ext) {
        let expanded = normalize_adoc_tables_for_markdown(&expanded);
        normalize_markdown_projection_links(&expanded)
    } else {
        expanded
    }
}
