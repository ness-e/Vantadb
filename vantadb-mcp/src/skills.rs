//! MCP tool handlers for the `skill_*` tools (MEM-07).
//!
//! Six review-agent tools ported from TDAM's `skill-tools.ts` (referencia
//! 97f9465): `skill_list`, `skill_view`, `skill_create`, `skill_update`,
//! `skill_patch`, `skill_files_write`. All persistence goes through the core
//! [`vantadb::skills::SkillStore`] (D13 — the MCP layer is a thin wrapper, no
//! duplicated logic); this module only adds what the core does not own:
//! caller identity (`owner_agent`), optimistic-lock version passing, resource
//! size limits (5 MB/resource, 50 MB/skill), path validation for
//! `skill_files_write`, and the substring-replace semantics of `skill_patch`.
//!
//! Security model (FASE SECURITY): the embedded stdio server has no HTTP auth
//! layer, so the caller declares its identity via the `owner_agent` argument.
//! Ownership is enforced on every read/write; a mismatch responds *identically*
//! to a missing skill (not-found), never leaking existence — mirroring TDAM's
//! `assertTeamMatch` → `SKILL_NOT_FOUND` behavior.

use crate::config::McpConfig;
use crate::error::McpError;
use crate::validation::{error_content, serialize_content, text_content, validate_identifier};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Arc;
use vantadb::sdk::{
    SkillCreateInput, SkillListOptions, SkillPatchInput, SkillRecord, SkillUpdateInput,
};
use vantadb::skills::SkillStore;
use vantadb::storage::StorageEngine;
use vantadb::VantaError;

/// Metadata key prefix under which `skill_files_write` resources are stored.
///
/// Each resource lives as a `file:{path}` entry in the skill's `metadata`
/// map, holding a JSON record `{content, encoding, mime_type, is_executable,
/// size_bytes}`. The core `SkillRecord` has no separate file concept (MEM-06
/// models a skill as content + metadata), so resources are scoped to the
/// skill's head and versioned together with it — no core changes needed.
const FILE_META_PREFIX: &str = "file:";

/// Tool definitions for `tools/list` (MEM-07).
pub(crate) fn skill_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "skill_list",
            "description": "Lists skills owned by an agent, with optional name prefix and pagination.",
            "annotations": {
                "title": "Skill List",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "owner_agent": { "type": "string", "description": "Agent whose skills to list (scope filter)" },
                    "name_prefix": { "type": "string", "description": "Only list skills whose name starts with this prefix" },
                    "limit": { "type": "number", "description": "Max skills, default 50" },
                    "offset": { "type": "number", "description": "Number of skills to skip" }
                },
                "required": ["owner_agent"]
            }
        }),
        json!({
            "name": "skill_view",
            "description": "Reads a skill (current head or a specific version) including its resource files.",
            "annotations": {
                "title": "Skill View",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string", "description": "Skill identifier" },
                    "owner_agent": { "type": "string", "description": "Caller identity; must own the skill" },
                    "version": { "type": "number", "description": "Optional version to read; defaults to the head" }
                },
                "required": ["skill_id", "owner_agent"]
            }
        }),
        json!({
            "name": "skill_create",
            "description": "Creates a new skill (version 1). Idempotent for the same owner, name and content.",
            "annotations": {
                "title": "Skill Create",
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Skill name, unique per owner_agent" },
                    "owner_agent": { "type": "string", "description": "Owning agent identifier" },
                    "content": { "type": "string", "description": "Skill body content (e.g. SKILL.md text)" },
                    "description": { "type": "string", "description": "Optional human-readable description" },
                    "metadata": { "type": "object", "additionalProperties": {"type": "string"}, "description": "Optional metadata key-value pairs" },
                    "ttl_secs": { "type": "number", "description": "Optional TTL: skill expires after this many seconds" }
                },
                "required": ["name", "owner_agent", "content"]
            }
        }),
        json!({
            "name": "skill_update",
            "description": "Replaces a skill's head content (and optionally description), appending a new version. Requires expected_version (optimistic lock).",
            "annotations": {
                "title": "Skill Update",
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string", "description": "Skill identifier" },
                    "owner_agent": { "type": "string", "description": "Caller identity; must own the skill" },
                    "expected_version": { "type": "number", "description": "Current head version the caller believes it is editing" },
                    "content": { "type": "string", "description": "New skill content" },
                    "description": { "type": "string", "description": "Optional new description; omitted keeps the current one" }
                },
                "required": ["skill_id", "owner_agent", "expected_version", "content"]
            }
        }),
        json!({
            "name": "skill_patch",
            "description": "Substring replacement in a skill's content (TDAM-compatible). Requires expected_version; use replace_all when the string occurs more than once.",
            "annotations": {
                "title": "Skill Patch",
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string", "description": "Skill identifier" },
                    "owner_agent": { "type": "string", "description": "Caller identity; must own the skill" },
                    "expected_version": { "type": "number", "description": "Current head version the caller believes it is editing" },
                    "old_string": { "type": "string", "description": "Substring to find" },
                    "new_string": { "type": "string", "description": "Replacement substring" },
                    "replace_all": { "type": "boolean", "description": "Replace every occurrence (required when old_string occurs more than once)" }
                },
                "required": ["skill_id", "owner_agent", "expected_version", "old_string", "new_string"]
            }
        }),
        json!({
            "name": "skill_files_write",
            "description": "Writes a resource file into a skill (stored in the skill's metadata manifest). Limits: 5 MB per resource, 50 MB total per skill.",
            "annotations": {
                "title": "Skill Files Write",
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "skill_id": { "type": "string", "description": "Skill identifier" },
                    "owner_agent": { "type": "string", "description": "Caller identity; must own the skill" },
                    "expected_version": { "type": "number", "description": "Current head version the caller believes it is editing" },
                    "path": { "type": "string", "description": "Relative file path (e.g. 'scripts/tool.py'). No absolute paths, no '..' segments." },
                    "content": { "type": "string", "description": "File content (utf-8 by default, or base64 when encoding='base64')" },
                    "encoding": { "type": "string", "enum": ["utf-8", "base64"], "description": "Content encoding, default utf-8" },
                    "mime_type": { "type": "string", "description": "Optional MIME type" },
                    "is_executable": { "type": "boolean", "description": "Whether the file is executable" }
                },
                "required": ["skill_id", "owner_agent", "expected_version", "path", "content"]
            }
        }),
    ]
}

/// Dispatch a `tools/call` for one of the `skill_*` tools.
///
/// Param errors surface as JSON-RPC invalid-params; domain errors (not-found,
/// version conflict, size/path violations) surface as `error_content` results
/// the LLM can self-correct — matching the existing MCP tool pattern.
pub(crate) fn handle_skill_tool(
    name: &str,
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let store = SkillStore::new(storage);
    match name {
        "skill_list" => skill_list(args, &store, config),
        "skill_view" => skill_view(args, &store, config),
        "skill_create" => skill_create(args, &store, config),
        "skill_update" => skill_update(args, &store, config),
        "skill_patch" => skill_patch(args, &store, config),
        "skill_files_write" => skill_files_write(args, &store, config),
        _ => McpError::method_not_found(format!("Tool not found: {}", name)).into_err(),
    }
}

// ── Shared helpers ──────────────────────────────────────────────────────────

fn require_str(args: &Value, key: &str) -> Result<String, Value> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| McpError::invalid_params(format!("Missing '{key}'")).to_json())
}

fn require_u64(args: &Value, key: &str) -> Result<u64, Value> {
    args.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| McpError::invalid_params(format!("Missing '{key}'")).to_json())
}

fn opt_str(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(Value::as_str).map(str::to_string)
}

fn opt_u64(args: &Value, key: &str) -> Option<u64> {
    args.get(key).and_then(Value::as_u64)
}

fn opt_bool(args: &Value, key: &str) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(false)
}

/// Error content shared by "skill does not exist" and "skill owned by someone
/// else" — the caller must not be able to distinguish the two (no existence
/// side-channel).
fn skill_not_found() -> Value {
    error_content("Skill not found")
}

fn store_err(e: VantaError) -> Value {
    // ERR-MCP-01: structured envelope (code/retriable/hint) instead of the
    // old "Skill Error: {e}" string, so clients can branch on code.
    error_content(crate::error::McpError::from(e).to_json().to_string())
}

/// Ownership gate. A mismatch produces the same not-found response as a
/// missing skill.
fn require_owned(head: &SkillRecord, owner_agent: &str) -> Result<(), Value> {
    if head.owner_agent == owner_agent {
        Ok(())
    } else {
        Err(skill_not_found())
    }
}

/// Parse an optional `metadata` object of string values.
fn parse_metadata(args: &Value) -> Result<BTreeMap<String, String>, Value> {
    match args.get("metadata") {
        Some(Value::Null) | None => Ok(BTreeMap::new()),
        Some(v) => {
            let obj = v.as_object().ok_or_else(|| {
                McpError::invalid_params("'metadata' must be an object of string key-value pairs")
                    .to_json()
            })?;
            let mut map = BTreeMap::new();
            for (k, val) in obj {
                let s = val.as_str().ok_or_else(|| {
                    McpError::invalid_params(format!("'metadata.{k}' must be a string")).to_json()
                })?;
                map.insert(k.clone(), s.to_string());
            }
            Ok(map)
        }
    }
}

// ── skill_list ──────────────────────────────────────────────────────────────

fn skill_list(args: &Value, store: &SkillStore<'_>, config: &McpConfig) -> Result<Value, Value> {
    let owner_agent = require_str(args, "owner_agent")?;
    validate_identifier(&owner_agent, "owner_agent", config.max_namespace_length)
        .map_err(|e| e.to_json())?;
    let name_prefix = opt_str(args, "name_prefix");
    if let Some(prefix) = &name_prefix {
        validate_identifier(prefix, "name_prefix", config.max_key_length)
            .map_err(|e| e.to_json())?;
    }
    let limit = opt_u64(args, "limit")
        .unwrap_or(config.default_list_limit as u64)
        .min(config.max_list_limit as u64) as usize;
    let offset = opt_u64(args, "offset").unwrap_or(0) as usize;

    let page = match store.list(SkillListOptions {
        owner_agent: Some(owner_agent),
        name_prefix,
        limit,
        offset,
    }) {
        Ok(page) => page,
        Err(e) => return Ok(store_err(e)),
    };
    let items: Vec<Value> = page
        .items
        .iter()
        .map(|r| {
            json!({
                "skill_id": r.skill_id,
                "version": r.version,
                "name": r.name,
                "description": r.description,
            })
        })
        .collect();
    Ok(text_content(serialize_content(&json!({
        "items": items,
        "total": page.total,
    }))))
}

// ── skill_view ──────────────────────────────────────────────────────────────

fn skill_view(args: &Value, store: &SkillStore<'_>, config: &McpConfig) -> Result<Value, Value> {
    let skill_id = require_str(args, "skill_id")?;
    validate_identifier(&skill_id, "skill_id", config.max_key_length).map_err(|e| e.to_json())?;
    let owner_agent = require_str(args, "owner_agent")?;
    validate_identifier(&owner_agent, "owner_agent", config.max_namespace_length)
        .map_err(|e| e.to_json())?;

    let record = match opt_u64(args, "version") {
        Some(version) => match store.get_version(&skill_id, version) {
            Ok(Some(r)) => r,
            Ok(None) => return Ok(skill_not_found()),
            Err(e) => return Ok(store_err(e)),
        },
        None => match store.get_head(&skill_id) {
            Ok(Some(r)) => r,
            Ok(None) => return Ok(skill_not_found()),
            Err(e) => return Ok(store_err(e)),
        },
    };
    if let Err(v) = require_owned(&record, &owner_agent) {
        return Ok(v);
    }
    Ok(text_content(serialize_content(&skill_view_entry(&record))))
}

fn skill_view_entry(r: &SkillRecord) -> Value {
    json!({
        "skill_id": r.skill_id,
        "version": r.version,
        "name": r.name,
        "description": r.description,
        "content": r.content,
        "files": manifest_files(&r.metadata),
    })
}

/// Decode the `file:` metadata entries into the manifest array (sorted by path).
fn manifest_files(metadata: &BTreeMap<String, String>) -> Vec<Value> {
    let mut files: Vec<Value> = metadata
        .iter()
        .filter_map(|(key, value)| {
            let path = key.strip_prefix(FILE_META_PREFIX)?;
            let record: Value = serde_json::from_str(value).ok()?;
            Some(json!({
                "path": path,
                "content": record["content"].as_str().unwrap_or_default(),
                "encoding": record["encoding"].as_str().unwrap_or("utf-8"),
                "mime_type": record["mime_type"].as_str().unwrap_or_default(),
                "is_executable": record["is_executable"].as_bool().unwrap_or(false),
                "size_bytes": record["size_bytes"].as_u64().unwrap_or(0),
            }))
        })
        .collect();
    files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    files
}

// ── skill_create ────────────────────────────────────────────────────────────

fn skill_create(args: &Value, store: &SkillStore<'_>, config: &McpConfig) -> Result<Value, Value> {
    let owner_agent = require_str(args, "owner_agent")?;
    validate_identifier(&owner_agent, "owner_agent", config.max_namespace_length)
        .map_err(|e| e.to_json())?;
    let name = require_str(args, "name")?;
    validate_identifier(&name, "name", config.max_key_length).map_err(|e| e.to_json())?;
    let content = require_str(args, "content")?;
    if content.len() > config.max_skill_total_bytes {
        return Ok(error_content(format!(
            "Skill content exceeds maximum total size of {} bytes",
            config.max_skill_total_bytes
        )));
    }
    let description = opt_str(args, "description").unwrap_or_default();
    if description.len() > config.max_payload_length {
        return Ok(error_content(format!(
            "Skill description exceeds maximum length of {} bytes",
            config.max_payload_length
        )));
    }
    let metadata = parse_metadata(args)?;
    let ttl_secs = opt_u64(args, "ttl_secs");

    let result = match store.create(SkillCreateInput {
        name,
        description,
        content,
        owner_agent,
        metadata,
        ttl_secs,
    }) {
        Ok(result) => result,
        Err(e) => return Ok(store_err(e)),
    };
    Ok(text_content(serialize_content(&json!({
        "ok": true,
        "skill_id": result.record.skill_id,
        "version": result.record.version,
        "idempotent": result.idempotent,
    }))))
}

// ── skill_update ────────────────────────────────────────────────────────────

fn skill_update(args: &Value, store: &SkillStore<'_>, config: &McpConfig) -> Result<Value, Value> {
    let skill_id = require_str(args, "skill_id")?;
    validate_identifier(&skill_id, "skill_id", config.max_key_length).map_err(|e| e.to_json())?;
    let owner_agent = require_str(args, "owner_agent")?;
    validate_identifier(&owner_agent, "owner_agent", config.max_namespace_length)
        .map_err(|e| e.to_json())?;
    let expected_version = require_u64(args, "expected_version")?;
    let content = require_str(args, "content")?;
    if content.len() > config.max_skill_total_bytes {
        return Ok(error_content(format!(
            "Skill content exceeds maximum total size of {} bytes",
            config.max_skill_total_bytes
        )));
    }

    let head = match store.get_head(&skill_id) {
        Ok(Some(head)) => head,
        Ok(None) => return Ok(skill_not_found()),
        Err(e) => return Ok(store_err(e)),
    };
    if let Err(v) = require_owned(&head, &owner_agent) {
        return Ok(v);
    }
    let description = match opt_str(args, "description") {
        Some(desc) => desc,
        None => head.description.clone(),
    };
    if description.len() > config.max_payload_length {
        return Ok(error_content(format!(
            "Skill description exceeds maximum length of {} bytes",
            config.max_payload_length
        )));
    }

    let result = match store.update(
        &skill_id,
        expected_version,
        SkillUpdateInput {
            description,
            content,
            metadata: None,
        },
    ) {
        Ok(result) => result,
        Err(e) => return Ok(store_err(e)),
    };
    Ok(text_content(serialize_content(&json!({
        "ok": true,
        "version": result.record.version,
        "idempotent": result.idempotent,
    }))))
}

// ── skill_patch ─────────────────────────────────────────────────────────────

fn skill_patch(args: &Value, store: &SkillStore<'_>, config: &McpConfig) -> Result<Value, Value> {
    let skill_id = require_str(args, "skill_id")?;
    validate_identifier(&skill_id, "skill_id", config.max_key_length).map_err(|e| e.to_json())?;
    let owner_agent = require_str(args, "owner_agent")?;
    validate_identifier(&owner_agent, "owner_agent", config.max_namespace_length)
        .map_err(|e| e.to_json())?;
    let expected_version = require_u64(args, "expected_version")?;
    let old_string = require_str(args, "old_string")?;
    let new_string = require_str(args, "new_string")?;
    let replace_all = opt_bool(args, "replace_all");
    if old_string.is_empty() {
        return Ok(error_content("'old_string' must not be empty"));
    }

    let head = match store.get_head(&skill_id) {
        Ok(Some(head)) => head,
        Ok(None) => return Ok(skill_not_found()),
        Err(e) => return Ok(store_err(e)),
    };
    if let Err(v) = require_owned(&head, &owner_agent) {
        return Ok(v);
    }

    let occurrences = head.content.matches(&old_string).count();
    if occurrences == 0 {
        return Ok(error_content("'old_string' not found in skill content"));
    }
    if occurrences > 1 && !replace_all {
        return Ok(error_content(format!(
            "'old_string' occurs {occurrences} times; pass replace_all=true to replace every occurrence"
        )));
    }
    let new_content = if replace_all {
        head.content.replace(&old_string, &new_string)
    } else {
        head.content.replacen(&old_string, &new_string, 1)
    };

    let result = match store.patch(
        &skill_id,
        expected_version,
        SkillPatchInput {
            content: Some(new_content),
            description: None,
            metadata: None,
        },
    ) {
        Ok(result) => result,
        Err(e) => return Ok(store_err(e)),
    };
    Ok(text_content(serialize_content(&json!({
        "ok": true,
        "version": result.record.version,
        "idempotent": result.idempotent,
    }))))
}

// ── skill_files_write ───────────────────────────────────────────────────────

fn skill_files_write(
    args: &Value,
    store: &SkillStore<'_>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let skill_id = require_str(args, "skill_id")?;
    validate_identifier(&skill_id, "skill_id", config.max_key_length).map_err(|e| e.to_json())?;
    let owner_agent = require_str(args, "owner_agent")?;
    validate_identifier(&owner_agent, "owner_agent", config.max_namespace_length)
        .map_err(|e| e.to_json())?;
    let expected_version = require_u64(args, "expected_version")?;
    let path = require_str(args, "path")?;
    if path.len() > config.max_key_length {
        return Ok(error_content(format!(
            "Skill file path exceeds maximum length of {} bytes",
            config.max_key_length
        )));
    }
    if let Err(v) = assert_file_path(&path) {
        return Ok(v);
    }
    let content = require_str(args, "content")?;
    let encoding = opt_str(args, "encoding").unwrap_or_else(|| "utf-8".into());
    let mime_type = opt_str(args, "mime_type").unwrap_or_default();
    let is_executable = opt_bool(args, "is_executable");

    let size = match content_size(&content, &encoding) {
        Ok(size) => size,
        Err(v) => return Ok(v),
    };
    if size > config.max_skill_resource_bytes {
        return Ok(error_content(format!(
            "Skill file '{}' exceeds maximum size of {} bytes",
            path, config.max_skill_resource_bytes
        )));
    }

    let head = match store.get_head(&skill_id) {
        Ok(Some(head)) => head,
        Ok(None) => return Ok(skill_not_found()),
        Err(e) => return Ok(store_err(e)),
    };
    if let Err(v) = require_owned(&head, &owner_agent) {
        return Ok(v);
    }

    // Aggregate size: content + all resource files ≤ max_skill_total_bytes.
    // Replacing an existing file subtracts the old size before adding the new.
    let mut existing_total = head.content.len();
    let mut replaced_size = 0usize;
    for (key, value) in &head.metadata {
        if let Some(existing_path) = key.strip_prefix(FILE_META_PREFIX) {
            let file_size = match file_record_size(value) {
                Ok(size) => size,
                Err(v) => return Ok(v),
            };
            existing_total = existing_total.saturating_add(file_size);
            if existing_path == path {
                replaced_size = file_size;
            }
        }
    }
    let new_total = existing_total
        .saturating_add(size)
        .saturating_sub(replaced_size);
    if new_total > config.max_skill_total_bytes {
        return Ok(error_content(format!(
            "Skill '{}' exceeds maximum total size of {} bytes",
            skill_id, config.max_skill_total_bytes
        )));
    }

    let mut metadata = head.metadata.clone();
    metadata.insert(
        format!("{FILE_META_PREFIX}{path}"),
        json!({
            "content": content,
            "encoding": encoding,
            "mime_type": mime_type,
            "is_executable": is_executable,
            "size_bytes": size,
        })
        .to_string(),
    );

    let result = match store.patch(
        &skill_id,
        expected_version,
        SkillPatchInput {
            content: None,
            description: None,
            metadata: Some(metadata),
        },
    ) {
        Ok(result) => result,
        Err(e) => return Ok(store_err(e)),
    };
    Ok(text_content(serialize_content(&json!({
        "ok": true,
        "version": result.record.version,
        "idempotent": result.idempotent,
    }))))
}

/// Reject resource paths that could escape the skill's file namespace: empty,
/// absolute (`/`, `C:\`), null bytes, or `..` segments — port of TDAM
/// `assertPath`.
fn assert_file_path(path: &str) -> Result<(), Value> {
    if path.is_empty() {
        return Err(error_content("Skill file path must not be empty"));
    }
    if path.contains('\0') {
        return Err(error_content("Skill file path contains null byte"));
    }
    let normalized = path.replace('\\', "/");
    if normalized.starts_with('/') {
        return Err(error_content("Skill file path must be relative"));
    }
    let mut segments = normalized.split('/').peekable();
    if let Some(first) = segments.peek() {
        if first.ends_with(':') {
            // Windows drive-absolute (e.g. "C:/...") normalized above.
            return Err(error_content("Skill file path must be relative"));
        }
    }
    if segments.any(|seg| seg == "..") {
        return Err(error_content(
            "Skill file path must not contain '..' segments",
        ));
    }
    Ok(())
}

/// Byte size accounted for a resource. For `utf-8` this is the raw string
/// length; for `base64` the stored (encoded) length is used — it bounds the
/// decoded payload at least as tightly, and corrupt encodings are rejected at
/// the boundary so garbage never reaches the store.
fn content_size(content: &str, encoding: &str) -> Result<usize, Value> {
    match encoding {
        "utf-8" => Ok(content.len()),
        "base64" => {
            if !is_valid_base64(content) {
                return Err(error_content(
                    "Skill file content is not valid base64 for encoding='base64'",
                ));
            }
            Ok(content.len())
        }
        other => Err(error_content(format!(
            "Unsupported skill file encoding '{other}' (expected 'utf-8' or 'base64')"
        ))),
    }
}

/// Validate standard-alphabet base64 without pulling in a base64 crate:
/// length multiple of 4, padding (`=`) only at the end and at most twice.
fn is_valid_base64(input: &str) -> bool {
    if input.len() % 4 != 0 {
        return false;
    }
    let mut padding = 0u8;
    for &b in input.as_bytes() {
        match b {
            b'=' => padding += 1,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'+' | b'/' => {
                if padding > 0 {
                    return false; // '=' must only appear at the very end
                }
            }
            _ => return false,
        }
    }
    padding <= 2
}

/// Parse the `size_bytes` of an existing file record. A corrupt record means
/// the manifest cannot be accounted for — fail closed.
fn file_record_size(value: &str) -> Result<usize, Value> {
    serde_json::from_str::<Value>(value)
        .ok()
        .and_then(|v| v["size_bytes"].as_u64())
        .map(|s| s as usize)
        .ok_or_else(|| error_content("Skill file metadata is corrupt"))
}
