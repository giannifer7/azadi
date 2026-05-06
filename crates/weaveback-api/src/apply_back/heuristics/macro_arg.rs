// weaveback-api/src/apply_back/heuristics/macro_arg.rs
// I'd Really Rather You Didn't edit this generated file.

/// For a `MacroArg` span: replace the changed portion at or after byte column `src_col`.
///
/// Primary strategy: exact match of `old_text` at `src_col` (works when `old_text` is
/// already the raw argument value).
///
/// Fallback: find the prefix where old/new expanded text first differ, then try
/// progressively shorter suffix lengths until we find an old fragment that actually
/// appears in the source from `src_col`.  This handles the common case where
/// `old_text` is the full expanded output line, not just the argument value — and
/// avoids false suffix matches when the old string is a suffix of the new one
/// (e.g. `literate` vs `illiterate`).
pub(in crate::apply_back) fn attempt_macro_arg_patch(
    lines: &[String],
    src_line: usize,
    src_col: u32,
    old_text: &str,
    new_text: &str,
) -> Option<String> {
    let line = lines.get(src_line)?;
    let col = src_col as usize;

    // Primary: exact col match.
    if col + old_text.len() <= line.len() && &line[col..col + old_text.len()] == old_text {
        let mut new_line = line.to_string();
        new_line.replace_range(col..col + old_text.len(), new_text);
        return Some(new_line);
    }

    // Fallback.
    let old_chars: Vec<char> = old_text.chars().collect();
    let new_chars: Vec<char> = new_text.chars().collect();

    // pfx: length of the common prefix between old and new.
    let pfx = old_chars.iter().zip(new_chars.iter())
        .take_while(|(a, b)| a == b).count();

    // max_sfx: upper bound on common suffix length.
    let max_sfx = old_chars.iter().rev().zip(new_chars.iter().rev())
        .take_while(|(a, b)| a == b).count();

    let search_start = col.min(line.len());
    let search_region = &line[search_start..];

    // Try increasing sfx values (longest fragment first) until we find an old_frag
    // that appears in the source.  Longest-first avoids false matches on short fragments
    // (e.g. a single "l" matching the wrong letter in the source line).
    for sfx in 0..=max_sfx {
        let end = old_chars.len().checked_sub(sfx)?;
        if pfx >= end { continue; }
        let old_frag: String = old_chars[pfx..end].iter().collect();
        if old_frag.is_empty() { continue; }

        if let Some(pos) = search_region.find(old_frag.as_str()) {
            let new_end = new_chars.len().checked_sub(sfx)?;
            if pfx > new_end { continue; }
            let new_frag: String = new_chars[pfx..new_end].iter().collect();
            let abs_pos = search_start + pos;
            let mut new_line = line.to_string();
            new_line.replace_range(abs_pos..abs_pos + old_frag.len(), &new_frag);
            return Some(new_line);
        }
    }
    None
}
