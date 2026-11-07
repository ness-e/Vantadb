// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! TEC-15 (Q4-C): GDPR Art. 17 Right to Erasure cascade integration tests.
//!
//! Verifies:
//! 1. L1 episodic records with PII are inserted.
//! 2. Consolidated L2 scenes and L3 axioms/personas derived from L1 are created
//!    with deterministic source episode provenance (`source_episode_ids`).
//! 3. `forget(episode_id, mode)` cascades from L1 -> L2 -> L3 for both
//!    `ForgetMode::Certified` and `ForgetMode::Hard` (as well as `Soft`).
//! 4. Zero PII leakage across subsequent searches and point-reads.
//! 5. Verifiable receipt provides complete traceability of purged cascaded nodes
//!    and attestation certificates.

use vantadb::config::Config;
use vantadb::sdk::{Embedded, ForgetMode, MemoryInput, MemoryMetadata, MemorySearchRequest};
use vantadb::storage::BackendKind;

use vanta_memory::core::abstractions::{MemoryRecord, MemoryType};
use vanta_memory::core::persona::persona_generator::{get_persona, persona_namespace, PERSONA_KEY};
use vanta_memory::core::record::cascade_forget::{forget, put_l3_axiom};
use vanta_memory::core::record::l1_reader::{l1_namespace, read_record, read_session_records};
use vanta_memory::core::record::put_record;
use vanta_memory::core::scene::scene_index::{
    get_scene, list_scenes, scene_namespace, upsert_scene_with_sources,
};

fn open_db() -> Embedded {
    let config = Config {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..Config::default()
    };
    Embedded::open_with_config(config).expect("open in-memory db")
}

fn create_l1_episode(
    db: &Embedded,
    session: &str,
    id: &str,
    content: &str,
    scene_name: &str,
) -> MemoryRecord {
    let now = chrono::Utc::now().to_rfc3339();
    let record = MemoryRecord {
        id: id.to_string(),
        content: content.to_string(),
        memory_type: MemoryType::Episodic,
        priority: 90,
        scene_name: scene_name.to_string(),
        source_message_ids: vec![format!("msg_{id}")],
        metadata: serde_json::json!({
            "is_episodic": true,
            "confidential": true,
        }),
        timestamps: vec![now.clone()],
        created_at: now.clone(),
        updated_at: now,
        version: 1,
        session_key: session.to_string(),
        session_id: session.to_string(),
        task_id: None,
        team_id: None,
        user_id: None,
        agent_id: None,
        vector: None,
        heat: 1,
        superseded_by: None,
    };
    put_record(db, &l1_namespace(session), &record, None).expect("put l1 record");
    record
}

#[test]
fn test_tec_15_gdpr_cascade_forget_l1_to_l2_and_l3() {
    let db = open_db();
    let session = "gdpr-session-certified";

    // ──────────────────────────────────────────────────────────────────────────
    // Step 1: Insert L1 episodes with sensitive personal data (PII)
    // ──────────────────────────────────────────────────────────────────────────
    let pii_content =
        "Patient SSN is 999-00-1234, diagnosed with acute pancreatitis, address 104 Riverside Dr.";
    let ep_pii = create_l1_episode(
        &db,
        session,
        "ep_pii_medical_001",
        pii_content,
        "medical_assessment",
    );

    let safe_content = "User prefers communication via secure encrypted email.";
    let _ep_safe = create_l1_episode(&db, session, "ep_general_002", safe_content, "preferences");

    // Verify L1 records exist
    let initial_l1 = read_session_records(&db, session).expect("read session records");
    assert_eq!(initial_l1.len(), 2);
    assert!(read_record(&db, session, "ep_pii_medical_001")
        .expect("read")
        .is_some());
    assert!(read_record(&db, session, "ep_general_002")
        .expect("read")
        .is_some());

    // ──────────────────────────────────────────────────────────────────────────
    // Step 2: Consolidate/generate an L2 scene and an L3 axiom derived from ep_pii
    // ──────────────────────────────────────────────────────────────────────────
    // L2 Scene derived from ep_pii_medical_001
    let scene_summary = "Clinical intake: patient SSN 999-00-1234 and acute pancreatitis notes";
    let scene_content = "Consolidated clinical scene. Patient SSN 999-00-1234 with acute pancreatitis at 104 Riverside Dr.";
    let l2_scene = upsert_scene_with_sources(
        &db,
        session,
        "medical_assessment",
        scene_summary,
        scene_content,
        vec![ep_pii.id.clone()],
    )
    .expect("create l2 scene");
    assert_eq!(l2_scene.source_episode_ids, vec!["ep_pii_medical_001"]);

    // L3 Semantic Axiom derived from ep_pii_medical_001
    let axiom_content = "High clinical risk: critical health profile for patient SSN 999-00-1234.";
    let l3_axiom = put_l3_axiom(
        &db,
        session,
        "axiom_medical_risk",
        axiom_content,
        vec![ep_pii.id.clone()],
    )
    .expect("create l3 axiom");
    assert_eq!(l3_axiom.source_message_ids, vec!["ep_pii_medical_001"]);

    // L3 Persona derived from the medical scene
    let persona_ns = persona_namespace(session);
    let persona_key = PERSONA_KEY.to_string();
    let persona_content =
        "Patient Persona Profile: Verified medical history and SSN 999-00-1234 on record.";
    let mut persona_meta = MemoryMetadata::new();
    persona_meta.insert(
        "source_episode_ids".into(),
        vantadb::sdk::Value::ListString(vec![ep_pii.id.clone()]),
    );
    persona_meta.insert(
        "derived_from".into(),
        vantadb::sdk::Value::ListString(vec![ep_pii.id.clone()]),
    );
    let persona_payload = serde_json::to_string(&vanta_memory::core::persona::PersonaRecord {
        content: persona_content.to_string(),
        mode: vanta_memory::core::abstractions::PersonaMode::First,
        generated_at_ms: 1000,
        generated_at: chrono::Utc::now().to_rfc3339(),
        source_episode_ids: vec![ep_pii.id.clone()],
        source_scene_names: vec!["medical_assessment".to_string()],
    })
    .expect("serialize persona");
    db.put(MemoryInput {
        namespace: persona_ns.clone(),
        key: persona_key,
        payload: persona_payload,
        metadata: persona_meta,
        ..Default::default()
    })
    .expect("store persona");

    // Pre-erasure assertions: confirm L1, L2, L3 are all searchable and readable
    assert!(get_scene(&db, session, "medical_assessment")
        .expect("get scene")
        .is_some());
    assert!(get_persona(&db, session).expect("get persona").is_some());
    assert!(read_record(&db, session, "axiom_medical_risk")
        .expect("get axiom")
        .is_some());

    let pre_search = db
        .search(MemorySearchRequest {
            namespace: l1_namespace(session),
            text_query: Some("pancreatitis".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("search");
    assert!(
        !pre_search.is_empty(),
        "PII was searchable in L1 before forget"
    );

    // ──────────────────────────────────────────────────────────────────────────
    // Step 3: Execute `forget` with `ForgetMode::Certified` on L1 episode
    // ──────────────────────────────────────────────────────────────────────────
    let receipt = forget(&db, session, "ep_pii_medical_001", ForgetMode::Certified)
        .expect("certified cascade forget");

    // Verification of Receipt properties
    assert_eq!(receipt.status, "erased");
    assert_eq!(receipt.mode, ForgetMode::Certified);
    assert_eq!(receipt.session_id, session);
    assert!(receipt
        .primary_keys
        .contains(&"ep_pii_medical_001".to_string()));

    // Traceability: cascaded_ids MUST contain L2 scene and L3 axioms/personas
    assert!(
        receipt
            .cascaded_ids
            .contains(&"medical_assessment".to_string()),
        "receipt must trace cascaded L2 scene 'medical_assessment', got: {:?}",
        receipt.cascaded_ids
    );
    assert!(
        receipt
            .cascaded_ids
            .contains(&"axiom_medical_risk".to_string()),
        "receipt must trace cascaded L3 axiom 'axiom_medical_risk', got: {:?}",
        receipt.cascaded_ids
    );
    assert!(
        receipt.cascaded_ids.contains(&PERSONA_KEY.to_string()),
        "receipt must trace cascaded L3 persona, got: {:?}",
        receipt.cascaded_ids
    );
    assert!(
        !receipt.certificates.is_empty(),
        "certified forget must generate attestation certificates"
    );

    // ──────────────────────────────────────────────────────────────────────────
    // Step 4: Verify L1, L2, L3 are completely eliminated and zero PII residues remain
    // ──────────────────────────────────────────────────────────────────────────
    // 1. Point reads must return None
    assert!(
        read_record(&db, session, "ep_pii_medical_001")
            .expect("read")
            .is_none(),
        "L1 episode must be deleted"
    );
    assert!(
        get_scene(&db, session, "medical_assessment")
            .expect("get scene")
            .is_none(),
        "L2 scene must be eliminated in cascade"
    );
    assert!(
        list_scenes(&db, session).expect("list scenes").is_empty(),
        "Scene index must not contain cascaded scene"
    );
    assert!(
        get_persona(&db, session).expect("get persona").is_none(),
        "L3 persona must be eliminated in cascade"
    );
    assert!(
        read_record(&db, session, "axiom_medical_risk")
            .expect("read axiom")
            .is_none(),
        "L3 axiom must be eliminated in cascade"
    );

    // 2. Unrelated/safe episode MUST survive untouched
    let safe_read = read_record(&db, session, "ep_general_002").expect("read safe");
    assert!(safe_read.is_some(), "Safe non-derived episode must survive");
    assert_eq!(safe_read.unwrap().content, safe_content);

    // 3. Search queries for sensitive PII must yield ZERO hits
    let pii_search_l1 = db
        .search(MemorySearchRequest {
            namespace: l1_namespace(session),
            text_query: Some("pancreatitis".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("search l1");
    assert_eq!(pii_search_l1.len(), 0, "No PII residues in L1 search");

    let ssn_search_l1 = db
        .search(MemorySearchRequest {
            namespace: l1_namespace(session),
            text_query: Some("999-00-1234".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("search l1 ssn");
    assert!(
        ssn_search_l1
            .iter()
            .all(|h| h.record.key != "ep_pii_medical_001"),
        "Erased L1 episode ep_pii_medical_001 must not appear in search results"
    );
    assert!(
        ssn_search_l1
            .iter()
            .all(|h| !h.record.payload.contains("999-00-1234")),
        "No search hit payload should contain SSN"
    );

    let address_search_l1 = db
        .search(MemorySearchRequest {
            namespace: l1_namespace(session),
            text_query: Some("Riverside".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("search l1 address");
    assert_eq!(
        address_search_l1.len(),
        0,
        "No address PII residues in L1 search"
    );

    let pii_search_scene = db
        .search(MemorySearchRequest {
            namespace: scene_namespace(session),
            text_query: Some("999-00-1234".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("search scene");
    assert_eq!(
        pii_search_scene.len(),
        0,
        "No PII residues in L2 scene search"
    );

    // ──────────────────────────────────────────────────────────────────────────
    // Step 5: Test Hard mode forget on a fresh session
    // ──────────────────────────────────────────────────────────────────────────
    test_hard_mode_cascade(&db);

    // ──────────────────────────────────────────────────────────────────────────
    // Step 6: Test Soft mode forget
    // ──────────────────────────────────────────────────────────────────────────
    test_soft_mode_cascade(&db);
}

fn test_hard_mode_cascade(db: &Embedded) {
    let session = "gdpr-session-hard";

    let ep = create_l1_episode(
        db,
        session,
        "ep_hard_01",
        "User passport number ABC123456",
        "passport_review",
    );
    upsert_scene_with_sources(
        db,
        session,
        "passport_review",
        "Passport verification notes ABC123456",
        "Scene content documenting passport ABC123456",
        vec![ep.id.clone()],
    )
    .expect("upsert scene");

    put_l3_axiom(
        db,
        session,
        "axiom_passport_validity",
        "User nationality verified via passport ABC123456",
        vec![ep.id.clone()],
    )
    .expect("put axiom");

    // Execute Hard forget
    let receipt = forget(db, session, "ep_hard_01", ForgetMode::Hard).expect("hard forget");
    assert_eq!(receipt.status, "erased");
    assert_eq!(receipt.mode, ForgetMode::Hard);
    assert!(receipt
        .cascaded_ids
        .contains(&"passport_review".to_string()));
    assert!(receipt
        .cascaded_ids
        .contains(&"axiom_passport_validity".to_string()));

    // Verify hard deletion across layers
    assert!(read_record(db, session, "ep_hard_01")
        .expect("read")
        .is_none());
    assert!(get_scene(db, session, "passport_review")
        .expect("get scene")
        .is_none());
    assert!(read_record(db, session, "axiom_passport_validity")
        .expect("read axiom")
        .is_none());

    let search_hard = db
        .search(MemorySearchRequest {
            namespace: l1_namespace(session),
            text_query: Some("ABC123456".into()),
            top_k: 10,
            ..Default::default()
        })
        .expect("search");
    assert_eq!(
        search_hard.len(),
        0,
        "No passport residues after Hard forget"
    );
}

fn test_soft_mode_cascade(db: &Embedded) {
    let session = "gdpr-session-soft";

    let ep = create_l1_episode(
        db,
        session,
        "ep_soft_01",
        "User credit score is 780",
        "credit_check",
    );
    upsert_scene_with_sources(
        db,
        session,
        "credit_check",
        "Credit rating evaluation",
        "Credit rating 780 reviewed",
        vec![ep.id.clone()],
    )
    .expect("upsert scene");

    let receipt = forget(db, session, "ep_soft_01", ForgetMode::Soft).expect("soft forget");
    assert_eq!(receipt.status, "soft_invalidated");
    assert_eq!(receipt.mode, ForgetMode::Soft);
    assert!(receipt.cascaded_ids.contains(&"credit_check".to_string()));

    // Soft delete scene marks deleted: true and content [DELETED]
    let scene = get_scene(db, session, "credit_check")
        .expect("get scene")
        .expect("soft deleted scene exists");
    assert!(scene.is_deleted(), "Scene must be marked deleted");
    assert_eq!(scene.content, "[DELETED]");

    // Scene index excludes soft deleted scenes
    assert!(list_scenes(db, session).expect("list scenes").is_empty());
}
