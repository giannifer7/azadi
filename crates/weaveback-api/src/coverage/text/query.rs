// weaveback-api/src/coverage/text/query.rs
// I'd Really Rather You Didn't edit this generated file.

use super::*;

pub fn run_impact(chunk: String, db_path: PathBuf) -> Result<(), CoverageApiError> {
    let json = crate::query::impact_analysis(&chunk, &db_path)?;
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
    Ok(())
}

pub fn run_graph(chunk: Option<String>, db_path: PathBuf) -> Result<(), CoverageApiError> {
    let dot = crate::query::chunk_graph_dot(chunk.as_deref(), &db_path)?;
    println!("{dot}");
    Ok(())
}

pub fn run_search(query: String, limit: usize, db_path: PathBuf) -> Result<(), CoverageApiError> {
    if !db_path.exists() {
        return Err(CoverageApiError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Database not found at {}. Run weaveback on your source files first.", db_path.display()),
        )));
    }
    let workspace = AgentWorkspace::open(AgentWorkspaceConfig {
        project_root: std::env::current_dir()?,
        db_path,
        gen_dir: PathBuf::from("gen"),
    });
    let results = workspace.session().search(&query, limit)
        .map_err(|e| CoverageApiError::Io(std::io::Error::other(e)))?;
    if results.is_empty() {
        println!("No results for {:?}", query);
        return Ok(());
    }
    for r in &results {
        let channels = if r.channels.is_empty() {
            String::new()
        } else {
            format!(" via {}", r.channels.join("+"))
        };
        if r.tags.is_empty() {
            println!(
                "{}:{}-{} [{}]{channels}",
                r.src_file,
                r.line_start,
                r.line_end,
                r.block_type,
            );
        } else {
            println!(
                "{}:{}-{} [{}]{channels}  #{}",
                r.src_file,
                r.line_start,
                r.line_end,
                r.block_type,
                r.tags.join(","),
            );
        }
        println!("  {}", r.snippet);
        println!();
    }
    Ok(())
}

pub fn run_tags(file: Option<String>, db_path: PathBuf) -> Result<(), CoverageApiError> {
    let blocks = crate::query::list_block_tags(file.as_deref(), &db_path)?;
    if blocks.is_empty() {
        println!("No tagged blocks found. Add a [tags] section to weaveback.toml and run wb-tangle.");
        return Ok(());
    }
    let mut current_file = String::new();
    for b in &blocks {
        if b.src_file != current_file {
            println!("\n{}", b.src_file);
            current_file = b.src_file.clone();
        }
        println!("  :{} [{}]  #{}", b.line_start, b.block_type, b.tags);
    }
    println!();
    Ok(())
}

pub fn run_trace(
    out_file: String,
    line: u32,
    col: u32,
    db_path: PathBuf,
    gen_dir: PathBuf,
    eval_config: weaveback_macro::evaluator::EvalConfig
) -> Result<(), CoverageApiError> {
    let db = open_db(&db_path)?;
    let project_root = std::env::current_dir().unwrap_or_default();
    let resolver = PathResolver::new(project_root, gen_dir);

    match lookup::perform_trace(&out_file, line, col, &db, &resolver, eval_config) {
        Ok(Some(json)) => {
            println!("{}", serde_json::to_string_pretty(&json).unwrap());
            Ok(())
        }
        Ok(None) => {
            eprintln!("No mapping found for {}:{}", out_file, line);
            Ok(())
        }
        Err(lookup::LookupError::InvalidInput(msg)) => {
            Err(CoverageApiError::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, msg)))
        }
        Err(lookup::LookupError::Db(e)) => Err(CoverageApiError::Noweb(WeavebackError::Db(e))),
        Err(lookup::LookupError::Io(e)) => Err(CoverageApiError::Io(e)),
    }
}
