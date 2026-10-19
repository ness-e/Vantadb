#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Unit tests for the sharing facade (MEMG-16).
//!
//! Invariants guarded (contract `docs/dev/tasks/MEMG-16.md` §Contrato):
//! - (a) a grant enables shared access between 2 users/agents;
//! - (b) after revoking, the *next* access is denied (ACL and membership);
//! - fail-closed: missing asset / non-member / no-grant all deny;
//! - a deny never returns the record (no existence oracle);
//! - grants reuse the checker subjects (user / team_role / agent), agent
//!   subjects only match when `agent_id` is supplied.

use super::{
    AccessQuery, Action, GrantInput, PermDecision, ShareAssetInput, Subject, TeamMemberInput,
    TeamRole, Visibility,
};
use crate::config::Config;
use crate::sdk::builder::Embedded;
use crate::sdk::types::MemoryInput;

const NS: &str = "team-a";
const TEAM: &str = "core";

fn db() -> Embedded {
    let config = Config {
        storage_path: ":memory:".into(),
        backend_kind: crate::BackendKind::InMemory,
        ..Default::default()
    };
    Embedded::open_with_config(config).expect("open in-memory Embedded")
}

fn share(
    db: &Embedded,
    asset_id: &str,
    owner: &str,
    visibility: Visibility,
) -> crate::entity::Entity {
    db.share_asset(ShareAssetInput {
        namespace: NS,
        asset_id,
        owner_user_id: owner,
        team_id: TEAM,
        visibility,
    })
    .expect("share asset")
}

fn add_member(db: &Embedded, user: &str, role: TeamRole) {
    db.add_team_member(TeamMemberInput {
        namespace: NS,
        team_id: TEAM,
        user_id: user,
        role,
    })
    .expect("add team member");
}

fn check(
    db: &Embedded,
    user: &str,
    asset: &str,
    action: Action,
    agent: Option<&str>,
) -> PermDecision {
    db.check_access(AccessQuery {
        namespace: NS,
        user_id: user,
        asset_id: asset,
        action,
        agent_id: agent,
    })
    .expect("check access")
}

fn grant(db: &Embedded, asset: &str, subject: Subject, action: Action) {
    db.grant_access(GrantInput {
        namespace: NS,
        asset_id: asset,
        subject,
        action,
    })
    .expect("grant");
}

fn revoke(db: &Embedded, asset: &str, subject: Subject, action: Action) -> bool {
    db.revoke_access(GrantInput {
        namespace: NS,
        asset_id: asset,
        subject,
        action,
    })
    .expect("revoke")
}

// ── (a) + (b): grant enables, revoke denies the next access ──

#[test]
fn grant_to_user_enables_write_and_revoke_denies_next_access() {
    let db = db();
    db.put(MemoryInput::new(NS, "doc-1", "shared payload"))
        .expect("put");
    share(&db, "doc-1", "usr-alice", Visibility::Team);
    add_member(&db, "usr-bob", TeamRole::Member);

    // Without a grant a member can read (role default) but not write.
    let before = check(&db, "usr-bob", "doc-1", Action::Write, None);
    assert!(!before.allowed);
    assert_eq!(before.reason, "no_permission");

    // (a) The grant enables the shared access.
    grant(&db, "doc-1", Subject::user("usr-bob"), Action::Write);
    let granted = check(&db, "usr-bob", "doc-1", Action::Write, None);
    assert!(granted.allowed);
    assert_eq!(granted.reason, "acl");

    // The checked operation honours the grant.
    let outcome = db
        .put_shared(
            "usr-bob",
            MemoryInput::new(NS, "doc-1", "edited by bob"),
            None,
        )
        .expect("put_shared");
    assert!(outcome.decision.allowed);
    assert_eq!(
        outcome.record.expect("written record").payload,
        "edited by bob"
    );

    // (b) Revoking denies the next access — and the checked write lands nothing.
    assert!(revoke(
        &db,
        "doc-1",
        Subject::user("usr-bob"),
        Action::Write
    ));
    let after = check(&db, "usr-bob", "doc-1", Action::Write, None);
    assert!(!after.allowed);
    assert_eq!(after.reason, "no_permission");

    let denied = db
        .put_shared(
            "usr-bob",
            MemoryInput::new(NS, "doc-1", "must not land"),
            None,
        )
        .expect("put_shared");
    assert!(!denied.decision.allowed);
    assert!(denied.record.is_none(), "deny must not return a record");
    assert_eq!(
        db.get(NS, "doc-1").expect("get").expect("record").payload,
        "edited by bob",
        "denied write must not modify the stored record"
    );

    // Revoking again is honest: nothing to delete.
    assert!(!revoke(
        &db,
        "doc-1",
        Subject::user("usr-bob"),
        Action::Write
    ));
}

#[test]
fn grant_read_on_restricted_asset_enables_and_revoke_denies() {
    let db = db();
    db.put(MemoryInput::new(NS, "doc-1", "restricted payload"))
        .expect("put");
    share(&db, "doc-1", "usr-alice", Visibility::Restricted);
    add_member(&db, "usr-bob", TeamRole::Member);

    // Restricted + no ACL → denied before the grant.
    let before = db
        .get_shared(NS, "usr-bob", "doc-1", None)
        .expect("get_shared");
    assert!(!before.decision.allowed);
    assert_eq!(before.decision.reason, "visibility_restricted");
    assert!(before.record.is_none(), "deny must not return a record");

    // (a) Grant read → shared read works.
    grant(&db, "doc-1", Subject::user("usr-bob"), Action::Read);
    let granted = db
        .get_shared(NS, "usr-bob", "doc-1", None)
        .expect("get_shared");
    assert!(granted.decision.allowed);
    assert_eq!(
        granted.record.expect("shared record").payload,
        "restricted payload"
    );

    // (b) Revoke → next access denied again.
    assert!(revoke(&db, "doc-1", Subject::user("usr-bob"), Action::Read));
    let after = db
        .get_shared(NS, "usr-bob", "doc-1", None)
        .expect("get_shared");
    assert!(!after.decision.allowed);
    assert_eq!(after.decision.reason, "visibility_restricted");
    assert!(after.record.is_none());
}

// ── agent subjects ──

#[test]
fn agent_grant_requires_matching_agent_id() {
    let db = db();
    db.put(MemoryInput::new(NS, "doc-1", "agent payload"))
        .expect("put");
    share(&db, "doc-1", "usr-alice", Visibility::Restricted);
    add_member(&db, "usr-carol", TeamRole::Member);
    grant(&db, "doc-1", Subject::agent("agt-9"), Action::Use);

    // Without agent_id there is no agent subject candidate → denied.
    let without = check(&db, "usr-carol", "doc-1", Action::Use, None);
    assert!(!without.allowed);

    // With the matching agent_id the ACL allows (a).
    let with = check(&db, "usr-carol", "doc-1", Action::Use, Some("agt-9"));
    assert!(with.allowed);
    assert_eq!(with.reason, "acl");

    // (b) Revoking the agent grant denies the next access.
    assert!(revoke(&db, "doc-1", Subject::agent("agt-9"), Action::Use));
    let after = check(&db, "usr-carol", "doc-1", Action::Use, Some("agt-9"));
    assert!(!after.allowed);
}

// ── membership revocation ──

#[test]
fn revoke_team_member_denies_next_access() {
    let db = db();
    db.put(MemoryInput::new(NS, "doc-1", "team payload"))
        .expect("put");
    share(&db, "doc-1", "usr-alice", Visibility::Team);
    add_member(&db, "usr-bob", TeamRole::Member);

    // Membership alone (role default) allows the read.
    assert!(check(&db, "usr-bob", "doc-1", Action::Read, None).allowed);

    // (b) Membership revocation → next access denied.
    assert!(db
        .revoke_team_member(NS, TEAM, "usr-bob")
        .expect("revoke member"));
    let after = check(&db, "usr-bob", "doc-1", Action::Read, None);
    assert!(!after.allowed);
    assert_eq!(after.reason, "not_team_member");

    // Revoking a non-member is honest: nothing to revoke.
    assert!(!db
        .revoke_team_member(NS, TEAM, "usr-ghost")
        .expect("revoke member"));
}

// ── fail-closed ──

#[test]
fn missing_asset_denies_checked_ops() {
    let db = db();
    db.put(MemoryInput::new(NS, "doc-1", "unshared payload"))
        .expect("put");
    add_member(&db, "usr-bob", TeamRole::Member);
    // No share_asset call → no asset entity.

    let read = db
        .get_shared(NS, "usr-bob", "doc-1", None)
        .expect("get_shared");
    assert!(!read.decision.allowed);
    assert_eq!(read.decision.reason, "asset_not_available");
    assert!(read.record.is_none(), "unshared record must not leak");

    let write = db
        .put_shared("usr-bob", MemoryInput::new(NS, "doc-1", "nope"), None)
        .expect("put_shared");
    assert!(!write.decision.allowed);
    assert!(write.record.is_none());
}

#[test]
fn grant_does_not_bypass_membership() {
    let db = db();
    db.put(MemoryInput::new(NS, "doc-1", "payload"))
        .expect("put");
    share(&db, "doc-1", "usr-alice", Visibility::Team);
    // usr-outsider is NOT a member.
    grant(&db, "doc-1", Subject::user("usr-outsider"), Action::Read);

    let d = check(&db, "usr-outsider", "doc-1", Action::Read, None);
    assert!(!d.allowed, "a grant must not bypass team membership");
    assert_eq!(d.reason, "not_team_member");
}

// ── entity management semantics ──

#[test]
fn share_asset_upsert_preserves_created_at_and_owner_short_circuits() {
    let db = db();
    let first = share(&db, "doc-1", "usr-alice", Visibility::Team);
    let second = share(&db, "doc-1", "usr-alice", Visibility::Restricted);

    assert_eq!(first.created_at, second.created_at, "created_at preserved");
    assert!(second.updated_at >= first.updated_at);
    assert_eq!(
        second.fields.get("visibility"),
        Some(&crate::node::FieldValue::String("restricted".into()))
    );

    // The owner short-circuits the chain regardless of visibility.
    let owner = check(&db, "usr-alice", "doc-1", Action::Write, None);
    assert!(owner.allowed);
    assert_eq!(owner.reason, "owner");
}

#[test]
fn rejects_ids_with_composite_key_separators() {
    // `.` is the checker's composite-key separator (`{team}.{user}`,
    // `{asset}.{subject_type}.{subject}.{action}`): allowing it inside a
    // component would make distinct grants/memberships collide
    // (`src/entity/checker.rs:25`). Regression for review P2-01 R1.
    let db = db();

    let err = db
        .add_team_member(TeamMemberInput {
            namespace: NS,
            team_id: "core.a",
            user_id: "b",
            role: TeamRole::Member,
        })
        .expect_err("dotted team id would collide with {team}.{user}");
    assert!(matches!(err, crate::error::Error::Validation { .. }));

    let err = db
        .grant_access(GrantInput {
            namespace: NS,
            asset_id: "doc-1",
            subject: Subject::user("b.user.c"),
            action: Action::Read,
        })
        .expect_err("dotted subject id would collide with the ACL key");
    assert!(matches!(err, crate::error::Error::Validation { .. }));

    let err = db
        .share_asset(ShareAssetInput {
            namespace: NS,
            asset_id: "a.user.b",
            owner_user_id: "usr-alice",
            team_id: TEAM,
            visibility: Visibility::Team,
        })
        .expect_err("dotted asset id would collide with the ACL key");
    assert!(matches!(err, crate::error::Error::Validation { .. }));

    let err = db
        .check_access(AccessQuery {
            namespace: NS,
            user_id: "a.b",
            asset_id: "doc-1",
            action: Action::Read,
            agent_id: None,
        })
        .expect_err("dotted query user id would collide with {team}.{user}");
    assert!(matches!(err, crate::error::Error::Validation { .. }));

    let err = db
        .check_access(AccessQuery {
            namespace: NS,
            user_id: "usr-bob",
            asset_id: "doc-1",
            action: Action::Use,
            agent_id: Some("agt.9"),
        })
        .expect_err("dotted agent id would collide with the ACL key");
    assert!(matches!(err, crate::error::Error::Validation { .. }));
}

#[test]
fn rejects_invalid_input_at_the_boundary() {
    let db = db();
    let err = db
        .share_asset(ShareAssetInput {
            namespace: NS,
            asset_id: "",
            owner_user_id: "usr-alice",
            team_id: TEAM,
            visibility: Visibility::Team,
        })
        .expect_err("empty asset id must be rejected");
    assert!(matches!(err, crate::error::Error::Validation { .. }));

    let err = db
        .grant_access(GrantInput {
            namespace: NS,
            asset_id: "doc-1",
            subject: Subject::user(""),
            action: Action::Read,
        })
        .expect_err("empty subject id must be rejected");
    assert!(matches!(err, crate::error::Error::Validation { .. }));

    let err = db
        .check_access(AccessQuery {
            namespace: NS,
            user_id: "",
            asset_id: "doc-1",
            action: Action::Read,
            agent_id: None,
        })
        .expect_err("empty user id must be rejected");
    assert!(matches!(err, crate::error::Error::Validation { .. }));
}

#[test]
fn team_role_grant_covers_role_subject() {
    let db = db();
    db.put(MemoryInput::new(NS, "doc-1", "payload"))
        .expect("put");
    share(&db, "doc-1", "usr-alice", Visibility::Restricted);
    add_member(&db, "usr-r", TeamRole::Reviewer);
    grant(
        &db,
        "doc-1",
        Subject::team_role(TeamRole::Reviewer),
        Action::Write,
    );

    let d = check(&db, "usr-r", "doc-1", Action::Write, None);
    assert!(d.allowed);
    assert_eq!(d.reason, "acl");

    assert!(revoke(
        &db,
        "doc-1",
        Subject::team_role(TeamRole::Reviewer),
        Action::Write
    ));
    assert!(!check(&db, "usr-r", "doc-1", Action::Write, None).allowed);
}
