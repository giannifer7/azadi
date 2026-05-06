// weaveback-api/src/coverage/lcov/summary/unattributed.rs
// I'd Really Rather You Didn't edit this generated file.

use super::types::UnattributedSummary;
use super::super::*;

pub(super) fn record_unattributed_line(
    file_name: &str,
    line_no: u32,
    hit_count: u64,
    noweb_map: &std::collections::HashMap<u32, weaveback_tangle::db::NowebMapEntry>,
    unattributed: &mut Vec<serde_json::Value>,
    unattributed_grouped: &mut std::collections::BTreeMap<String, UnattributedSummary>,
) {
    let covered = hit_count > 0;
    let mapped_line_start = noweb_map.keys().min().copied().map(|line_0| line_0 + 1);
    let mapped_line_end = noweb_map.keys().max().copied().map(|line_0| line_0 + 1);
    let generated_line = json!({
        "generated_file": file_name,
        "generated_line": line_no,
        "hit_count": hit_count,
        "covered": covered,
        "has_noweb_entries": !noweb_map.is_empty(),
        "mapped_line_start": mapped_line_start,
        "mapped_line_end": mapped_line_end,
    });

    unattributed.push(generated_line.clone());
    let file = unattributed_grouped.entry(file_name.to_string()).or_default();
    file.total_lines += 1;
    if covered {
        file.covered_lines += 1;
    } else {
        file.missed_lines += 1;
    }
    file.has_noweb_entries |= !noweb_map.is_empty();
    file.mapped_line_start = match (file.mapped_line_start, mapped_line_start) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (None, b) => b,
        (a, None) => a,
    };
    file.mapped_line_end = match (file.mapped_line_end, mapped_line_end) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (None, b) => b,
        (a, None) => a,
    };
    file.generated_lines.push(generated_line);
}
