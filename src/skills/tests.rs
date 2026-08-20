//! Dedicated multi-version skill tests (D19, plan vanta-memory).
//!
//! Pattern AAA: arrange → act → assert. Uses an in-memory `StorageEngine`
//! (same setup as `src/entity/tests.rs`).

use super::{SkillStore, KEEP_RECENT};
use crate::config::VantaConfig;
use crate::error::VantaError;
use crate::sdk::types::{SkillCreateInput, SkillListOptions, SkillPatchInput, SkillUpdateInput};
use crate::storage::{BackendKind, StorageEngine};

fn in_memory_engine() -> StorageEngine {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..VantaConfig::default()
    };
    StorageEngine::open_with_config(":memory:", Some(config)).expect("open in-memory engine")
}

fn create_input(name: &str, content: &str) -> SkillCreateInput {
    SkillCreateInput {
        name: name.into(),
        description: format!("desc {name}"),
        content: content.into(),
        owner_agent: "agent-1".into(),
        metadata: Default::default(),
        ttl_secs: None,
    }
}

// ── CRUD roundtrip ──

#[test]
fn create_get_update_patch_delete_roundtrip() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    let created = store
        .create(create_input("greeter", "return 'hello'"))
        .expect("create");
    assert!(!created.idempotent);
    assert_eq!(created.record.version, 1);
    assert!(created.record.is_head);
    assert!(created.record.skill_id.starts_with("skl-"));

    let head = store
        .get_head(&created.record.skill_id)
        .expect("get head")
        .expect("head exists");
    assert_eq!(head, created.record);

    let updated = store
        .update(
            &created.record.skill_id,
            1,
            SkillUpdateInput {
                description: "greets the user".into(),
                content: "return 'hello world'".into(),
                metadata: None,
            },
        )
        .expect("update");
    assert!(!updated.idempotent);
    assert_eq!(updated.record.version, 2);
    assert!(updated.record.is_head);
    assert_eq!(updated.record.content, "return 'hello world'");

    let v1 = store
        .get_version(&created.record.skill_id, 1)
        .expect("get v1")
        .expect("v1 exists");
    assert!(!v1.is_head, "old head must be demoted");

    let patched = store
        .patch(
            &created.record.skill_id,
            2,
            SkillPatchInput {
                description: Some("polite greeter".into()),
                content: None,
                metadata: None,
            },
        )
        .expect("patch");
    assert!(!patched.idempotent);
    assert_eq!(patched.record.version, 3);
    assert_eq!(patched.record.description, "polite greeter");
    assert_eq!(patched.record.content, "return 'hello world'");

    let deleted = store.delete(&created.record.skill_id, 3).expect("delete");
    assert!(deleted);
    assert!(
        store
            .get_head(&created.record.skill_id)
            .expect("get head")
            .is_none(),
        "head must be gone after delete"
    );
}

// ── Versioning: name/owner immutable, DESC list ──

#[test]
fn versions_list_newest_first_and_identity_immutable() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    let created = store
        .create(create_input("immutable", "v1"))
        .expect("create");
    let skill_id = created.record.skill_id.clone();

    for (i, content) in ["v2", "v3", "v4"].iter().enumerate() {
        store
            .update(
                &skill_id,
                (i + 1) as u64,
                SkillUpdateInput {
                    description: format!("desc {content}"),
                    content: (*content).into(),
                    metadata: None,
                },
            )
            .expect("update");
    }

    let page = store
        .list_versions(&skill_id, 10, 0)
        .expect("list versions");
    assert_eq!(page.total, 4);
    let versions: Vec<u64> = page.items.iter().map(|r| r.version).collect();
    assert_eq!(versions, vec![4, 3, 2, 1], "newest first");

    for record in page.items {
        assert_eq!(record.owner_agent, "agent-1");
        assert_eq!(record.name, "immutable");
        assert_eq!(record.skill_id, skill_id);
    }

    let page2 = store.list_versions(&skill_id, 2, 1).expect("paginate");
    assert_eq!(page2.items.len(), 2);
    assert_eq!(page2.items[0].version, 3);
    assert_eq!(page2.items[1].version, 2);
}

// ── Optimistic lock ──

#[test]
fn optimistic_lock_rejects_stale_expected_version() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    let created = store.create(create_input("locked", "v1")).expect("create");
    let skill_id = created.record.skill_id.clone();

    // Advance to v2 so a writer still holding expected_version=1 is stale.
    store
        .update(
            &skill_id,
            1,
            SkillUpdateInput {
                description: "bump".into(),
                content: "v2".into(),
                metadata: None,
            },
        )
        .expect("advance to v2");

    let err = store
        .update(
            &skill_id,
            1,
            SkillUpdateInput {
                description: "bump again".into(),
                content: "v3".into(),
                metadata: None,
            },
        )
        .expect_err("stale expected version must fail");
    match err {
        VantaError::ExecutionConflict { resource, detail } => {
            assert!(resource.contains(&skill_id));
            assert!(detail.contains("expected version 1, head is 2"));
        }
        other => panic!("expected ExecutionConflict, got {other:?}"),
    }

    // Concurrent writer with correct version succeeds.
    store
        .update(
            &skill_id,
            2,
            SkillUpdateInput {
                description: "bump".into(),
                content: "v2".into(),
                metadata: None,
            },
        )
        .expect("correct version update");
}

#[test]
fn delete_rejects_stale_expected_version() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    let created = store
        .create(create_input("locked-del", "v1"))
        .expect("create");
    let skill_id = created.record.skill_id.clone();
    store
        .update(
            &skill_id,
            1,
            SkillUpdateInput {
                description: "bump".into(),
                content: "v2".into(),
                metadata: None,
            },
        )
        .expect("update to v2");

    let err = store.delete(&skill_id, 1).expect_err("stale delete");
    match err {
        VantaError::ExecutionConflict { .. } => {}
        other => panic!("expected ExecutionConflict, got {other:?}"),
    }
    assert!(
        store.get_head(&skill_id).expect("head").is_some(),
        "still alive"
    );
}

// ── TTL keep-recent=3 ──

#[test]
fn ttl_cleanup_keeps_head_and_three_most_recent() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    let mut input = create_input("ttl-skill", "v1");
    input.ttl_secs = Some(100);
    let created = store.create(input).expect("create");
    let skill_id = created.record.skill_id.clone();

    // v1..v6; all expire at created+100. now = created+1000 → everything expired.
    for i in 2..=6u64 {
        store
            .update(
                &skill_id,
                i - 1,
                SkillUpdateInput {
                    description: format!("desc {i}"),
                    content: format!("v{i}"),
                    metadata: None,
                },
            )
            .expect("update");
    }

    let now = created.record.created_at + 1000;
    let deleted = store
        .cleanup_expired_versions(&skill_id, now)
        .expect("cleanup");
    // 6 versions total: head (v6) + 5 non-head; keep 3 most recent non-head
    // (v5, v4, v3) → delete v2, v1.
    assert_eq!(deleted, 2);

    let page = store.list_versions(&skill_id, 100, 0).expect("list");
    assert_eq!(page.total, 4, "v6 + v5 + v4 + v3 remain");
    assert_eq!(page.items[0].version, 6);
    assert!(page.items[0].is_head);
}

#[test]
fn ttl_cleanup_preserves_future_expirations() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    let mut input = create_input("future-ttl", "v1");
    input.ttl_secs = Some(1000);
    let created = store.create(input).expect("create");
    let skill_id = created.record.skill_id.clone();

    let now = created.record.created_at + 500; // before expiry
    let deleted = store
        .cleanup_expired_versions(&skill_id, now)
        .expect("cleanup");
    assert_eq!(deleted, 0, "nothing expired yet");
}

#[test]
fn ttl_expires_at_set_on_versions() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    let mut input = create_input("expires", "v1");
    input.ttl_secs = Some(60);
    let created = store.create(input).expect("create");
    assert_eq!(
        created.record.expires_at,
        Some(created.record.created_at + 60)
    );

    let mut no_ttl = create_input("no-expires", "v1");
    no_ttl.ttl_secs = None;
    let created = store.create(no_ttl).expect("create no ttl");
    assert_eq!(created.record.expires_at, None);
}

// ── Idempotency ──

#[test]
fn create_idempotent_on_same_name_and_content() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    let first = store
        .create(create_input("idem", "same content"))
        .expect("create");
    let second = store
        .create(create_input("idem", "same content"))
        .expect("create again");

    assert!(second.idempotent);
    assert_eq!(second.record.skill_id, first.record.skill_id);
    assert_eq!(
        second.record.version, 1,
        "no new version on idempotent create"
    );
}

#[test]
fn update_idempotent_when_nothing_changed() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    let created = store
        .create(create_input("idem-up", "content"))
        .expect("create");
    let skill_id = created.record.skill_id.clone();

    let noop = store
        .update(
            &skill_id,
            1,
            SkillUpdateInput {
                description: created.record.description.clone(),
                content: "content".into(),
                metadata: None,
            },
        )
        .expect("no-op update");
    assert!(noop.idempotent);
    assert_eq!(noop.record.version, 1);

    let real = store
        .update(
            &skill_id,
            1,
            SkillUpdateInput {
                description: created.record.description.clone(),
                content: "changed".into(),
                metadata: None,
            },
        )
        .expect("real update");
    assert!(!real.idempotent);
    assert_eq!(real.record.version, 2);
}

// ── Unique partial index (owner, name) WHERE is_head ──

#[test]
fn unique_index_rejects_duplicate_owner_name() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    store
        .create(create_input("dup", "content a"))
        .expect("create first");

    let err = store
        .create(create_input("dup", "content b"))
        .expect_err("duplicate (owner, name) with different content");
    match err {
        VantaError::ExecutionConflict { detail, .. } => {
            assert!(detail.contains("already exists"), "detail: {detail}");
        }
        other => panic!("expected ExecutionConflict, got {other:?}"),
    }
}

#[test]
fn unique_index_allows_same_name_different_owner() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    let mut a = create_input("shared-name", "content");
    a.owner_agent = "agent-a".into();
    let mut b = create_input("shared-name", "content");
    b.owner_agent = "agent-b".into();
    store.create(a).expect("create a");
    store.create(b).expect("create b");

    let page = store
        .list(SkillListOptions {
            owner_agent: Some("agent-b".into()),
            ..Default::default()
        })
        .expect("list b");
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].owner_agent, "agent-b");
}

#[test]
fn unique_index_frees_name_after_delete() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    let created = store
        .create(create_input("reusable", "v1"))
        .expect("create");
    store.delete(&created.record.skill_id, 1).expect("delete");

    let recreated = store
        .create(create_input("reusable", "v2"))
        .expect("recreate");
    assert!(!recreated.idempotent);
    assert_eq!(recreated.record.name, "reusable");
}

// ── List filters ──

#[test]
fn list_filters_by_owner_and_name_prefix() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    for (name, owner) in [
        ("alpha-skill", "agent-1"),
        ("alpha-helper", "agent-1"),
        ("beta-skill", "agent-1"),
        ("alpha-skill", "agent-2"),
    ] {
        let mut input = create_input(name, &format!("content {name}"));
        input.owner_agent = owner.into();
        store.create(input).expect("create");
    }

    let all = store.list(SkillListOptions::default()).expect("list all");
    assert_eq!(all.total, 4);

    let prefixed = store
        .list(SkillListOptions {
            name_prefix: Some("alpha".into()),
            ..Default::default()
        })
        .expect("list prefix");
    assert_eq!(prefixed.total, 3);

    let owned = store
        .list(SkillListOptions {
            owner_agent: Some("agent-2".into()),
            ..Default::default()
        })
        .expect("list owner");
    assert_eq!(owned.total, 1);
    assert_eq!(owned.items[0].name, "alpha-skill");

    // Stale version is not listed: only heads.
    let store_ref = &store;
    let head = store_ref
        .get_head(&owned.items[0].skill_id)
        .expect("head")
        .expect("exists");
    store_ref
        .update(
            &head.skill_id,
            head.version,
            SkillUpdateInput {
                description: "bump".into(),
                content: "v2".into(),
                metadata: None,
            },
        )
        .expect("update");
    let page = store
        .list(SkillListOptions {
            owner_agent: Some("agent-2".into()),
            ..Default::default()
        })
        .expect("list after update");
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].version, 2);
}

// ── Validation ──

#[test]
fn validation_rejects_bad_identifiers() {
    let engine = in_memory_engine();
    let store = SkillStore::new(&engine);

    let bad_name = store
        .create(create_input("bad#name", "content"))
        .expect_err("bad name");
    assert!(matches!(bad_name, VantaError::ValidationError { .. }));

    let mut bad_owner = create_input("ok-name", "content");
    bad_owner.owner_agent = "owner:with:colon".into();
    let bad_owner = store.create(bad_owner).expect_err("bad owner");
    assert!(matches!(bad_owner, VantaError::ValidationError { .. }));

    assert_eq!(KEEP_RECENT, 3, "contract: TTL keep-recent=3");
}
