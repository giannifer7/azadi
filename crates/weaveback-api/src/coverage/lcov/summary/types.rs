// weaveback-api/src/coverage/lcov/summary/types.rs
// I'd Really Rather You Didn't edit this generated file.

#[derive(Default)]
pub(super) struct SectionSummary {
    pub(super) total_lines: usize,
    pub(super) covered_lines: usize,
    pub(super) missed_lines: usize,
    pub(super) chunks: std::collections::BTreeSet<String>,
    pub(super) generated_lines: Vec<serde_json::Value>,
    pub(super) prose: Option<String>,
    pub(super) range: Option<serde_json::Value>,
    pub(super) breadcrumb: Vec<String>,
}

#[derive(Default)]
pub(super) struct SourceSummary {
    pub(super) total_lines: usize,
    pub(super) covered_lines: usize,
    pub(super) missed_lines: usize,
    pub(super) chunks: std::collections::BTreeSet<String>,
    pub(super) sections: std::collections::BTreeMap<String, SectionSummary>,
}

#[derive(Default)]
pub(super) struct UnattributedSummary {
    pub(super) total_lines: usize,
    pub(super) covered_lines: usize,
    pub(super) missed_lines: usize,
    pub(super) has_noweb_entries: bool,
    pub(super) mapped_line_start: Option<u32>,
    pub(super) mapped_line_end: Option<u32>,
    pub(super) generated_lines: Vec<serde_json::Value>,
}
