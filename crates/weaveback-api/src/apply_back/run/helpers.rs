// weaveback-api/src/apply_back/run/helpers.rs
// I'd Really Rather You Didn't edit this generated file.

use super::super::*;

pub(super) struct FileEvalSettings {
    pub(super) eval_config: Option<EvalConfig>,
    pub(super) sigil: char,
}

pub(super) struct ContiguousNowebHunk {
    pub(super) first: NowebMapEntry,
}

pub(super) fn resolve_gen_dir(opts: &ApplyBackOptions, db: &WeavebackDb) -> Result<PathBuf, ApplyBackError> {
    let default_gen = PathBuf::from("gen");
    if opts.gen_dir == default_gen && !default_gen.exists() {
        Ok(db
            .get_run_config("gen_dir")?
            .map(PathBuf::from)
            .unwrap_or_else(|| opts.gen_dir.clone()))
    } else {
        Ok(opts.gen_dir.clone())
    }
}

pub(super) fn project_root_from_db_path(db_path: &std::path::Path) -> PathBuf {
    db_path
        .canonicalize()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

pub(super) fn selected_baselines(
    db: &WeavebackDb,
    files: &[String],
) -> Result<Vec<(String, Vec<u8>)>, ApplyBackError> {
    if files.is_empty() {
        return db.list_baselines().map_err(ApplyBackError::from);
    }

    Ok(files
        .iter()
        .filter_map(|f| db.get_baseline(f).ok().flatten().map(|b| (f.clone(), b)))
        .collect())
}

pub(super) fn file_eval_settings(
    db: &WeavebackDb,
    src_file: &str,
    base_eval_config: Option<EvalConfig>,
    default_sigil: char,
) -> FileEvalSettings {
    let mut eval_config = base_eval_config;
    let mut sigil = default_sigil;
    if let Ok(Some(cfg)) = weaveback_tangle::lookup::find_best_source_config(db, src_file) {
        if eval_config.is_none() {
            eval_config = Some(EvalConfig::default());
        }
        if let Some(ec) = &mut eval_config {
            ec.sigil = cfg.sigil;
        }
        sigil = cfg.sigil;
    }

    FileEvalSettings { eval_config, sigil }
}

pub(super) fn resolve_contiguous_noweb_hunk(
    db: &WeavebackDb,
    rel_path: &str,
    old_index: usize,
    old_len: usize,
    resolver: &PathResolver,
) -> Result<Option<ContiguousNowebHunk>, ApplyBackError> {
    let mut hunk_entries = Vec::new();
    for i in 0..old_len {
        hunk_entries.push(resolve_noweb_entry(db, rel_path, (old_index + i) as u32, resolver)?);
    }

    if !hunk_entries.iter().all(|e| e.is_some()) {
        return Ok(None);
    }

    let entries: Vec<_> = hunk_entries.into_iter().flatten().collect();
    let first = entries[0].clone();
    let is_contiguous = entries
        .iter()
        .all(|e| e.src_file == first.src_file && e.indent == first.indent)
        && entries.windows(2).all(|w| w[1].src_line == w[0].src_line + 1);

    if is_contiguous {
        Ok(Some(ContiguousNowebHunk { first }))
    } else {
        Ok(None)
    }
}

pub(super) struct ApplyCollectedPatchesCtx<'a> {
    pub(super) db: &'a WeavebackDb,
    pub(super) src_patches: &'a HashMap<String, Vec<Patch>>,
    pub(super) snapshot_cache: &'a mut HashMap<String, Option<Vec<u8>>>,
    pub(super) project_root: &'a std::path::Path,
    pub(super) dry_run: bool,
    pub(super) base_eval_config: Option<EvalConfig>,
    pub(super) default_sigil: char,
}

pub(super) fn apply_collected_patches(
    ctx: ApplyCollectedPatchesCtx<'_>,
    skipped: &mut usize,
    out: &mut dyn Write,
) -> Result<(), ApplyBackError> {
    for (src_file, patches) in ctx.src_patches {
        let snap = ctx
            .snapshot_cache
            .entry(src_file.clone())
            .or_insert_with(|| ctx.db.get_src_snapshot(src_file).ok().flatten())
            .as_deref();

        let settings = file_eval_settings(
            ctx.db,
            src_file,
            ctx.base_eval_config.clone(),
            ctx.default_sigil,
        );

        apply_patches_to_file(
            FilePatchContext {
                db: ctx.db,
                src_file,
                src_root: ctx.project_root,
                patches,
                dry_run: ctx.dry_run,
                eval_config: settings.eval_config,
                snapshot: snap,
                sigil: settings.sigil,
            },
            skipped,
            out,
        )?;
    }

    Ok(())
}
