// weaveback-docgen/src/xref_cmd.rs
// I'd Really Rather You Didn't edit this generated file.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use super::error::Error;
use super::xref::XrefEntry;

pub(super) fn run_xref_cmd(cmd: &str, project_root: &Path) -> Result<HashMap<String, XrefEntry>, Error> {
    let output = Command::new(cmd)
        .arg(project_root)
        .output()
        .map_err(|source| Error::XrefCommandSpawn {
            cmd: cmd.to_string(),
            source,
        });
    let output = output?;
    if !output.status.success() {
        let code = output.status.code().unwrap_or(1);
        return Err(Error::XrefCommandStatus { cmd: cmd.to_string(), code });
    }
    serde_json::from_slice(&output.stdout).map_err(|source| Error::XrefCommandJson {
        cmd: cmd.to_string(),
        source,
    })
}
