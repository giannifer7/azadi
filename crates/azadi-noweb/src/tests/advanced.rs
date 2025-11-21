// src/tests/advanced.rs

use super::*;
use crate::AzadiError;
use crate::ChunkError;

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
        "undefined.nw",
    );

    let result = setup.clip.expand("main", "");
    match result {
        // Note how we capture `file_name` in the pattern
        Err(AzadiError::Chunk(ChunkError::UndefinedChunk {
            chunk,
            file_name,
            location,
        })) => {
            assert_eq!(chunk, "nonexistent");
            assert_eq!(file_name, "undefined.nw");
            assert_eq!(location.line, 1);
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
        "recursive.nw",
    );

    let result = setup.clip.expand("recursive", "");
    match result {
        // Also capture file_name
        Err(AzadiError::Chunk(ChunkError::RecursiveReference {
            chunk,
            file_name,
            location,
        })) => {
            assert_eq!(chunk, "recursive");
            assert_eq!(file_name, "recursive.nw");
            assert_eq!(location.line, 2);
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
        Err(AzadiError::Chunk(ChunkError::RecursiveReference {
            chunk,
            file_name,
            location,
        })) => {
            assert_eq!(chunk, "chunk-a");
            assert_eq!(file_name, "mutual_recursion.nw");
            assert_eq!(location.line, 8);
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
            i,     // a-001, a-002, etc.
            i + 1  // a-002, a-003, etc.
        ));
    }

    setup.clip.read(&content, "max_recursion.nw");
    let result = setup.clip.expand("a-000", "");

    // We just match the variant here (less strict). Alternatively, pattern match with { chunk, file_name, location }
    assert!(
        matches!(
            result,
            Err(AzadiError::Chunk(ChunkError::RecursionLimit { .. }))
        ),
        "Expected RecursionLimit error"
    );
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
    assert!(error_msg.contains("referenced chunk 'nonexistent' is undefined"));
}

#[test]
fn test_dangerous_comment_markers() {
    let markers = &[
        "#",         // normal case
        r".*",       // regex wildcard
        r"[a-z]+",   // regex character class
        r"\d+",      // regex digit
        "<<",        // same as delimiter
        ">>",        // same as delimiter
        "(comment)", // regex group
    ];

    let content = r#"
#<<test1>>=
Content1
@
.*<<test2>>=
Content2
@
[a-z]+<<test3>>=
Content3
@
(comment)<<test4>>=
Content4
@
"#;

    let mut setup = TestSetup::new(markers);
    setup.clip.read(content, "regex_test.nw");

    assert!(setup.clip.has_chunk("test1"), "Basic marker # failed");
    assert!(setup.clip.has_chunk("test2"), "Wildcard marker .* failed");
    assert!(
        setup.clip.has_chunk("test3"),
        "Character class marker [a-z]+ failed"
    );
    assert!(
        setup.clip.has_chunk("test4"),
        "Group marker (comment) failed"
    );

    assert_eq!(
        setup.clip.get_chunk_content("test1").unwrap(),
        vec!["Content1\n"]
    );
}
