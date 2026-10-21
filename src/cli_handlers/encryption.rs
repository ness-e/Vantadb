//! Encryption command handlers — rotate the master key of a database
//! (ADR-0057 W6).
//!
//! The rotation **re-wraps** the per-artefact DEKs under a new KEK: the DEKs
//! themselves are never re-derived, so **no ciphertext already on disk is
//! invalidated** (`FIND-194`). The operation is O(number of artefacts) = 4,
//! not O(data).
//!
//! The previous registry is left as `encryption.registry.bak`: copying it
//! back is a manual rollback, possible while the **old** master key still
//! exists.
//!
//! The database must be **closed**: the rotation rewrites the registry that
//! the live engine holds open.

use crate::error::{Error, Result};

/// Rotate the master key of the database at `db_path`.
///
/// The **current** key comes from the usual sources (environment → key file
/// → key command); the **new** key comes from `--new-key` / `--new-key-file`.
#[cfg(feature = "encryption")]
#[tracing::instrument]
pub fn cmd_encryption_rotate(
    db_path: &str,
    new_key: Option<&str>,
    new_key_file: Option<&str>,
    json_output: bool,
) -> Result<()> {
    use crate::cli_handlers::{print_info, print_json, print_success};
    use crate::crypto::registry::{KeyRegistry, REGISTRY_BACKUP_FILE_NAME};
    use crate::crypto::{resolve_master_key_material_strict, CryptoError};

    let data_dir = std::path::Path::new(db_path).join("data");
    let registry_path = KeyRegistry::path_in(&data_dir);
    if !registry_path.exists() {
        return Err(Error::generic_error(format!(
            "{} has no encryption registry: the database is not encrypted",
            db_path
        )));
    }

    // The current key: the same three sources the engine uses.
    let env_key = std::env::var("VANTADB_ENCRYPTION_KEY").ok();
    let key_file = std::env::var("VANTADB_ENCRYPTION_KEY_FILE").ok();
    let key_cmd = std::env::var("VANTADB_ENCRYPTION_KEY_CMD").ok();
    let current = resolve_master_key_material_strict(
        env_key.as_deref(),
        key_file.as_ref().map(std::path::Path::new),
        key_cmd.as_deref(),
    )
    .map_err(|e| match e {
        CryptoError::KeyNotSet => Error::generic_error(
            "no current encryption key: set VANTADB_ENCRYPTION_KEY, \
             VANTADB_ENCRYPTION_KEY_FILE or VANTADB_ENCRYPTION_KEY_CMD",
        ),
        other => Error::generic_error(format!("encryption key error: {other}")),
    })?;

    // The new key.
    let new_material =
        match (new_key, new_key_file) {
            (Some(hex), None) => crate::crypto::Zeroizing::new(
                crate::crypto::decode_hex(hex)
                    .map_err(|e| Error::generic_error(format!("the new key is not valid: {e}")))?,
            ),
            (None, Some(path)) => {
                let text = std::fs::read_to_string(path).map_err(|e| {
                    Error::generic_error(format!("cannot read the new key file {path}: {e}"))
                })?;
                crate::crypto::Zeroizing::new(crate::crypto::decode_hex(text.trim()).map_err(
                    |e| Error::generic_error(format!("the new key file is not valid: {e}")),
                )?)
            }
            _ => {
                return Err(Error::generic_error(
                    "a new key is required: pass --new-key <hex> or --new-key-file <path>",
                ))
            }
        };

    let mut registry = KeyRegistry::open_or_create(&data_dir, &current[..])
        .map_err(|e| Error::generic_error(format!("cannot open the encryption registry: {e}")))?;
    let artifacts = registry.artifacts();
    let previous_version = registry.key_version();
    let new_version = registry
        .rotate(&new_material[..])
        .map_err(|e| Error::generic_error(format!("the rotation failed: {e}")))?;

    let backup = data_dir.join(REGISTRY_BACKUP_FILE_NAME);
    if json_output {
        return print_json(&serde_json::json!({
            "rotated": true,
            "key_version": new_version,
            "previous_key_version": previous_version,
            "artifacts": artifacts.iter().map(|a| a.as_str()).collect::<Vec<_>>(),
            "registry": registry_path.display().to_string(),
            "backup": backup.display().to_string(),
        }));
    }
    print_success(&format!(
        "Master key rotated: key_version {previous_version} → {new_version} \
         ({} artefacts re-wrapped; no ciphertext was re-encrypted).",
        artifacts.len()
    ));
    print_info(&format!("Registry: {}", registry_path.display()));
    print_info(&format!(
        "Previous registry kept for a manual rollback: {}",
        backup.display()
    ));
    Ok(())
}

/// Rotate the master key of the database at `db_path`.
///
/// This build has **no** `encryption` feature: there is nothing to rotate.
#[cfg(not(feature = "encryption"))]
#[tracing::instrument]
pub fn cmd_encryption_rotate(
    db_path: &str,
    new_key: Option<&str>,
    new_key_file: Option<&str>,
    json_output: bool,
) -> Result<()> {
    let _ = (db_path, new_key, new_key_file, json_output);
    Err(Error::generic_error(
        "this build has no `encryption` feature: the at-rest encryption cannot be \
         rotated. Rebuild with `--features encryption`.",
    ))
}
