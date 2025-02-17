use std::env;
use std::process::Command;

fn main() {
    // Only configure Python linking when pyo3 feature is enabled
    if env::var_os("CARGO_FEATURE_PYO3").is_none() {
        return;
    }

    // Get Python version from python3-config
    let python_version = Command::new("python3-config")
        .arg("--embed")
        .arg("--libs")
        .output()
        .expect("Failed to execute python3-config")
        .stdout;
    let python_libs = String::from_utf8(python_version).unwrap();

    // Parse library flags
    for flag in python_libs.split_whitespace() {
        if flag.starts_with("-L") {
            println!("cargo:rustc-link-search=native={}", &flag[2..]);
        } else if flag.starts_with("-l") {
            println!("cargo:rustc-link-lib=dylib={}", &flag[2..]);
        }
    }

    // Tell cargo to invalidate the built crate whenever the build script changes
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PYENV_VERSION");
}
