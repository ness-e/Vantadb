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

fn extract_cargo_version(cargo_toml: &str, workspace: &str) -> String {
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed == "version.workspace = true" {
            return extract_workspace_version(workspace);
        }
        if trimmed == "[package]" {
            continue;
        }
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

fn extract_workspace_version(workspace: &str) -> String {
    let mut in_workspace_package = false;
    for line in workspace.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace.package]" {
            in_workspace_package = true;
            continue;
        }
        if in_workspace_package {
            if trimmed.starts_with('[') {
                break;
            }
            if trimmed.starts_with("version") {
                if let Some((_, rhs)) = trimmed.split_once('=') {
                    let v = rhs.trim().trim_matches('"');
                    if !v.is_empty() {
                        return v.to_string();
                    }
                }
            }
        }
    }
    panic!("failed to extract version from [workspace.package]");
}

#[test]
fn public_surfaces_report_same_version() {
    let root = repo_root();

    let cargo_toml = read(root.join("Cargo.toml"));
    let version = extract_cargo_version(&cargo_toml, &cargo_toml);

    let pyproject = read(root.join("vantadb-python").join("pyproject.toml"));
    assert!(
        pyproject.contains(&format!("version = \"{}\"", version)),
        "pyproject.toml must match Cargo.toml version {}",
        version
    );

    let python_cargo = read(root.join("vantadb-python").join("Cargo.toml"));
    assert_eq!(
        extract_cargo_version(&python_cargo, &cargo_toml),
        version,
        "vantadb-python/Cargo.toml must match root Cargo.toml version {}",
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

    // MCP was extracted from vantadb-server/src/mcp.rs into its own crate during CUARENTENA-01.
    let mcp = read(root.join("vantadb-mcp").join("src").join("lib.rs"));
    assert!(
        mcp.contains("metadata::reported_version()"),
        "MCP serverInfo.version must use vantadb::metadata::reported_version()"
    );

    let mcp_cargo = read(root.join("vantadb-mcp").join("Cargo.toml"));
    assert_eq!(
        extract_cargo_version(&mcp_cargo, &cargo_toml),
        version,
        "vantadb-mcp/Cargo.toml must match root Cargo.toml version {}",
        version
    );

    let server_cargo = read(root.join("vantadb-server").join("Cargo.toml"));
    assert_eq!(
        extract_cargo_version(&server_cargo, &cargo_toml),
        version,
        "vantadb-server/Cargo.toml must match root Cargo.toml version {}",
        version
    );

    // Optional: Verify langchain-vantadb if it exists in the repository
    let langchain_path = root
        .join("packages")
        .join("langchain-vantadb")
        .join("pyproject.toml");
    if langchain_path.exists() {
        let langchain_pyproject = read(&langchain_path);
        assert!(
            langchain_pyproject.contains(&format!("version = \"{}\"", version)),
            "packages/langchain-vantadb/pyproject.toml must match Cargo.toml version {}",
            version
        );
        assert!(
            langchain_pyproject.contains(&format!("\"vantadb-py>={}\"", version)),
            "packages/langchain-vantadb/pyproject.toml must require vantadb-py >= {}",
            version
        );
    }

    // Optional: Verify llamaindex-vantadb if it exists in the repository
    let llamaindex_path = root
        .join("packages")
        .join("llamaindex-vantadb")
        .join("pyproject.toml");
    if llamaindex_path.exists() {
        let llamaindex_pyproject = read(&llamaindex_path);
        assert!(
            llamaindex_pyproject.contains(&format!("version = \"{}\"", version)),
            "packages/llamaindex-vantadb/pyproject.toml must match Cargo.toml version {}",
            version
        );
        assert!(
            llamaindex_pyproject.contains(&format!("\"vantadb-py>={}\"", version)),
            "packages/llamaindex-vantadb/pyproject.toml must require vantadb-py >= {}",
            version
        );
    }
}
