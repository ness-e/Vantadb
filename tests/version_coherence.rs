//! Version drift guardrails.
//!
//! This suite ensures that user-facing surfaces do not report conflicting versions.

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: impl Into<PathBuf>) -> String {
    fs::read_to_string(path.into()).expect("read file")
}

fn extract_cargo_version(cargo_toml: &str) -> String {
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version") {
            if let Some((_, rhs)) = trimmed.split_once('=') {
                let v = rhs.trim().trim_matches('"');
                if !v.is_empty() {
                    return v.to_string();
                }
            }
        }
    }
    panic!("failed to extract version from Cargo.toml");
}

#[test]
fn public_surfaces_report_same_version() {
    let root = repo_root();

    let cargo_toml = read(root.join("Cargo.toml"));
    let version = extract_cargo_version(&cargo_toml);

    let pyproject = read(root.join("vantadb-python").join("pyproject.toml"));
    assert!(
        pyproject.contains(&format!("version = \"{}\"", version)),
        "pyproject.toml must match Cargo.toml version {}",
        version
    );

    let python_binding = read(root.join("vantadb-python").join("src").join("lib.rs"));
    assert!(
        python_binding.contains("metadata::reported_version"),
        "Python __version__ must use vantadb::metadata::reported_version() so it matches MCP/banner and optional VANTADB_REPORTED_VERSION"
    );

    let console = read(root.join("src").join("console.rs"));
    assert!(
        console.contains("metadata::version_label()"),
        "Console banner must derive version via vantadb::metadata::version_label()"
    );

    let mcp = read(root.join("src").join("api").join("mcp.rs"));
    assert!(
        mcp.contains("metadata::reported_version()"),
        "MCP serverInfo.version must use vantadb::metadata::reported_version()"
    );
}
