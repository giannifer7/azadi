// <[@file crates/azadi-macros/src/evaluator/source_utils.rs]>=
// azadi/crates/azadi-macros/src/evaluator/source_utils.rs

use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// Create a backup of `source_file` under `backup_dir`.
pub fn backup_source_file(source_file: &Path, backup_dir: &Path) -> io::Result<()> {
    let abs_source = source_file.canonicalize()?;
    // If you need to strip a certain prefix, adapt here:
    let rel = abs_source.strip_prefix("/").unwrap_or(&abs_source);
    let backup_path = backup_dir.join(rel);
    if let Some(parent) = backup_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&abs_source, &backup_path)?;
    Ok(())
}

/// Modify `source_file` by inserting text at byte offsets, optionally skipping to newline.
pub fn modify_source(
    source_file: &Path,
    insertions: &[(usize, Vec<u8>, bool)],
    backup_dir: Option<&Path>,
) -> io::Result<()> {
    if let Some(dir) = backup_dir {
        backup_source_file(source_file, dir)?;
    }
    let content = fs::read(source_file)?;
    let mut result = Vec::new();
    let mut last_pos = 0usize;

    let mut sorted = insertions.to_vec();
    sorted.sort_by_key(|(pos, _, _)| *pos);

    use std::cmp;
    for (pos, text, skip_to_newline) in sorted {
        if pos < content.len() {
            result.extend_from_slice(&content[last_pos..pos]);
        } else {
            result.extend_from_slice(&content[last_pos..]);
        }
        result.extend_from_slice(&text);
        if skip_to_newline {
            let mut idx = pos;
            while idx < content.len() && content[idx] != b'\n' {
                idx += 1;
            }
            if idx < content.len() {
                idx += 1; // skip actual newline
            }
            last_pos = idx;
        } else {
            last_pos = cmp::min(pos, content.len());
        }
    }
    if last_pos < content.len() {
        result.extend_from_slice(&content[last_pos..]);
    }

    let mut f = fs::File::create(source_file)?;
    f.write_all(&result)?;
    Ok(())
}
