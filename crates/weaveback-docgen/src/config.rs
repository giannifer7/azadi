// weaveback-docgen/src/config.rs
// I'd Really Rather You Didn't edit this generated file.

use std::path::Path;


#[derive(serde::Deserialize, Default)]
pub(super) struct DocsConfig {
    pub(super) d2_theme: Option<u32>,
    pub(super) d2_layout: Option<String>,
}

#[derive(serde::Deserialize, Default)]
pub(super) struct WeavebackConfig {
    pub(super) docs: Option<DocsConfig>,
}

pub(super) fn read_config(root: &Path) -> WeavebackConfig {
    let path = root.join("weaveback.toml");
    if let Ok(content) = std::fs::read_to_string(&path) {
        toml::from_str(&content).unwrap_or_default()
    } else {
        WeavebackConfig::default()
    }
}
