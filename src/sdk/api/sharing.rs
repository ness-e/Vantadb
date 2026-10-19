//! Shared-memory facade (MEMG-16): scopes, grants, and revocation over the
//! existing [`PermissionChecker`](crate::entity::checker::PermissionChecker)
//! and entity ACL machinery.
//!
//! # Scope model (declared)
//!
//! | Scope | Maps to | Enforcement |
//! |-------|---------|-------------|
//! | **org** | namespace (tenant root; MEMG-04 RBAC isolates tenants) | namespace boundary |
//! | **project** | namespace sub-path `{org}/{project}` (e.g. `acme/atlas`) | same namespace boundary |
//! | **team** | `team_member` entities (`team_id.user_id`, role + status) | checker membership step + role defaults |
//! | **asset** | `asset` entity (id = record key) + `acl` grants | checker visibility + ACL chain |
//!
//! Sharing entities live in the same `InternalMetadata` partition as every
//! other entity (`entity:{namespace}:{collection}::{id}`); nothing is
//! duplicated and no new storage mechanism is introduced.
//!
//! # Revocation semantics
//!
//! Revocation is **future-access only, never a purge**: [`Embedded::revoke_access`]
//! deletes the ACL entity and [`Embedded::revoke_team_member`] flips the
//! membership `status` to `removed`. The checker reads live entities on every
//! call (no cache), so the *next* access is denied. Data already delivered or
//! copied is not un-shared — erasing shared content is the erasure path
//! (MEMG-17), a different operation with its own contract.
//!
//! # Trust model
//!
//! The management methods (`share_asset`, `grant_access`, `revoke_access`,
//! membership ops) carry the same trust level as the rest of the embedded SDK:
//! the embedding application decides who may call them. Exposing them over
//! HTTP/MCP requires the caller to gate them with its own principal/RBAC
//! layer (server surface: FIND-305).
//!
//! Full model, boundaries (MEMG-17 / SCH-05 / MEMG-10), containment and trust
//! notes: `docs/api/SHARING.md`.

use std::collections::HashMap;

use super::super::builder::Embedded;
use super::super::types::{MemoryInput, MemoryRecord};
use crate::audit::AuditEvent;
use crate::entity::checker::PermissionChecker;
use crate::entity::{Entity, EntityStore, EntityWrite};
use crate::error::{Error, Result};
use crate::node::FieldValue;

pub use crate::entity::checker::{Action, PermDecision, TeamRole, Visibility};

/// Subject of a grant — parity with the checker ACL subjects
/// (`user` / `team_role` / `agent`).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subject {
    /// A user id (resolved principal).
    User(String),
    /// A team role (`admin` / `member` / `reviewer`).
    TeamRole(TeamRole),
    /// An agent id (requires `agent_id` at check time).
    Agent(String),
}

impl Subject {
    /// Grant subject: user id.
    pub fn user(id: impl Into<String>) -> Self {
        Self::User(id.into())
    }

    /// Grant subject: team role.
    pub fn team_role(role: TeamRole) -> Self {
        Self::TeamRole(role)
    }

    /// Grant subject: agent id.
    pub fn agent(id: impl Into<String>) -> Self {
        Self::Agent(id.into())
    }

    /// `(subject_type, subject_id)` wire pair used in ACL keys and fields.
    fn as_pair(&self) -> (&'static str, &str) {
        match self {
            Self::User(id) => ("user", id),
            Self::TeamRole(role) => ("team_role", role.as_str()),
            Self::Agent(id) => ("agent", id),
        }
    }
}

/// Command object for [`Embedded::share_asset`] (D3: F2 command-object).
///
/// `asset_id` is the memory record key (convention declared in
/// `docs/api/SHARING.md`); ids must not contain `.`, `{`, `}` or `:`
/// (composite-key separators) — validated at the boundary.
#[derive(Debug, Clone)]
pub struct ShareAssetInput<'a> {
    /// Namespace (org/project scope) holding the asset.
    pub namespace: &'a str,
    /// Asset id — the shared memory record key.
    pub asset_id: &'a str,
    /// Owner principal; the owner always short-circuits the checker chain.
    pub owner_user_id: &'a str,
    /// Team the asset belongs to (membership is checked against it).
    pub team_id: &'a str,
    /// Asset visibility (`private` / `team` / `restricted` / `agent` / `task`).
    pub visibility: Visibility,
}

/// Command object for [`Embedded::add_team_member`].
#[derive(Debug, Clone)]
pub struct TeamMemberInput<'a> {
    /// Namespace holding the membership.
    pub namespace: &'a str,
    /// Team id.
    pub team_id: &'a str,
    /// User id.
    pub user_id: &'a str,
    /// Role granted by the membership.
    pub role: TeamRole,
}

/// Command object for [`Embedded::grant_access`] / [`Embedded::revoke_access`].
#[derive(Debug, Clone)]
pub struct GrantInput<'a> {
    /// Namespace holding the ACL.
    pub namespace: &'a str,
    /// Asset id the grant applies to.
    pub asset_id: &'a str,
    /// Grant subject.
    pub subject: Subject,
    /// Action granted/revoked.
    pub action: Action,
}

/// Query for [`Embedded::check_access`].
#[derive(Debug, Clone)]
pub struct AccessQuery<'a> {
    /// Namespace holding the asset.
    pub namespace: &'a str,
    /// Principal user id (membership is resolved for it).
    pub user_id: &'a str,
    /// Asset id.
    pub asset_id: &'a str,
    /// Action to evaluate.
    pub action: Action,
    /// Optional agent id — only consulted for `agent` ACL subjects.
    pub agent_id: Option<&'a str>,
}

/// Outcome of a checked shared operation (read/write).
///
/// `decision.allowed == false` implies `record == None` — a deny never
/// returns (or writes) the record, so it cannot be used as an existence
/// oracle.
#[derive(Debug, Clone)]
pub struct SharedOutcome {
    /// Full checker decision (`allowed` + `reason`).
    pub decision: PermDecision,
    /// The record, only present when the operation was allowed.
    pub record: Option<MemoryRecord>,
}

impl Embedded {
    /// Share (or update the sharing metadata of) a memory record.
    ///
    /// Upserts the `asset` entity: `team_id`, `owner_user_id`, `visibility`
    /// and `status = active`. The record itself is untouched — write it with
    /// [`Embedded::put`] (or [`Embedded::put_shared`]).
    #[tracing::instrument(skip(self, input), err)]
    pub fn share_asset(&self, input: ShareAssetInput<'_>) -> Result<Entity> {
        let res = self.share_asset_inner(&input);
        self.audit(AuditEvent::new(
            "share",
            input.namespace,
            input.asset_id,
            if res.is_ok() { "ok" } else { "err" },
            None,
        ));
        res
    }

    fn share_asset_inner(&self, input: &ShareAssetInput<'_>) -> Result<Entity> {
        validate_entity_id("asset_id", input.asset_id)?;
        validate_entity_id("owner_user_id", input.owner_user_id)?;
        validate_entity_id("team_id", input.team_id)?;
        let engine = self.engine_handle()?;
        let store = EntityStore::new(&engine);
        let mut fields = HashMap::new();
        fields.insert(
            "team_id".to_string(),
            FieldValue::String(input.team_id.to_string()),
        );
        fields.insert(
            "owner_user_id".to_string(),
            FieldValue::String(input.owner_user_id.to_string()),
        );
        fields.insert(
            "visibility".to_string(),
            FieldValue::String(input.visibility.as_str().to_string()),
        );
        fields.insert(
            "status".to_string(),
            FieldValue::String("active".to_string()),
        );
        store.set(EntityWrite {
            namespace: input.namespace,
            collection: "asset",
            id: input.asset_id,
            fields,
        })
    }

    /// Add (or re-activate) a team membership: grants the role defaults for
    /// the team (`admin` → read/write/assign/share; `member`/`reviewer` →
    /// read).
    #[tracing::instrument(skip(self, input), err)]
    pub fn add_team_member(&self, input: TeamMemberInput<'_>) -> Result<Entity> {
        let res = self.add_team_member_inner(&input);
        self.audit(AuditEvent::new(
            "member_add",
            input.namespace,
            &format!("{}.{}", input.team_id, input.user_id),
            if res.is_ok() { "ok" } else { "err" },
            None,
        ));
        res
    }

    fn add_team_member_inner(&self, input: &TeamMemberInput<'_>) -> Result<Entity> {
        validate_entity_id("team_id", input.team_id)?;
        validate_entity_id("user_id", input.user_id)?;
        let engine = self.engine_handle()?;
        let store = EntityStore::new(&engine);
        let id = format!("{}.{}", input.team_id, input.user_id);
        let mut fields = HashMap::new();
        fields.insert(
            "role".to_string(),
            FieldValue::String(input.role.as_str().to_string()),
        );
        fields.insert(
            "status".to_string(),
            FieldValue::String("active".to_string()),
        );
        store.set(EntityWrite {
            namespace: input.namespace,
            collection: "team_member",
            id: &id,
            fields,
        })
    }

    /// Revoke a team membership: flips `status` to `removed` (the entity is
    /// kept for audit) — the next access is denied `not_team_member`.
    ///
    /// Returns `true` when a membership existed and is now revoked.
    #[tracing::instrument(skip(self), err)]
    pub fn revoke_team_member(
        &self,
        namespace: &str,
        team_id: &str,
        user_id: &str,
    ) -> Result<bool> {
        let res = self.revoke_team_member_inner(namespace, team_id, user_id);
        let reason = match &res {
            Ok(true) => "removed",
            Ok(false) => "not_found",
            Err(_) => "err",
        };
        self.audit(AuditEvent::new(
            "member_revoke",
            namespace,
            &format!("{team_id}.{user_id}"),
            if res.is_ok() { "ok" } else { "err" },
            Some(reason.to_string()),
        ));
        res
    }

    fn revoke_team_member_inner(
        &self,
        namespace: &str,
        team_id: &str,
        user_id: &str,
    ) -> Result<bool> {
        validate_entity_id("team_id", team_id)?;
        validate_entity_id("user_id", user_id)?;
        let engine = self.engine_handle()?;
        let store = EntityStore::new(&engine);
        let id = format!("{team_id}.{user_id}");
        let Some(existing) = store.get(namespace, "team_member", &id)? else {
            return Ok(false);
        };
        let mut fields = existing.fields;
        fields.insert(
            "status".to_string(),
            FieldValue::String("removed".to_string()),
        );
        store.set(EntityWrite {
            namespace,
            collection: "team_member",
            id: &id,
            fields,
        })?;
        Ok(true)
    }

    /// Grant an action on an asset to a subject: upserts the ACL allow
    /// entity (key `{asset_id}.{subject_type}.{subject_id}.{action}`).
    #[tracing::instrument(skip(self, input), err)]
    pub fn grant_access(&self, input: GrantInput<'_>) -> Result<Entity> {
        let res = self.grant_access_inner(&input);
        self.audit(AuditEvent::new(
            "grant",
            input.namespace,
            input.asset_id,
            if res.is_ok() { "ok" } else { "err" },
            Some(subject_reason(&input)),
        ));
        res
    }

    fn grant_access_inner(&self, input: &GrantInput<'_>) -> Result<Entity> {
        validate_entity_id("asset_id", input.asset_id)?;
        let (subject_type, subject_id) = input.subject.as_pair();
        validate_entity_id("subject_id", subject_id)?;
        let engine = self.engine_handle()?;
        let store = EntityStore::new(&engine);
        let action = input.action.as_str();
        let id = format!(
            "{}.{}.{}.{}",
            input.asset_id, subject_type, subject_id, action
        );
        let mut fields = HashMap::new();
        fields.insert(
            "effect".to_string(),
            FieldValue::String("allow".to_string()),
        );
        fields.insert(
            "permission".to_string(),
            FieldValue::String(action.to_string()),
        );
        fields.insert(
            "subject_id".to_string(),
            FieldValue::String(subject_id.to_string()),
        );
        fields.insert(
            "subject_type".to_string(),
            FieldValue::String(subject_type.to_string()),
        );
        store.set(EntityWrite {
            namespace: input.namespace,
            collection: "acl",
            id: &id,
            fields,
        })
    }

    /// Revoke a grant: deletes the ACL entity — the next access is denied.
    ///
    /// Returns `true` when a grant existed and was removed.
    #[tracing::instrument(skip(self, input), err)]
    pub fn revoke_access(&self, input: GrantInput<'_>) -> Result<bool> {
        let res = self.revoke_access_inner(&input);
        self.audit(AuditEvent::new(
            "revoke",
            input.namespace,
            input.asset_id,
            if res.is_ok() { "ok" } else { "err" },
            Some(revoke_reason(&input, res.as_ref().ok().copied())),
        ));
        res
    }

    fn revoke_access_inner(&self, input: &GrantInput<'_>) -> Result<bool> {
        validate_entity_id("asset_id", input.asset_id)?;
        let (subject_type, subject_id) = input.subject.as_pair();
        validate_entity_id("subject_id", subject_id)?;
        let engine = self.engine_handle()?;
        let store = EntityStore::new(&engine);
        let id = format!(
            "{}.{}.{}.{}",
            input.asset_id,
            subject_type,
            subject_id,
            input.action.as_str()
        );
        store.delete(input.namespace, "acl", &id)
    }

    /// Evaluate the checker chain (resource → owner → membership →
    /// visibility → role-default → ACL → deny) for one principal/asset/action.
    ///
    /// The decision is data (`allowed` + `reason`); callers map it to their
    /// surface (e.g. HTTP 403). Fail-closed: a missing/archived asset, a
    /// non-member, or no allow rule all deny.
    #[tracing::instrument(skip(self, query), err)]
    pub fn check_access(&self, query: AccessQuery<'_>) -> Result<PermDecision> {
        validate_entity_id("user_id", query.user_id)?;
        validate_entity_id("asset_id", query.asset_id)?;
        if let Some(agent_id) = query.agent_id {
            validate_entity_id("agent_id", agent_id)?;
        }
        let engine = self.engine_handle()?;
        let store = EntityStore::new(&engine);
        let checker = PermissionChecker::new(&store);
        checker.can_access_asset(
            query.namespace,
            query.user_id,
            query.asset_id,
            query.action,
            query.agent_id,
        )
    }

    /// Checked shared read: evaluates [`Action::Read`] and returns the record
    /// only when allowed (`record: None` on deny — no existence oracle).
    #[tracing::instrument(skip(self), err)]
    pub fn get_shared(
        &self,
        namespace: &str,
        user_id: &str,
        key: &str,
        agent_id: Option<&str>,
    ) -> Result<SharedOutcome> {
        let decision = self.check_access(AccessQuery {
            namespace,
            user_id,
            asset_id: key,
            action: Action::Read,
            agent_id,
        })?;
        let record = if decision.allowed {
            self.get(namespace, key)?
        } else {
            None
        };
        Ok(SharedOutcome { decision, record })
    }

    /// Checked shared write: evaluates [`Action::Write`] and stores the input
    /// only when allowed (`record: None` on deny — nothing is written).
    #[tracing::instrument(skip(self, input), err)]
    pub fn put_shared(
        &self,
        user_id: &str,
        input: MemoryInput,
        agent_id: Option<&str>,
    ) -> Result<SharedOutcome> {
        let decision = self.check_access(AccessQuery {
            namespace: &input.namespace,
            user_id,
            asset_id: &input.key,
            action: Action::Write,
            agent_id,
        })?;
        let record = if decision.allowed {
            Some(self.put(input)?)
        } else {
            None
        };
        Ok(SharedOutcome { decision, record })
    }
}

/// `subject_type:subject_id:action` — audit reason (never secrets).
fn subject_reason(input: &GrantInput<'_>) -> String {
    let (subject_type, subject_id) = input.subject.as_pair();
    format!("{subject_type}:{subject_id}:{}", input.action.as_str())
}

/// Audit reason of a revocation: subject + action + whether anything was removed.
fn revoke_reason(input: &GrantInput<'_>, removed: Option<bool>) -> String {
    let outcome = match removed {
        Some(true) => "removed",
        Some(false) => "not_found",
        None => "err",
    };
    format!("{}:{}", subject_reason(input), outcome)
}

/// Boundary validation for ids composed into entity keys.
///
/// The checker composes `{team_id}.{user_id}` (membership) and
/// `{asset_id}.{subject_type}.{subject_id}.{action}` (ACL) — a `.` inside a
/// component would let distinct grants/memberships collide (`checker.rs:25`).
/// Ids must also satisfy the entity charset (`{`, `}`, `:` are rejected by
/// `EntityStore::validate_key`).
fn validate_entity_id(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Error::Validation {
            field: field.to_string(),
            reason: "must not be empty".to_string(),
        });
    }
    if value.contains(['.', '{', '}', ':']) {
        return Err(Error::Validation {
            field: field.to_string(),
            reason: "must not contain '.', '{', '}' or ':' (entity key separators)".to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
#[path = "sharing_tests.rs"]
mod tests;
