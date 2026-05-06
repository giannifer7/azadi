// weaveback-api/src/apply_back/heuristics/search.rs
// I'd Really Rather You Didn't edit this generated file.

use super::body::attempt_macro_body_fix;
use super::macro_arg::attempt_macro_arg_patch;
use super::ranking::{candidate_line_indices, choose_best_candidate, chunk_context_bonus, rank_candidate};
use super::*;

pub(in crate::apply_back) fn search_macro_arg_candidate(request: MacroArgSearch<'_>) -> Option<CandidateResolution> {
    let candidate_indices = candidate_line_indices(
        request.lines,
        request.hinted_line,
        None,
        request.old_text,
    );
    let mut candidates = Vec::new();

    for idx in candidate_indices {
        let Some(new_line) = attempt_macro_arg_patch(
            request.lines,
            idx,
            request.src_col,
            request.old_text,
            request.new_text,
        ) else {
            continue;
        };
        let candidate_src = splice_line(request.lines, idx, &new_line, true);
        if !verify_candidate(
            &candidate_src,
            request.src_path,
            request.eval_config,
            request.expanded_line,
            request.new_text,
        ) {
            continue;
        }
        candidates.push(CandidateResolution {
            line_idx: idx,
            new_line,
            score: rank_candidate(
                request.hinted_line,
                idx,
                &request.lines[idx],
                request.old_text,
                request.new_text,
                chunk_context_bonus(
                    request.db,
                    &request.src_path.to_string_lossy(),
                    request.hinted_line,
                    idx,
                ),
            ),
        });
    }

    choose_best_candidate(candidates)
}

pub(in crate::apply_back) fn search_macro_body_candidate(request: MacroBodySearch<'_>) -> Option<CandidateResolution> {
    let anchor = request.body_template.unwrap_or(request.old_text);
    let candidate_indices = candidate_line_indices(
        request.lines,
        request.hinted_line,
        Some(anchor),
        request.old_text,
    );
    let mut candidates = Vec::new();

    for idx in candidate_indices {
        let template = request.body_template.unwrap_or(request.lines.get(idx)?.as_str());
        let Some(new_line) = attempt_macro_body_fix(
            template,
            request.old_text,
            request.new_text,
            request.sigil,
        ) else {
            continue;
        };
        let candidate_src = splice_line(request.lines, idx, &new_line, true);
        if !verify_candidate(
            &candidate_src,
            request.src_path,
            request.eval_config,
            request.expanded_line,
            request.new_text,
        ) {
            continue;
        }
        candidates.push(CandidateResolution {
            line_idx: idx,
            new_line,
            score: rank_candidate(
                request.hinted_line,
                idx,
                &request.lines[idx],
                request.old_text,
                request.new_text,
                chunk_context_bonus(
                    request.db,
                    &request.src_path.to_string_lossy(),
                    request.hinted_line,
                    idx,
                ),
            ),
        });
    }

    choose_best_candidate(candidates)
}

pub(in crate::apply_back) fn search_macro_call_candidate(request: MacroCallSearch<'_>) -> Option<CandidateResolution> {
    let needle = format!("{}{}(", request.sigil, request.macro_name);
    let mut candidates = Vec::new();
    let token_pair = differing_token_pair(request.old_text, request.new_text);

    for (idx, line) in request.lines.iter().enumerate() {
        if !line.contains(&needle) {
            continue;
        }
        if let Some(new_line) = attempt_macro_arg_patch(
            request.lines,
            idx,
            0,
            request.old_text,
            request.new_text,
        ) {
            let candidate_src = splice_line(request.lines, idx, &new_line, true);
            if verify_candidate(
                &candidate_src,
                request.src_path,
                request.eval_config,
                request.expanded_line,
                request.new_text,
            ) {
                candidates.push(CandidateResolution {
                    line_idx: idx,
                    new_line,
                    score: 80 + token_overlap_score(line, request.old_text, request.new_text),
                });
            }
        }

        if let Some((ref old_token, ref new_token)) = token_pair {
            for (pos, _) in line.match_indices(old_token) {
                let before_ok = pos == 0 || !line[..pos].chars().last().is_some_and(|ch| ch.is_alphanumeric() || ch == '_');
                let after_pos = pos + old_token.len();
                let after_ok = after_pos == line.len() || !line[after_pos..].chars().next().is_some_and(|ch| ch.is_alphanumeric() || ch == '_');
                if !(before_ok && after_ok) {
                    continue;
                }

                let mut token_line = line.clone();
                token_line.replace_range(pos..after_pos, new_token);
                let candidate_src = splice_line(request.lines, idx, &token_line, true);
                if !verify_candidate(
                    &candidate_src,
                    request.src_path,
                    request.eval_config,
                    request.expanded_line,
                    request.new_text,
                ) {
                    continue;
                }
                candidates.push(CandidateResolution {
                    line_idx: idx,
                    new_line: token_line,
                    score: 95 + token_overlap_score(line, request.old_text, request.new_text),
                });
            }
        }
    }

    choose_best_candidate(candidates)
}
