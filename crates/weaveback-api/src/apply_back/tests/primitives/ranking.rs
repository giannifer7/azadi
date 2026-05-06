// weaveback-api/src/apply_back/tests/primitives/ranking.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

// ── rank_candidate ─────────────────────────────────────────────────────

#[test]
fn rank_candidate_closer_line_scores_higher() {
    let score_close = rank_candidate(5, 5, "old foo", "old foo", "new foo", 0);
    let score_far   = rank_candidate(5, 15, "old foo", "old foo", "new foo", 0);
    assert!(score_close > score_far);
}

// ── choose_best_candidate ──────────────────────────────────────────────

#[test]
fn choose_best_candidate_returns_highest_score() {
    let candidates = vec![
        CandidateResolution { line_idx: 0, new_line: "a".into(), score: 10 },
        CandidateResolution { line_idx: 1, new_line: "b".into(), score: 99 },
        CandidateResolution { line_idx: 2, new_line: "c".into(), score: 5  },
    ];
    let best = choose_best_candidate(candidates).unwrap();
    assert_eq!(best.line_idx, 1);
    assert_eq!(best.score, 99);
}

#[test]
fn choose_best_candidate_returns_none_on_tie() {
    let candidates = vec![
        CandidateResolution { line_idx: 0, new_line: "a".into(), score: 50 },
        CandidateResolution { line_idx: 1, new_line: "b".into(), score: 50 },
    ];
    assert!(choose_best_candidate(candidates).is_none());
}

#[test]
fn choose_best_candidate_returns_none_when_empty() {
    let result = choose_best_candidate(vec![]);
    assert!(result.is_none());
}
