use azadi_macros::evaluator::output::{MacroMapEntry, SpanKind};
use azadi_noweb::db::AzadiDb;
use std::path::Path;
use serde_json::{json, Value};

#[derive(Debug)]
pub enum LookupError {
    Db(azadi_noweb::db::DbError),
    Io(std::io::Error),
    InvalidInput(String),
}

impl From<azadi_noweb::db::DbError> for LookupError {
    fn from(e: azadi_noweb::db::DbError) -> Self {
        LookupError::Db(e)
    }
}

impl From<std::io::Error> for LookupError {
    fn from(e: std::io::Error) -> Self {
        LookupError::Io(e)
    }
}

pub fn perform_where(
    out_file: &str,
    line: u32,
    db: &AzadiDb,
    gen_dir: &Path,
) -> Result<Option<Value>, LookupError> {
    if line == 0 {
        return Err(LookupError::InvalidInput("Line number must be >= 1".to_string()));
    }
    let out_line_0 = line - 1;

    let db_lookup_path = normalize_path(out_file, gen_dir);

    if let Some(entry) = db.get_noweb_entry(&db_lookup_path, out_line_0)? {
        Ok(Some(json!({
            "out_file": out_file,
            "out_line": line,
            "chunk": entry.chunk_name,
            "expanded_file": entry.src_file,
            "expanded_line": entry.src_line + 1,
            "indent": entry.indent,
        })))
    } else {
        Ok(None)
    }
}

pub fn perform_trace(
    out_file: &str,
    line: u32,
    db: &AzadiDb,
    gen_dir: &Path,
) -> Result<Option<Value>, LookupError> {
    if line == 0 {
        return Err(LookupError::InvalidInput("Line number must be >= 1".to_string()));
    }
    let out_line_0 = line - 1;

    let db_lookup_path = normalize_path(out_file, gen_dir);

    let nw = db.get_noweb_entry(&db_lookup_path, out_line_0)?;
    if let Some(nw_entry) = nw {
        let mut result = json!({
            "out_file": out_file,
            "out_line": line,
            "chunk": nw_entry.chunk_name,
            "expanded_file": nw_entry.src_file,
            "expanded_line": nw_entry.src_line + 1,
            "indent": nw_entry.indent,
        });

        if let Ok(Some(bytes)) = db.get_macro_map_bytes(&nw_entry.src_file, nw_entry.src_line)
            && let Ok(m_entry) = postcard::from_bytes::<MacroMapEntry>(&bytes)
        {
            let obj = result.as_object_mut().unwrap();
            obj.insert("src_file".to_string(), Value::String(m_entry.src_file));
            obj.insert("src_line".to_string(), Value::Number((m_entry.src_line + 1).into()));
            obj.insert("src_col".to_string(), Value::Number(m_entry.src_col.into()));
            
            let (kind_str, extra) = match m_entry.kind {
                SpanKind::Literal => ("Literal", None),
                SpanKind::MacroBody { ref macro_name } => ("MacroBody", Some(("macro_name", macro_name.clone()))),
                SpanKind::MacroArg { ref macro_name, .. } => ("MacroArg", Some(("macro_name", macro_name.clone()))),
                SpanKind::VarBinding { ref var_name } => ("VarBinding", Some(("var_name", var_name.clone()))),
                SpanKind::Computed => ("Computed", None),
            };
            obj.insert("kind".to_string(), Value::String(kind_str.to_string()));
            if let Some((k, v)) = extra {
                obj.insert(k.to_string(), Value::String(v));
            }
        }
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

fn normalize_path(out_file: &str, gen_dir: &Path) -> String {
    let mut db_lookup_path = out_file.to_string();
    if let (Ok(canon_gen), Ok(canon_out)) = (gen_dir.canonicalize(), Path::new(out_file).canonicalize()) {
        if let Ok(rel) = canon_out.strip_prefix(&canon_gen) {
            db_lookup_path = rel.to_string_lossy().into_owned();
        }
    } else {
        let prefix = gen_dir.to_string_lossy();
        if out_file.starts_with(prefix.as_ref()) {
            let stripped = out_file.trim_start_matches(prefix.as_ref());
            db_lookup_path = stripped.trim_start_matches('/').to_string();
        }
    }
    db_lookup_path
}
