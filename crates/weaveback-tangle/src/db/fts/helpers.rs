// weaveback-tangle/src/db/fts/helpers.rs
// I'd Really Rather You Didn't edit this generated file.

pub(super) fn normalise_snapshot_path(raw: &str, cwd: &std::path::Path) -> String {
    let p = std::path::Path::new(raw);
    if p.is_absolute() {
        if let Ok(rel) = p.strip_prefix(cwd) {
            return rel.to_string_lossy().into_owned();
        }
        return raw.to_string();
    }
    raw.strip_prefix("./").map(str::to_owned).unwrap_or_else(|| raw.to_string())
}

pub(super) fn prose_snippet(content: &str) -> String {
    let compact = content.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = compact.chars();
    let prefix: String = chars.by_ref().take(160).collect();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

pub(super) fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.is_empty() || a.len() != b.len() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;
    for (&lhs, &rhs) in a.iter().zip(b.iter()) {
        dot += lhs * rhs;
        norm_a += lhs * lhs;
        norm_b += rhs * rhs;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom <= f32::EPSILON {
        0.0
    } else {
        dot / denom
    }
}

