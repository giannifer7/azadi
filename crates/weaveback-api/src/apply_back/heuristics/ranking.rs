// weaveback-api/src/apply_back/heuristics/ranking.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

pub(super) fn candidate_line_indices(
    lines: &[String],
    hinted: usize,
    anchor_text: Option<&str>,
    old_text: &str,
) -> Vec<usize> {
    let mut indices = Vec::new();
    let mut push_unique = |idx: usize| {
        if idx < lines.len() && !indices.contains(&idx) {
            indices.push(idx);
        }
    };

    push_unique(hinted);

    if let Some(anchor) = anchor_text
        && let Some(idx) = fuzzy_find_line(lines, hinted, anchor, 40)
    {
        push_unique(idx);
    }
    if let Some(idx) = fuzzy_find_line(lines, hinted, old_text, 40) {
        push_unique(idx);
    }

    let lo = hinted.saturating_sub(6);
    let hi = (hinted + 6).min(lines.len().saturating_sub(1));
    for idx in lo..=hi {
        push_unique(idx);
    }

    indices
}

pub(in crate::apply_back) fn rank_candidate(
    hinted: usize,
    idx: usize,
    current_line: &str,
    old_text: &str,
    new_text: &str,
    context_bonus: i32,
) -> i32 {
    let distance_penalty = hinted.abs_diff(idx) as i32 * 2;
    let mut score = 100 - distance_penalty + context_bonus;
    score += token_overlap_score(current_line, old_text, new_text);
    if current_line.contains(old_text) {
        score += 12;
    }
    score
}

pub(in crate::apply_back) fn choose_best_candidate(
    mut candidates: Vec<CandidateResolution>,
) -> Option<CandidateResolution> {
    candidates.sort_by(|left, right| {
        right.score.cmp(&left.score)
            .then_with(|| left.line_idx.cmp(&right.line_idx))
    });
    let best = candidates.first()?;
    if candidates.get(1).is_some_and(|next| next.score == best.score && next.line_idx != best.line_idx) {
        None
    } else {
        Some(candidates.remove(0))
    }
}

pub(super) fn chunk_context_bonus(
    db: &WeavebackDb,
    src_file: &str,
    hinted_line_0: usize,
    idx: usize,
) -> i32 {
    let Ok(defs) = db.query_chunk_defs_overlapping(src_file, hinted_line_0 as u32 + 1, hinted_line_0 as u32 + 1) else {
        return 0;
    };
    if defs.iter().any(|def| {
        let lo = def.def_start.saturating_sub(1) as usize;
        let hi = def.def_end.saturating_sub(1) as usize;
        idx >= lo && idx <= hi
    }) {
        20
    } else {
        0
    }
}
