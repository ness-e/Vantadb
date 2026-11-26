// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! E2E for VER-06: the git-friendly Markdown flow.
//!
//! `vanta-cli export --format md` → manual edit of one `.md` →
//! `vanta-seed import-md` (real binary) → `vanta-cli rebuild-index` →
//! `get`/`search` return the edited value.
//!
//! The `vanta-cli` steps run through `vantadb::cli_handlers` (the exact
//! dispatch of the binary — `src/bin/vanta-cli.rs:64/:82`); the import spawns
//! the real `vanta-seed` binary (`CARGO_BIN_EXE_vanta-seed`) against the same
//! Fjall database, proving cross-process persistence of the flow. A second
//! test asserts the export is byte-identical across runs (git-friendly: no
//! volatile timestamps, sorted manifest).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use vantadb::config::Config;
use vantadb::sdk::{Embedded, MemoryInput, MemorySearchRequest};
use vantadb::storage::BackendKind;

fn open_fjall(path: &str) -> Embedded {
    Embedded::open_with_config(Config {
        storage_path: path.to_string(),
        backend_kind: BackendKind::Fjall,
        read_only: false,
        ..Config::default()
    })
    .expect("open fjall db")
}

#[test]
fn md_git_flow_export_edit_import_rebuild_search() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("db").to_string_lossy().to_string();
    let out_dir = tmp.path().join("export").to_string_lossy().to_string();

    // 1) Seed a Fjall DB: a plain record plus a superseded pair (wikilinks).
    {
        let db = open_fjall(&db_path);
        db.put(MemoryInput::new("agent/team", "intro", "original payload"))
            .expect("put intro");
        db.put(MemoryInput::new("agent/team", "old", "superseded body"))
            .expect("put old");
        db.put(MemoryInput::new("agent/team", "new", "replacement body"))
            .expect("put new");
        db.supersede("agent/team", "old", "new").expect("supersede");
        db.close().expect("close seeding db");
    }

    // 2) Export via the CLI handler (same code as `vanta-cli export --format md`).
    vantadb::cli_handlers::cmd_export_md(&db_path, None, &out_dir, true).expect("export md");

    // Layout: sanitize_component("agent/team") == "agent_team".
    let intro_md = Path::new(&out_dir).join("agent_team").join("intro.md");
    let old_md = Path::new(&out_dir).join("agent_team").join("old.md");
    let raw = fs::read_to_string(&intro_md).expect("read intro.md");
    assert!(
        raw.contains("\"schema_version\":2"),
        "export must emit v2 frontmatter: {raw}"
    );
    assert!(raw.contains("\"valid_at_ms\""), "v2 validity field present");
    assert!(
        raw.contains("\"confidence_class\":\"Asserted\""),
        "v2 confidence field present"
    );
    let raw_old = fs::read_to_string(&old_md).expect("read old.md");
    assert!(
        raw_old.contains("- [[agent/team/new]]"),
        "superseded_by renders as a wikilink: {raw_old}"
    );

    // 3) Manual edit: change the payload text only.
    let edited = raw.replace("original payload", "edited payload");
    assert_ne!(edited, raw, "the edit must change the file");
    fs::write(&intro_md, edited).expect("write edited intro.md");

    // 4) Re-import with the real `vanta-seed import-md` binary, same DB.
    let out = Command::new(env!("CARGO_BIN_EXE_vanta-seed"))
        .args(["import-md", &out_dir, "--db", &db_path])
        .output()
        .expect("spawn vanta-seed");
    assert!(
        out.status.success(),
        "vanta-seed import-md failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("updated=1"),
        "the edited record must update: {stdout}"
    );
    assert!(
        stdout.contains("unchanged=2"),
        "untouched records must stay unchanged: {stdout}"
    );
    assert!(
        stdout.contains("links_unresolved=0"),
        "the wikilink target resolves: {stdout}"
    );

    // 5) Rebuild indexes via the CLI handler (`vanta-cli rebuild-index`).
    vantadb::cli_handlers::cmd_rebuild_index(&db_path, false, true).expect("rebuild-index");

    // 6) get + search return the edited value.
    let db = open_fjall(&db_path);
    let got = db
        .get("agent/team", "intro")
        .expect("get intro")
        .expect("intro exists");
    assert_eq!(
        got.payload, "edited payload",
        "get returns the edited value"
    );

    let hits = db
        .search(MemorySearchRequest {
            namespace: "agent/team".into(),
            text_query: Some("edited".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("search");
    assert!(
        hits.iter()
            .any(|h| h.record.key == "intro" && h.record.payload == "edited payload"),
        "search after rebuild returns the edited value: {hits:?}"
    );
    db.close().expect("close");
}

#[test]
fn md_export_is_byte_identical_across_runs() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("db").to_string_lossy().to_string();
    let out_a = tmp.path().join("export-a").to_string_lossy().to_string();
    let out_b = tmp.path().join("export-b").to_string_lossy().to_string();

    {
        let db = open_fjall(&db_path);
        db.put(MemoryInput::new("ns/one", "alpha", "first body"))
            .expect("put alpha");
        db.put(MemoryInput::new("ns/one", "beta", "second body"))
            .expect("put beta");
        db.put(MemoryInput::new("ns/two", "gamma", "third body"))
            .expect("put gamma");
        db.close().expect("close");
    }

    vantadb::cli_handlers::cmd_export_md(&db_path, None, &out_a, true).expect("export a");
    // No data changes between runs: the output must be byte-identical (no
    // wall-clock timestamp, manifest sorted by file path).
    vantadb::cli_handlers::cmd_export_md(&db_path, None, &out_b, true).expect("export b");

    let files_a = collect_relative_files(Path::new(&out_a));
    let files_b = collect_relative_files(Path::new(&out_b));
    assert_eq!(files_a, files_b, "same file set in both exports");
    assert!(!files_a.is_empty(), "export produced files");
    for rel in &files_a {
        let bytes_a = fs::read(Path::new(&out_a).join(rel)).expect("read a");
        let bytes_b = fs::read(Path::new(&out_b).join(rel)).expect("read b");
        assert_eq!(
            bytes_a,
            bytes_b,
            "byte-identical export for {}",
            rel.display()
        );
    }
}

fn collect_relative_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_inner(root, root, &mut out);
    out.sort();
    out
}

fn collect_inner(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).expect("read_dir");
    for entry in entries {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.is_dir() {
            collect_inner(root, &path, out);
        } else {
            out.push(
                path.strip_prefix(root)
                    .expect("relative path")
                    .to_path_buf(),
            );
        }
    }
}
