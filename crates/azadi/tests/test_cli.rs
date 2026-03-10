// crates/azadi/tests/test_cli.rs
//
// Integration tests for the azadi combined CLI, covering:
//   - --directory mode (auto-discovers drivers, skips %include'd fragments)
//   - --depfile and --stamp (build-system integration)

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, content: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn azadi() -> Command {
    Command::cargo_bin("azadi").unwrap()
}

/// Common azadi-noweb delimiters used across tests.
fn delim_args() -> Vec<&'static str> {
    vec![
        "--open-delim", "<<",
        "--close-delim", ">>",
        "--chunk-end", "@",
        "--comment-markers", "#",
    ]
}

// ── Directory mode ────────────────────────────────────────────────────────────

/// --directory discovers driver files and processes them, while %include'd
/// fragments are skipped as standalone inputs.
#[test]
fn test_directory_mode_processes_drivers() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().canonicalize().unwrap();

    // fragment.adoc: just defines a macro — no @file chunk of its own.
    // It would produce no output if run standalone, but here we verify it is
    // correctly identified as a fragment (referenced via %include) and not
    // run as an independent driver.
    write(&root, "src/fragment.adoc", "%def(greeting, Hello from fragment)\n");

    // driver.adoc: includes the fragment and writes one @file output.
    write(
        &root,
        "src/driver.adoc",
        "%include(src/fragment.adoc)\n\
         # <<@file out.txt>>=\n\
         %greeting()\n\
         # @\n",
    );

    let gen_dir = root.join("gen");

    azadi()
        .arg("--directory").arg(root.join("src"))
        .arg("--include").arg(&root)
        .arg("--gen").arg(&gen_dir)
        .arg("--work-dir").arg(root.join("work"))
        .args(delim_args())
        .assert()
        .success();

    let output = fs::read_to_string(gen_dir.join("out.txt")).unwrap();
    assert!(
        output.contains("Hello from fragment"),
        "driver output should contain the macro expansion from the included fragment"
    );
}

/// --directory with multiple independent drivers (no shared includes).
#[test]
fn test_directory_mode_multiple_drivers() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().canonicalize().unwrap();

    write(
        &root,
        "src/a.adoc",
        "# <<@file a.txt>>=\nfrom-a\n# @\n",
    );
    write(
        &root,
        "src/b.adoc",
        "# <<@file b.txt>>=\nfrom-b\n# @\n",
    );

    let gen_dir = root.join("gen");

    azadi()
        .arg("--directory").arg(root.join("src"))
        .arg("--include").arg(&root)
        .arg("--gen").arg(&gen_dir)
        .arg("--work-dir").arg(root.join("work"))
        .args(delim_args())
        .assert()
        .success();

    assert_eq!(fs::read_to_string(gen_dir.join("a.txt")).unwrap().trim(), "from-a");
    assert_eq!(fs::read_to_string(gen_dir.join("b.txt")).unwrap().trim(), "from-b");
}

// ── Depfile and stamp ─────────────────────────────────────────────────────────

/// --stamp creates an empty file on success.
#[test]
fn test_stamp_is_written_on_success() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().canonicalize().unwrap();

    write(&root, "src/driver.adoc", "# <<@file out.txt>>=\nok\n# @\n");

    let stamp = root.join("build.stamp");

    azadi()
        .arg("--directory").arg(root.join("src"))
        .arg("--include").arg(&root)
        .arg("--gen").arg(root.join("gen"))
        .arg("--work-dir").arg(root.join("work"))
        .arg("--stamp").arg(&stamp)
        .args(delim_args())
        .assert()
        .success();

    assert!(stamp.exists(), "--stamp file should be created on success");
}

/// --depfile lists all discovered .adoc files as dependencies and names the
/// stamp as the Makefile target.
#[test]
fn test_depfile_lists_adoc_files() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path().canonicalize().unwrap();

    write(&root, "src/fragment.adoc", "%def(msg, hi)\n");
    write(
        &root,
        "src/driver.adoc",
        "%include(src/fragment.adoc)\n# <<@file out.txt>>=\n%msg()\n# @\n",
    );

    let stamp = root.join("build.stamp");
    let depfile = root.join("build.d");

    azadi()
        .arg("--directory").arg(root.join("src"))
        .arg("--include").arg(&root)
        .arg("--gen").arg(root.join("gen"))
        .arg("--work-dir").arg(root.join("work"))
        .arg("--stamp").arg(&stamp)
        .arg("--depfile").arg(&depfile)
        .args(delim_args())
        .assert()
        .success();

    let dep_content = fs::read_to_string(&depfile).unwrap();

    // The depfile target should be the stamp path.
    assert!(
        dep_content.contains("build.stamp"),
        "depfile should name the stamp as target; got:\n{dep_content}"
    );
    // Both .adoc files should appear as dependencies.
    assert!(
        dep_content.contains("driver.adoc"),
        "depfile should list driver.adoc; got:\n{dep_content}"
    );
    assert!(
        dep_content.contains("fragment.adoc"),
        "depfile should list fragment.adoc; got:\n{dep_content}"
    );
}
