// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! D19 tests for the seed/import module (MEM-39): file import, content-hash
//! idempotency (replay never duplicates), typed validation errors, and
//! namespace sanitization.

use std::io::Write;

use tempfile::NamedTempFile;
use vanta_memory::core::persona::{get_persona, persona_namespace, PERSONA_KEY};
use vanta_memory::core::skill::conversation_add::sink::SkillCoreSink;
use vanta_memory::seed::{import_seed_file, import_seed_str, SeedError};
use vantadb::config::VantaConfig;
use vantadb::sdk::VantaEmbedded;
use vantadb::storage::BackendKind;

const SEED_JSON: &str = r##"{
  "scope": "my agent/1",
  "skills": [
    { "name": "deploy runbook", "description": "how to deploy", "content": "step 1: build" },
    { "name": "review", "description": "", "content": "checklist body" }
  ],
  "persona": { "session_key": "user one", "content": "# User Narrative Profile\nlikes rust" }
}"##;

fn open_db() -> VantaEmbedded {
    VantaEmbedded::open_with_config(VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..VantaConfig::default()
    })
    .expect("open in-memory db")
}

fn temp_seed_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("temp file");
    file.write_all(content.as_bytes()).expect("write seed");
    file
}

#[test]
fn import_from_temp_file_creates_skills_and_persona() {
    let db = open_db();
    let path = temp_seed_file(SEED_JSON).into_temp_path();

    let counts = import_seed_file(&db, &path).expect("import");
    assert_eq!(counts.created, 3, "2 skills + 1 persona");
    assert_eq!(counts.updated, 0);
    assert_eq!(counts.unchanged, 0);

    // Skills readable through the MEM-06 sink reader (payload parity).
    let sink = SkillCoreSink::new(&db);
    let skill = sink
        .read_skill("my agent/1", "deploy runbook")
        .expect("read");
    let skill = skill.expect("skill exists");
    assert_eq!(skill.content, "step 1: build");
    assert_eq!(skill.description, "how to deploy");

    // Persona readable through the L3 getter.
    let persona = get_persona(&db, "user one").expect("read").expect("exists");
    assert!(persona.content.contains("likes rust"));
}

#[test]
fn replay_is_fully_idempotent_no_duplicates() {
    let db = open_db();
    let path = temp_seed_file(SEED_JSON).into_temp_path();

    let first = import_seed_file(&db, &path).expect("first import");
    assert_eq!(first.created, 3);

    let replay = import_seed_file(&db, &path).expect("replay");
    assert_eq!(
        replay,
        vanta_memory::seed::SeedCounts {
            created: 0,
            updated: 0,
            unchanged: 3
        },
        "replay must not duplicate or rewrite"
    );

    // Changed content → update, not duplicate (same key).
    let changed = SEED_JSON.replace("step 1: build", "step 1: build v2");
    let path2 = temp_seed_file(&changed).into_temp_path();
    let second = import_seed_file(&db, &path2).expect("second import");
    assert_eq!(second.updated, 1, "changed skill updated in place");
    assert_eq!(second.unchanged, 2);
}

#[test]
fn invalid_inputs_yield_typed_errors() {
    let db = open_db();

    // Missing file → Io.
    let missing = std::path::Path::new("Z:/definitely/not/here.json");
    assert!(matches!(
        import_seed_file(&db, missing),
        Err(SeedError::Io(_))
    ));

    // Malformed JSON → Json.
    assert!(matches!(
        import_seed_str(&db, "{not json"),
        Err(SeedError::Json(_))
    ));

    // Structurally invalid → Validation.
    assert!(matches!(
        import_seed_str(&db, r#"{"skills":[{"name":"","content":"x"}]}"#),
        Err(SeedError::Validation(_))
    ));
    assert!(matches!(
        import_seed_str(&db, r#"{"persona":{"session_key":"u","content":"  "}}"#),
        Err(SeedError::Validation(_))
    ));
    assert!(matches!(
        import_seed_str(&db, "{}"),
        Err(SeedError::Validation(_))
    ));
}

#[test]
fn weird_scope_and_session_keys_are_sanitized() {
    let db = open_db();
    let seed = import_seed_str(&db, SEED_JSON).expect("import");

    assert_eq!(seed.created, 3);

    // Persona lands under the sanitized namespace with the canonical key.
    let ns = persona_namespace("user one");
    assert_eq!(ns, "persona/user_one", "space replaced per safe charset");
    let record = db.get(&ns, PERSONA_KEY).expect("get").expect("exists");

    // Raw payload is valid JSON with the expected shape.
    let value: serde_json::Value = serde_json::from_str(&record.payload).expect("json");
    assert_eq!(value["mode"], "first");
}
