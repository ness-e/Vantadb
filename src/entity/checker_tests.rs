//! Dedicated tests for the permission checker (D19).
//!
//! Pattern AAA: arrange → act → assert. Same in-memory engine setup as
//! `src/entity/tests.rs`. Seeds entity records through `EntityStore` the way
//! MEM-05/MEM-07 producers will, then asserts the allow-only chain.

use super::{Action, PermissionChecker, TeamRole, Visibility};
use crate::config::VantaConfig;
use crate::entity::EntityStore;
use crate::node::FieldValue;
use crate::storage::{BackendKind, StorageEngine};
use std::collections::HashMap;

fn in_memory_engine() -> StorageEngine {
    let config = VantaConfig {
        backend_kind: BackendKind::InMemory,
        read_only: false,
        ..VantaConfig::default()
    };
    StorageEngine::open_with_config(":memory:", Some(config)).expect("open in-memory engine")
}

fn fields(pairs: &[(&str, &str)]) -> HashMap<String, FieldValue> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), FieldValue::String(v.to_string())))
        .collect()
}

const NS: &str = "default";

// ── Seed helpers ──

fn seed_member(store: &EntityStore, team_id: &str, user_id: &str, role: &str, status: &str) {
    let id = format!("{team_id}.{user_id}");
    store
        .entity_set(
            NS,
            "team_member",
            &id,
            fields(&[("role", role), ("status", status)]),
        )
        .expect("seed member");
}

fn seed_asset(
    store: &EntityStore,
    asset_id: &str,
    team_id: &str,
    owner: &str,
    visibility: &str,
    status: &str,
) {
    store
        .entity_set(
            NS,
            "asset",
            asset_id,
            fields(&[
                ("team_id", team_id),
                ("owner_user_id", owner),
                ("visibility", visibility),
                ("status", status),
            ]),
        )
        .expect("seed asset");
}

fn seed_acl(
    store: &EntityStore,
    asset_id: &str,
    subject_type: &str,
    subject_id: &str,
    permission: &str,
    effect: &str,
) {
    let id = format!("{asset_id}.{subject_type}.{subject_id}.{permission}");
    store
        .entity_set(
            NS,
            "acl",
            &id,
            fields(&[
                ("effect", effect),
                ("permission", permission),
                ("subject_id", subject_id),
                ("subject_type", subject_type),
            ]),
        )
        .expect("seed acl");
}

// ── Tests: cadena allow-only ──

#[test]
fn owner_allowed_read_write() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_asset(&store, "ast-1", "team-1", "usr-owner", "team", "active");
    let checker = PermissionChecker::new(&store);

    assert!(checker.can_read(NS, "usr-owner", "ast-1").unwrap().allowed);
    assert!(checker.can_write(NS, "usr-owner", "ast-1").unwrap().allowed);
}

#[test]
fn owner_allowed_any_action_without_membership() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    // Owner is not even a member; ownership must short-circuit before membership.
    seed_asset(&store, "ast-1", "team-1", "usr-owner", "private", "active");
    let checker = PermissionChecker::new(&store);

    let d = checker
        .can_access_asset(NS, "usr-owner", "ast-1", Action::Share, None)
        .unwrap();
    assert!(d.allowed);
    assert_eq!(d.reason, "owner");
}

#[test]
fn non_member_denied() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_asset(&store, "ast-1", "team-1", "usr-owner", "team", "active");
    let checker = PermissionChecker::new(&store);

    let d = checker.can_read(NS, "usr-outsider", "ast-1").unwrap();
    assert!(!d.allowed);
    assert_eq!(d.reason, "not_team_member");
}

#[test]
fn removed_member_denied() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-old", "member", "removed");
    seed_asset(&store, "ast-1", "team-1", "usr-owner", "team", "active");
    let checker = PermissionChecker::new(&store);

    let d = checker.can_read(NS, "usr-old", "ast-1").unwrap();
    assert!(!d.allowed);
    assert_eq!(d.reason, "not_team_member");
}

#[test]
fn member_read_only() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-m", "member", "active");
    seed_asset(&store, "ast-1", "team-1", "usr-owner", "team", "active");
    let checker = PermissionChecker::new(&store);

    assert!(checker.can_read(NS, "usr-m", "ast-1").unwrap().allowed);
    let d = checker.can_write(NS, "usr-m", "ast-1").unwrap();
    assert!(!d.allowed);
    assert_eq!(d.reason, "no_permission");
}

#[test]
fn admin_all_role_defaults() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-a", "admin", "active");
    seed_asset(&store, "ast-1", "team-1", "usr-owner", "team", "active");
    let checker = PermissionChecker::new(&store);

    for action in [Action::Read, Action::Write, Action::Assign, Action::Share] {
        let d = checker
            .can_access_asset(NS, "usr-a", "ast-1", action, None)
            .unwrap();
        assert!(d.allowed, "admin should {action:?}");
        assert!(d.reason.starts_with("role_default:admin"));
    }
}

#[test]
fn private_denies_non_owner_even_admin() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-a", "admin", "active");
    seed_asset(&store, "ast-1", "team-1", "usr-owner", "private", "active");
    let checker = PermissionChecker::new(&store);

    let d = checker.can_read(NS, "usr-a", "ast-1").unwrap();
    assert!(!d.allowed);
    assert_eq!(d.reason, "visibility_restricted");
}

#[test]
fn restricted_acl_allows_user() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-r", "member", "active");
    seed_asset(
        &store,
        "ast-1",
        "team-1",
        "usr-owner",
        "restricted",
        "active",
    );
    seed_acl(&store, "ast-1", "user", "usr-r", "read", "allow");
    seed_acl(&store, "ast-1", "user", "usr-r", "write", "allow");
    let checker = PermissionChecker::new(&store);

    assert!(checker.can_read(NS, "usr-r", "ast-1").unwrap().allowed);
    assert!(checker.can_write(NS, "usr-r", "ast-1").unwrap().allowed);
}

#[test]
fn restricted_no_acl_denies() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-r", "member", "active");
    seed_asset(
        &store,
        "ast-1",
        "team-1",
        "usr-owner",
        "restricted",
        "active",
    );
    let checker = PermissionChecker::new(&store);

    let d = checker.can_read(NS, "usr-r", "ast-1").unwrap();
    assert!(!d.allowed);
    assert_eq!(d.reason, "visibility_restricted");
}

#[test]
fn restricted_acl_deny_effect_wins() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-r", "member", "active");
    seed_asset(
        &store,
        "ast-1",
        "team-1",
        "usr-owner",
        "restricted",
        "active",
    );
    seed_acl(&store, "ast-1", "user", "usr-r", "read", "deny");
    let checker = PermissionChecker::new(&store);

    let d = checker.can_read(NS, "usr-r", "ast-1").unwrap();
    assert!(!d.allowed, "deny effect must not allow");
}

#[test]
fn restricted_admin_falls_to_defaults() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-a", "admin", "active");
    seed_asset(
        &store,
        "ast-1",
        "team-1",
        "usr-owner",
        "restricted",
        "active",
    );
    let checker = PermissionChecker::new(&store);

    let d = checker.can_read(NS, "usr-a", "ast-1").unwrap();
    assert!(d.allowed, "restricted does not block admins");
    assert_eq!(d.reason, "role_default:admin");
}

#[test]
fn task_visibility_read_only_for_member() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-m", "member", "active");
    seed_asset(&store, "ast-1", "team-1", "usr-owner", "task", "active");
    let checker = PermissionChecker::new(&store);

    assert!(checker.can_read(NS, "usr-m", "ast-1").unwrap().allowed);
    let d = checker.can_write(NS, "usr-m", "ast-1").unwrap();
    assert!(!d.allowed);
    assert_eq!(d.reason, "visibility_restricted");
}

#[test]
fn task_admin_write_ok() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-a", "admin", "active");
    seed_asset(&store, "ast-1", "team-1", "usr-owner", "task", "active");
    let checker = PermissionChecker::new(&store);

    assert!(checker.can_write(NS, "usr-a", "ast-1").unwrap().allowed);
}

#[test]
fn archived_asset_denied() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-a", "admin", "active");
    seed_asset(&store, "ast-1", "team-1", "usr-owner", "team", "archived");
    let checker = PermissionChecker::new(&store);

    let d = checker.can_read(NS, "usr-a", "ast-1").unwrap();
    assert!(!d.allowed);
    assert_eq!(d.reason, "asset_not_available");
}

#[test]
fn missing_asset_denied() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    let checker = PermissionChecker::new(&store);

    let d = checker.can_read(NS, "usr-anyone", "ast-ghost").unwrap();
    assert!(!d.allowed);
    assert_eq!(d.reason, "asset_not_available");
}

#[test]
fn acl_team_role_subject() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-m", "reviewer", "active");
    seed_asset(
        &store,
        "ast-1",
        "team-1",
        "usr-owner",
        "restricted",
        "active",
    );
    seed_acl(&store, "ast-1", "team_role", "reviewer", "write", "allow");
    let checker = PermissionChecker::new(&store);

    assert!(checker.can_write(NS, "usr-m", "ast-1").unwrap().allowed);
}

#[test]
fn acl_agent_subject_requires_agent_id() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-o", "member", "active");
    seed_asset(
        &store,
        "ast-1",
        "team-1",
        "usr-owner",
        "restricted",
        "active",
    );
    seed_acl(&store, "ast-1", "agent", "agt-9", "use", "allow");
    let checker = PermissionChecker::new(&store);

    // Sin agent_id → no hay subject agent candidato.
    let d = checker
        .can_access_asset(NS, "usr-o", "ast-1", Action::Use, None)
        .unwrap();
    assert!(!d.allowed);

    // Con agent_id coincidente → allow vía ACL.
    let d = checker
        .can_access_asset(NS, "usr-o", "ast-1", Action::Use, Some("agt-9"))
        .unwrap();
    assert!(d.allowed);
    assert_eq!(d.reason, "acl");
}

#[test]
fn unknown_visibility_denies() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-a", "admin", "active");
    seed_asset(&store, "ast-1", "team-1", "usr-owner", "weird", "active");
    let checker = PermissionChecker::new(&store);

    let d = checker.can_read(NS, "usr-a", "ast-1").unwrap();
    assert!(!d.allowed);
    assert_eq!(d.reason, "visibility_restricted");
}

// ── Tests: is_admin / is_member ──

#[test]
fn is_admin_true_only_for_active_admin() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-a", "admin", "active");
    seed_member(&store, "team-1", "usr-a2", "admin", "removed");
    seed_member(&store, "team-1", "usr-m", "member", "active");
    let checker = PermissionChecker::new(&store);

    assert!(checker.is_admin(NS, "usr-a", "team-1").unwrap());
    assert!(
        !checker.is_admin(NS, "usr-a2", "team-1").unwrap(),
        "removed admin is not admin"
    );
    assert!(!checker.is_admin(NS, "usr-m", "team-1").unwrap());
    assert!(!checker.is_admin(NS, "usr-nobody", "team-1").unwrap());
}

#[test]
fn is_member_true_only_for_active_membership() {
    let engine = in_memory_engine();
    let store = EntityStore::new(&engine);
    seed_member(&store, "team-1", "usr-m", "member", "active");
    seed_member(&store, "team-1", "usr-x", "member", "removed");
    let checker = PermissionChecker::new(&store);

    assert!(checker.is_member(NS, "usr-m", "team-1").unwrap());
    assert!(!checker.is_member(NS, "usr-x", "team-1").unwrap());
    assert!(!checker.is_member(NS, "usr-ghost", "team-1").unwrap());
    assert!(
        !checker.is_member(NS, "usr-m", "team-other").unwrap(),
        "team isolation"
    );
}

// ── Tests: tipos (R-6 non_exhaustive, as_str) ──

#[test]
fn action_str_roundtrip() {
    assert_eq!(Action::Read.as_str(), "read");
    assert_eq!(Action::Write.as_str(), "write");
    assert_eq!(Action::Assign.as_str(), "assign");
    assert_eq!(Action::Share.as_str(), "share");
    assert_eq!(Action::Use.as_str(), "use");
}

#[test]
fn visibility_matches_tdam_values() {
    assert_eq!(Visibility::Private.as_str(), "private");
    assert_eq!(Visibility::Restricted.as_str(), "restricted");
    assert_eq!(Visibility::Task.as_str(), "task");
}

#[test]
fn team_role_str_matches_tdam_values() {
    assert_eq!(TeamRole::Admin.as_str(), "admin");
    assert_eq!(TeamRole::Member.as_str(), "member");
    assert_eq!(TeamRole::Reviewer.as_str(), "reviewer");
}
