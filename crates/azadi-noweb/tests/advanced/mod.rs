// crates/azadi-noweb/tests/advanced.rs
use super::common::{TestSetup, FILE_CHUNKS};
use azadi_noweb::AzadiError;

#[test]
fn test_file_chunk_detection() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(FILE_CHUNKS, "test_files.nw");

    let file_chunks = setup.clip.get_file_chunks();
    assert_eq!(file_chunks.len(), 1);
    assert!(file_chunks.contains(&"@file output.txt".to_string()));
}

#[test]
fn test_undefined_chunk_error() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(
        r#"
# <<main>>=
# <<nonexistent>>
# @
"#,
        "test_undefined.nw",
    );

    let result = setup.clip.expand("main", "");
    match result {
        Err(AzadiError::UndefinedChunk {
            chunk,
            file_name,
            line,
        }) => {
            assert_eq!(chunk, "nonexistent");
            assert_eq!(file_name, "test_undefined.nw");
            assert_eq!(line, 1);
        }
        _ => panic!("Expected UndefinedChunk error"),
    }
}

#[test]
fn test_recursive_chunk_error() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(
        r#"
# <<recursive>>=
Start
# <<recursive>>
End
# @
"#,
        "recursive_test.nw",
    );

    let result = setup.clip.expand("recursive", "");
    match result {
        Err(AzadiError::RecursiveReference {
            chunk,
            file_name,
            line,
        }) => {
            assert_eq!(chunk, "recursive");
            assert_eq!(file_name, "recursive_test.nw");
            assert_eq!(line, 2);
        }
        _ => panic!("Expected RecursiveReference error"),
    }
}

#[test]
fn test_mutual_recursion_error() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(
        r#"
# <<chunk-a>>=
Start A
# <<chunk-b>>
End A
# @

# <<chunk-b>>=
Middle B
# <<chunk-a>>
End B
# @
"#,
        "mutual_recursion.nw",
    );

    let result = setup.clip.expand("chunk-a", "");
    match result {
        Err(AzadiError::RecursiveReference {
            chunk,
            file_name,
            line,
        }) => {
            assert_eq!(file_name, "mutual_recursion.nw");
            assert_eq!(chunk, "chunk-a");
            assert_eq!(line, 8);
        }
        _ => panic!("Expected RecursiveReference error"),
    }
}

#[test]
fn test_max_recursion_depth() {
    let mut setup = TestSetup::new(&["#"]);

    let mut content = String::from(
        r#"
# <<a-000>>=
# <<a-001>>
# @"#,
    );

    let chain_length = 150; // More than MAX_DEPTH = 100
    for i in 1..chain_length {
        content.push_str(&format!(
            r#"
# <<a-{:03}>>=
# <<a-{:03}>>
# @"#,
            i,
            i + 1
        ));
    }

    setup.clip.read(&content, "max_recursion.nw");
    let result = setup.clip.expand("a-000", "");
    assert!(matches!(result, Err(AzadiError::RecursionLimit { .. })));
}

#[test]
fn test_error_messages_format() {
    let mut setup = TestSetup::new(&["#"]);
    setup.clip.read(
        r#"
# <<a>>=
# <<nonexistent>>
# @
"#,
        "errors.nw",
    );

    let err = setup.clip.expand("a", "").unwrap_err();
    let error_msg = err.to_string();

    assert!(error_msg.contains("Error: errors.nw line 2:"));
    assert!(error_msg.contains("chunk 'nonexistent' is undefined"));
}
