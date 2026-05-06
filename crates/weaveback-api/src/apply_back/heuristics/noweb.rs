// weaveback-api/src/apply_back/heuristics/noweb.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

pub(in crate::apply_back) fn resolve_noweb_entry(
    db: &WeavebackDb,
    out_file: &str,
    out_line_0: u32,
    resolver: &PathResolver,
) -> Result<Option<NowebMapEntry>, ApplyBackError> {
    if let Some(entry) =
        find_best_noweb_entry(db, out_file, out_line_0, resolver).map_err(ApplyBackError::Db)?
    {
        return Ok(Some(entry));
    }

    let resolved = resolver.resolve_gen(out_file);
    find_best_noweb_entry(db, resolved.to_string_lossy().as_ref(), out_line_0, resolver)
        .map_err(ApplyBackError::Db)
}
