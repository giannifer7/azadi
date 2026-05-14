// weaveback-api/src/coverage/text/cargo_run.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

pub fn run_cargo_annotated(
    cargo_args: Vec<String>,
    diagnostics_only: bool,
    db_path: PathBuf,
    gen_dir: PathBuf,
    eval_config: EvalConfig,
) -> Result<(), CoverageApiError> {
    let project_root = std::env::current_dir().unwrap_or_default();
    let mut stdout_out = std::io::stdout().lock();
    run_cargo_annotated_to_writer(
        cargo_args,
        diagnostics_only,
        db_path,
        gen_dir,
        eval_config,
        &project_root,
        &mut stdout_out,
    )
}

pub fn run_cargo_annotated_to_writer(
    mut cargo_args: Vec<String>,
    diagnostics_only: bool,
    db_path: PathBuf,
    gen_dir: PathBuf,
    eval_config: EvalConfig,
    project_root: &Path,
    mut out: impl Write,
) -> Result<(), CoverageApiError> {
    if cargo_args.is_empty() {
        cargo_args.push("check".to_string());
    }
    if !cargo_args
        .iter()
        .any(|arg| arg.starts_with("--message-format"))
    {
        let message_format = "--message-format=json-diagnostic-rendered-ansi".to_string();
        if let Some(idx) = cargo_args.iter().position(|arg| arg == "--") {
            cargo_args.insert(idx, message_format);
        } else {
            cargo_args.push(message_format);
        }
    }

    let resolver = PathResolver::new(project_root.to_path_buf(), gen_dir);
    let db = if db_path.exists() {
        Some(weaveback_tangle::db::WeavebackDb::open_read_only(&db_path)?)
    } else {
        None
    };

    let cargo_bin = std::env::var("WEAVEBACK_CARGO_BIN").unwrap_or_else(|_| "cargo".to_string());
    let mut child = Command::new(cargo_bin)
        .args(&cargo_args)
        .current_dir(project_root)
        .stdin(Stdio::inherit())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(CoverageApiError::Io)?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| CoverageApiError::Io(std::io::Error::other("failed to capture cargo stdout")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| CoverageApiError::Io(std::io::Error::other("failed to capture cargo stderr")))?;
    let reader = BufReader::new(stdout);
    let err_reader = BufReader::new(stderr);
    let mut compiler_message_count = 0usize;
    let mut all_span_records = Vec::new();
    let (stderr_tx, stderr_rx) = mpsc::channel::<Result<String, std::io::Error>>();

    thread::spawn(move || {
        for line in err_reader.lines() {
            let _ = stderr_tx.send(line);
        }
    });

    for line in reader.lines() {
        let line = line.map_err(CoverageApiError::Io)?;
        let Ok(envelope) = serde_json::from_str::<CargoMessageEnvelope>(&line) else {
            let attributions =
                collect_text_attributions(&line, db.as_ref(), project_root, &resolver, &eval_config);
            if !attributions.is_empty() {
                emit_text_attribution_message("stdout", &line, attributions, &mut out)
                    .map_err(CoverageApiError::Io)?;
            } else if !diagnostics_only {
                writeln!(out, "{line}").map_err(CoverageApiError::Io)?;
            }
            continue;
        };

        if envelope.reason == "compiler-message"
            && let Some(diagnostic) = envelope.message
        {
            compiler_message_count += 1;
            let records =
                collect_cargo_attributions(
                    &diagnostic,
                    db.as_ref(),
                    project_root,
                    &resolver,
                    &eval_config,
                );
            let span_records = collect_cargo_span_attributions(
                &diagnostic,
                db.as_ref(),
                project_root,
                &resolver,
                &eval_config,
            );
            all_span_records.extend(span_records.iter().cloned());
            emit_augmented_cargo_message(&line, records, span_records, &mut out)
                .map_err(CoverageApiError::Io)?;
        } else if !diagnostics_only || envelope.reason == "build-finished" {
            writeln!(out, "{line}").map_err(CoverageApiError::Io)?;
        }
    }

    for line in stderr_rx {
        let line = line.map_err(CoverageApiError::Io)?;
        let attributions =
            collect_text_attributions(&line, db.as_ref(), project_root, &resolver, &eval_config);
        if !attributions.is_empty() {
            emit_text_attribution_message("stderr", &line, attributions, &mut out)
                .map_err(CoverageApiError::Io)?;
        } else if !diagnostics_only {
            writeln!(out, "{line}").map_err(CoverageApiError::Io)?;
        }
    }

    emit_cargo_summary_message(compiler_message_count, &all_span_records, &mut out)
        .map_err(CoverageApiError::Io)?;

    let status = child.wait().map_err(CoverageApiError::Io)?;
    if status.success() {
        Ok(())
    } else {
        Err(CoverageApiError::Io(std::io::Error::other(format!(
            "cargo exited with status {status}"
        ))))
    }
}
