//! Permission checker (allow-only) — eslabón "permission" de la cadena D7.
//!
//! [`PermissionChecker`] decide si un usuario puede actuar sobre un *asset*
//! leyendo las entidades `entity_*` (MEM-03) con semántica **allow-only**: si
//! no existe una regla que permita explícitamente la acción → denegar. Port del
//! algoritmo de TDAM `metadata/service/permission-checker.ts` (172 líneas
//! reales; la contradicción SYNTHESIS "96 vs ~40" se resolvió contra el clon
//! @ `97f9465` — la fuente de verdad es el archivo, no el reporte).
//!
//! Cadena de decisión (orden importa, port fiel del TDAM):
//! resource → owner → membership → visibility → role-default → ACL → deny.
//!
//! Convención de collections/keys (consumidores: MEM-05 auth 3 capas, MEM-07
//! skills, MEM-35 data plane):
//! - `user`: collection `user`, id = `user_id` (fields: `status`, `user_type`)
//! - `team`: collection `team`, id = `team_id` (fields: `owner_user_id`, `status`)
//! - membership: collection `team_member`, id = `{team_id}.{user_id}`
//!   (fields: `role` ∈ admin/member/reviewer, `status` ∈ active/removed)
//! - asset: collection `asset`, id = `asset_id`
//!   (fields: `team_id`, `owner_user_id`, `visibility` ∈
//!   private/team/restricted/agent/task, `status` ∈ …/archived)
//! - acl: collection `acl`, id = `{asset_id}.{subject_type}.{subject_id}.{permission}`
//!   (fields: `effect` ∈ allow/deny) — una regla por (asset, subject, action).
//!
//! El separador `.` es seguro: los ids de [`crate::entity::generate_id`] solo
//! contienen `[a-z0-9-]`, y `validate_key` rechaza `{`, `}`, `:`.

use crate::entity::EntityStore;
use crate::error::Result;
use crate::node::FieldValue;

// ── Types ──

/// Acción evaluada por el checker (port de `Permission` TDAM).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Read,
    Write,
    Assign,
    Share,
    Use,
}

/// Visibilidad de un asset (port de `AssetVisibility` TDAM).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Team,
    Restricted,
    Agent,
    Task,
}

/// Rol de un miembro de equipo (port de `TeamRole` TDAM).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamRole {
    Admin,
    Member,
    Reviewer,
}

/// Resultado de una evaluación de permiso.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermDecision {
    pub allowed: bool,
    /// Razón de la decisión (paridad TDAM): `owner`, `role_default:<role>`,
    /// `acl`, o motivo de denegación (`asset_not_available`,
    /// `not_team_member`, `visibility_restricted`, `no_permission`).
    pub reason: String,
}

// ── PermissionChecker ──

/// Checker allow-only de acceso a assets basado en entidades `entity_*`.
pub struct PermissionChecker<'a> {
    store: &'a EntityStore<'a>,
}

impl<'a> PermissionChecker<'a> {
    /// Wrap a store reference.
    pub fn new(store: &'a EntityStore<'a>) -> Self {
        Self { store }
    }

    /// ¿Es `user_id` admin activo del equipo `team_id`?
    pub fn is_admin(&self, namespace: &str, user_id: &str, team_id: &str) -> Result<bool> {
        Ok(self
            .membership(namespace, team_id, user_id)?
            .is_some_and(|m| m.role == TeamRole::Admin && m.active))
    }

    /// ¿Es `user_id` miembro activo del equipo `team_id`?
    pub fn is_member(&self, namespace: &str, user_id: &str, team_id: &str) -> Result<bool> {
        Ok(self
            .membership(namespace, team_id, user_id)?
            .is_some_and(|m| m.active))
    }

    /// ¿Puede `user_id` leer el asset? (azúcar sobre [`Self::can_access_asset`])
    pub fn can_read(&self, namespace: &str, user_id: &str, asset_id: &str) -> Result<PermDecision> {
        self.can_access_asset(namespace, user_id, asset_id, Action::Read, None)
    }

    /// ¿Puede `user_id` escribir el asset? (azúcar sobre [`Self::can_access_asset`])
    pub fn can_write(
        &self,
        namespace: &str,
        user_id: &str,
        asset_id: &str,
    ) -> Result<PermDecision> {
        self.can_access_asset(namespace, user_id, asset_id, Action::Write, None)
    }

    /// Cadena allow-only de 7 pasos: resource → owner → membership →
    /// visibility → role-default → ACL → deny.
    ///
    /// `agent_id` es opcional y solo se consulta en el paso ACL cuando la
    /// acción es de agente (subject `agent`).
    pub fn can_access_asset(
        &self,
        namespace: &str,
        user_id: &str,
        asset_id: &str,
        action: Action,
        agent_id: Option<&str>,
    ) -> Result<PermDecision> {
        let deny = |reason: &str| PermDecision {
            allowed: false,
            reason: reason.to_string(),
        };

        // 1. resource: ausente o archivado → DENY.
        let Some(asset) = self.store.entity_get(namespace, "asset", asset_id)? else {
            return Ok(deny("asset_not_available"));
        };
        if asset.status() == Some("archived") {
            return Ok(deny("asset_not_available"));
        }

        // 2. owner → ALLOW total (owner siempre puede todo sobre su asset).
        if asset.owner_user_id() == Some(user_id) {
            return Ok(PermDecision {
                allowed: true,
                reason: "owner".into(),
            });
        }

        // 3. membership: debe existir y estar activa en el team del asset.
        let team_id = asset.team_id().unwrap_or_default();
        let Some(membership) = self.membership(namespace, team_id, user_id)? else {
            return Ok(deny("not_team_member"));
        };
        if !membership.active {
            return Ok(deny("not_team_member"));
        }

        // 4. visibility.
        match asset.visibility() {
            // private: estricto — solo el owner (ya retornó ALLOW en paso 2).
            Some(Visibility::Private) => return Ok(deny("visibility_restricted")),
            // restricted: non-admin solo vía ACL explícita; admin cae a defaults.
            Some(Visibility::Restricted) => {
                if membership.role != TeamRole::Admin
                    && !self.acl_allows(
                        namespace,
                        asset_id,
                        &membership,
                        user_id,
                        action,
                        agent_id,
                    )?
                {
                    return Ok(deny("visibility_restricted"));
                }
            }
            // task: solo lectura para non-admin.
            Some(Visibility::Task) => {
                if action != Action::Read && membership.role != TeamRole::Admin {
                    return Ok(deny("visibility_restricted"));
                }
            }
            // team / agent: sigue a defaults.
            Some(Visibility::Team) | Some(Visibility::Agent) => {}
            // visibilidad ausente/desconocida → DENY.
            None => return Ok(deny("visibility_restricted")),
        }

        // 5. role-default (código, sin tabla): admin → read/write/assign/share;
        //    member/reviewer → read. Si cubre, ALLOW sin consultar ACL.
        if role_default_covers(membership.role, action) {
            return Ok(PermDecision {
                allowed: true,
                reason: format!("role_default:{}", membership.role.as_str()),
            });
        }

        // 6. ACL explícita (user / team_role / agent).
        if self.acl_allows(namespace, asset_id, &membership, user_id, action, agent_id)? {
            return Ok(PermDecision {
                allowed: true,
                reason: "acl".into(),
            });
        }

        // 7. Sin regla que permita → DENY (allow-only).
        Ok(deny("no_permission"))
    }

    /// Membership activa o `None` (status != `active` se reporta como `None`).
    fn membership(&self, namespace: &str, team_id: &str, user_id: &str) -> Result<Option<Member>> {
        let key = format!("{team_id}.{user_id}");
        let Some(entity) = self.store.entity_get(namespace, "team_member", &key)? else {
            return Ok(None);
        };
        let role = match entity.fields.get("role").and_then(FieldValue::as_str) {
            Some("admin") => TeamRole::Admin,
            Some("member") => TeamRole::Member,
            Some("reviewer") => TeamRole::Reviewer,
            _ => return Ok(None),
        };
        let active = entity.fields.get("status").and_then(FieldValue::as_str) == Some("active");
        Ok(Some(Member { role, active }))
    }

    /// ¿Existe una ACL `allow` para (asset, action) y algún subject candidato?
    fn acl_allows(
        &self,
        namespace: &str,
        asset_id: &str,
        membership: &Member,
        user_id: &str,
        action: Action,
        agent_id: Option<&str>,
    ) -> Result<bool> {
        let action_str = action.as_str();
        let subjects: [(&str, String); 3] = [
            ("user", user_id.to_string()),
            ("team_role", membership.role.as_str().to_string()),
            ("agent", agent_id.unwrap_or_default().to_string()),
        ];
        for (subject_type, subject_id) in subjects {
            if subject_type == "agent" && agent_id.is_none() {
                continue;
            }
            let key = format!("{asset_id}.{subject_type}.{subject_id}.{action_str}");
            let Some(acl) = self.store.entity_get(namespace, "acl", &key)? else {
                continue;
            };
            if acl.fields.get("effect").and_then(FieldValue::as_str) == Some("allow") {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

/// Membership resuelta del checker.
struct Member {
    role: TeamRole,
    active: bool,
}

/// Defaults por rol (código, paridad TDAM: admin → 4 acciones, resto → read).
fn role_default_covers(role: TeamRole, action: Action) -> bool {
    match role {
        TeamRole::Admin => matches!(
            action,
            Action::Read | Action::Write | Action::Assign | Action::Share
        ),
        TeamRole::Member | TeamRole::Reviewer => action == Action::Read,
    }
}

impl Action {
    /// Wire value of the action (parity with TDAM `Permission` strings).
    pub fn as_str(&self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Write => "write",
            Action::Assign => "assign",
            Action::Share => "share",
            Action::Use => "use",
        }
    }
}

impl TeamRole {
    /// Wire value of the role (parity with TDAM `TeamRole` strings).
    pub fn as_str(&self) -> &'static str {
        match self {
            TeamRole::Admin => "admin",
            TeamRole::Member => "member",
            TeamRole::Reviewer => "reviewer",
        }
    }
}

impl Visibility {
    /// Wire value of the visibility (parity with TDAM `AssetVisibility` strings).
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Private => "private",
            Visibility::Team => "team",
            Visibility::Restricted => "restricted",
            Visibility::Agent => "agent",
            Visibility::Task => "task",
        }
    }
}

impl crate::entity::Entity {
    fn team_id(&self) -> Option<&str> {
        self.fields.get("team_id").and_then(FieldValue::as_str)
    }

    fn owner_user_id(&self) -> Option<&str> {
        self.fields
            .get("owner_user_id")
            .and_then(FieldValue::as_str)
    }

    fn status(&self) -> Option<&str> {
        self.fields.get("status").and_then(FieldValue::as_str)
    }

    fn visibility(&self) -> Option<Visibility> {
        match self.fields.get("visibility").and_then(FieldValue::as_str) {
            Some("private") => Some(Visibility::Private),
            Some("team") => Some(Visibility::Team),
            Some("restricted") => Some(Visibility::Restricted),
            Some("agent") => Some(Visibility::Agent),
            Some("task") => Some(Visibility::Task),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "checker_tests.rs"]
mod tests;
