// weaveback-api/src/lookup/context.rs
// I'd Really Rather You Didn't edit this generated file.

use serde_json::Value;

pub fn build_source_context_value(src_content: &str, src_line_1: usize) -> Value {
    let mut obj = serde_json::Map::new();
    append_source_context(&mut obj, src_content, src_line_1);
    Value::Object(obj)
}

fn heading_level(line: &str) -> Option<usize> {
    let trimmed = line.trim_end();
    if trimmed.is_empty() {
        return None;
    }
    let count = trimmed.bytes().take_while(|&byte| byte == b'=').count();
    if count > 0 && trimmed.len() > count && trimmed.as_bytes()[count] == b' ' {
        Some(count)
    } else {
        None
    }
}

fn section_range(lines: &[&str], line_1: usize) -> (usize, usize) {
    let anchor = line_1.saturating_sub(1).min(lines.len().saturating_sub(1));
    let mut sec_start = 0usize;
    let mut sec_level = 1usize;
    for idx in (0..=anchor).rev() {
        if let Some(level) = heading_level(lines[idx]) {
            sec_start = idx;
            sec_level = level;
            break;
        }
    }

    let sec_end = lines
        .iter()
        .enumerate()
        .skip(anchor.saturating_add(1))
        .find(|(_, line)| heading_level(line).is_some_and(|level| level <= sec_level))
        .map(|(idx, _)| idx)
        .unwrap_or(lines.len());

    (sec_start, sec_end)
}

fn title_chain(lines: &[&str], line_1: usize) -> Vec<String> {
    let anchor = line_1.saturating_sub(1).min(lines.len().saturating_sub(1));
    let mut chain = Vec::new();
    for line in lines.iter().take(anchor.saturating_add(1)) {
        if let Some(level) = heading_level(line) {
            let title = line[level + 1..].trim().to_string();
            chain.retain(|(existing_level, _): &(usize, String)| *existing_level < level);
            chain.push((level, title));
        }
    }
    chain.into_iter().map(|(_, title)| title).collect()
}

fn extract_prose(lines: &[&str], start: usize, end: usize) -> String {
    let end = end.min(lines.len());
    let mut in_fence = false;
    let mut in_chunk = false;
    let mut out = Vec::new();

    for line in lines.iter().take(end).skip(start) {
        let trimmed = line.trim();
        if trimmed == "----" {
            in_fence = !in_fence;
            continue;
        }
        if trimmed.starts_with("// <<") && trimmed.ends_with(">>=") {
            in_chunk = true;
            continue;
        }
        if trimmed == "// @" {
            in_chunk = false;
            continue;
        }
        if !in_fence && !in_chunk {
            out.push(*line);
        }
    }

    let mut collapsed = Vec::new();
    let mut prev_blank = false;
    for line in out {
        let blank = line.trim().is_empty();
        if blank && prev_blank {
            continue;
        }
        prev_blank = blank;
        collapsed.push(line);
    }
    while collapsed.first().is_some_and(|line| line.trim().is_empty()) {
        collapsed.remove(0);
    }
    while collapsed.last().is_some_and(|line| line.trim().is_empty()) {
        collapsed.pop();
    }
    collapsed.join("\n")
}

pub(in crate::lookup) fn append_source_context(
    obj: &mut serde_json::Map<String, Value>,
    src_content: &str,
    src_line_1: usize,
) {
    use serde_json::json;

    let lines: Vec<&str> = src_content.lines().collect();
    if lines.is_empty() {
        return;
    }
    let (sec_start, sec_end) = section_range(&lines, src_line_1);
    let section_breadcrumb = title_chain(&lines, src_line_1);
    let section_prose = extract_prose(&lines, sec_start, sec_end);
    obj.insert(
        "source_section_breadcrumb".into(),
        Value::Array(section_breadcrumb.into_iter().map(Value::String).collect()),
    );
    obj.insert(
        "source_section_range".into(),
        json!({
            "start_line": sec_start + 1,
            "end_line": sec_end,
        }),
    );
    obj.insert("source_section_prose".into(), Value::String(section_prose));
}
