//! Handler for `vanta-cli mcp init` (GTM-03).
//!
//! Configures VantaDB MCP server for IDEs and AI coding clients in <30s.
//! Supported clients: `claude-code`, `cursor`, `windsurf`, `opencode`.

use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

use crate::cli_handlers::fmt::{print_json, print_success};
use crate::error::{ChainedError, Error, Result};

/// Supported client names
pub const SUPPORTED_CLIENTS: &[&str] = &["claude-code", "cursor", "windsurf", "opencode"];

/// Resolves the default configuration file path for a supported IDE or coding client.
pub fn resolve_default_config_path(client: &str) -> Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from);

    match client {
        "claude-code" => {
            let local_mcp = Path::new(".mcp.json");
            if local_mcp.exists() {
                Ok(local_mcp.to_path_buf())
            } else if let Some(h) = home {
                Ok(h.join(".claude.json"))
            } else {
                Ok(PathBuf::from(".claude.json"))
            }
        }
        "cursor" => {
            if let Some(h) = home {
                Ok(h.join(".cursor").join("mcp.json"))
            } else {
                Ok(PathBuf::from(".cursor").join("mcp.json"))
            }
        }
        "windsurf" => {
            if let Some(h) = home {
                Ok(h.join(".codeium").join("windsurf").join("mcp_config.json"))
            } else {
                Ok(PathBuf::from(".codeium")
                    .join("windsurf")
                    .join("mcp_config.json"))
            }
        }
        "opencode" => Ok(PathBuf::from(".opencode").join("opencode.json")),
        other => Err(Error::InvalidInput(format!(
            "Unsupported client '{other}'. Supported clients: {}",
            SUPPORTED_CLIENTS.join(", ")
        ))),
    }
}

/// Injects or updates the VantaDB MCP server entry in the JSON document,
/// preserving all existing top-level keys and other MCP server configurations.
pub fn inject_mcp_config(
    client: &str,
    profile: &str,
    existing_json: Option<&str>,
) -> Result<Value> {
    let mut root: Value = match existing_json {
        Some(content) if !content.trim().is_empty() => {
            serde_json::from_str(content).map_err(|e| {
                Error::InvalidInput(format!("Failed to parse existing JSON config: {e}"))
            })?
        }
        _ => Value::Object(Map::new()),
    };

    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| Error::InvalidInput("Config root must be a JSON object".to_string()))?;

    let container_key = if client == "opencode" {
        "mcp"
    } else {
        "mcpServers"
    };

    if !root_obj.contains_key(container_key) || !root_obj[container_key].is_object() {
        root_obj.insert(container_key.to_string(), Value::Object(Map::new()));
    }

    let servers_obj = root_obj
        .get_mut(container_key)
        .and_then(|v| v.as_object_mut())
        .ok_or_else(|| {
            Error::InvalidInput(format!("'{container_key}' is not a valid JSON object"))
        })?;

    let vantadb_server_config = json!({
        "command": "vanta-cli",
        "args": ["server", "--mcp", "--profile", profile],
        "env": {
            "VANTADB_MCP_PROFILE": profile
        }
    });

    servers_obj.insert("vantadb".to_string(), vantadb_server_config);

    Ok(root)
}

/// Main CLI handler for `vanta-cli mcp init`.
pub fn cmd_mcp_init(
    client: &str,
    profile: &str,
    config_path: Option<&Path>,
    dry_run: bool,
    json_output: bool,
) -> Result<()> {
    if !SUPPORTED_CLIENTS.contains(&client) {
        return Err(Error::InvalidInput(format!(
            "Unsupported client '{client}'. Supported clients: {}",
            SUPPORTED_CLIENTS.join(", ")
        )));
    }

    let target_path = match config_path {
        Some(p) => p.to_path_buf(),
        None => resolve_default_config_path(client)?,
    };

    let existing_content = if target_path.exists() {
        Some(std::fs::read_to_string(&target_path).map_err(|e| {
            Error::Cli(ChainedError::msg(format!(
                "Failed to read existing config at '{}': {e}",
                target_path.display()
            )))
        })?)
    } else {
        None
    };

    let updated_json = inject_mcp_config(client, profile, existing_content.as_deref())?;
    let formatted = serde_json::to_string_pretty(&updated_json).map_err(|e| {
        Error::Cli(ChainedError::msg(format!(
            "Failed to serialize JSON config: {e}"
        )))
    })?;

    if dry_run {
        if json_output {
            print_json(&json!({
                "dry_run": true,
                "client": client,
                "profile": profile,
                "path": target_path.to_string_lossy(),
                "config": updated_json,
            }))?;
        } else {
            println!("{formatted}");
        }
        return Ok(());
    }

    if let Some(parent) = target_path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::Cli(ChainedError::msg(format!(
                    "Failed to create config directory '{}': {e}",
                    parent.display()
                )))
            })?;
        }
    }

    std::fs::write(&target_path, formatted).map_err(|e| {
        Error::Cli(ChainedError::msg(format!(
            "Failed to write config to '{}': {e}",
            target_path.display()
        )))
    })?;

    if json_output {
        print_json(&json!({
            "success": true,
            "client": client,
            "profile": profile,
            "path": target_path.to_string_lossy(),
        }))?;
    } else {
        print_success(&format!(
            "VantaDB MCP configuration injected into '{}' (client: {}, profile: {})",
            target_path.display(),
            client,
            profile
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_client_rejected() {
        let err = cmd_mcp_init("unknown-client", "brain", None, true, false);
        assert!(err.is_err());
        let msg = err.err().unwrap().to_string();
        assert!(msg.contains("Unsupported client 'unknown-client'"));
    }

    #[test]
    fn test_inject_preserves_existing_mcp_servers_and_keys() {
        let existing = json!({
            "otherKey": "preserved_value",
            "mcpServers": {
                "fetch": {
                    "command": "uvx",
                    "args": ["mcp-server-fetch"]
                }
            }
        });
        let existing_str = serde_json::to_string(&existing).unwrap();

        let updated = inject_mcp_config("claude-code", "brain", Some(&existing_str)).unwrap();

        assert_eq!(updated["otherKey"], "preserved_value");
        assert_eq!(updated["mcpServers"]["fetch"]["command"], "uvx");

        let vanta_entry = &updated["mcpServers"]["vantadb"];
        assert_eq!(vanta_entry["command"], "vanta-cli");
        assert_eq!(
            vanta_entry["args"],
            json!(["server", "--mcp", "--profile", "brain"])
        );
        assert_eq!(vanta_entry["env"]["VANTADB_MCP_PROFILE"], "brain");
    }

    #[test]
    fn test_opencode_uses_mcp_container_key() {
        let existing = json!({
            "version": "1.0",
            "mcp": {
                "existing_tool": {
                    "command": "node"
                }
            }
        });
        let existing_str = serde_json::to_string(&existing).unwrap();

        let updated = inject_mcp_config("opencode", "brain", Some(&existing_str)).unwrap();

        assert_eq!(updated["version"], "1.0");
        assert_eq!(updated["mcp"]["existing_tool"]["command"], "node");
        assert_eq!(updated["mcp"]["vantadb"]["command"], "vanta-cli");
        assert_eq!(
            updated["mcp"]["vantadb"]["args"],
            json!(["server", "--mcp", "--profile", "brain"])
        );
        assert_eq!(
            updated["mcp"]["vantadb"]["env"]["VANTADB_MCP_PROFILE"],
            "brain"
        );
        assert!(updated.get("mcpServers").is_none());
    }

    #[test]
    fn test_all_4_clients_on_tempdir() {
        let tmp = tempfile::tempdir().unwrap();

        // 1. claude-code
        let claude_path = tmp.path().join("claude_code.json");
        cmd_mcp_init("claude-code", "brain", Some(&claude_path), false, false).unwrap();
        assert!(claude_path.exists());
        let content: Value =
            serde_json::from_str(&std::fs::read_to_string(&claude_path).unwrap()).unwrap();
        assert_eq!(content["mcpServers"]["vantadb"]["command"], "vanta-cli");
        assert_eq!(
            content["mcpServers"]["vantadb"]["args"],
            json!(["server", "--mcp", "--profile", "brain"])
        );
        assert_eq!(
            content["mcpServers"]["vantadb"]["env"]["VANTADB_MCP_PROFILE"],
            "brain"
        );

        // 2. cursor
        let cursor_path = tmp.path().join(".cursor").join("mcp.json");
        cmd_mcp_init("cursor", "brain", Some(&cursor_path), false, false).unwrap();
        assert!(cursor_path.exists());
        let content: Value =
            serde_json::from_str(&std::fs::read_to_string(&cursor_path).unwrap()).unwrap();
        assert_eq!(content["mcpServers"]["vantadb"]["command"], "vanta-cli");
        assert_eq!(
            content["mcpServers"]["vantadb"]["args"],
            json!(["server", "--mcp", "--profile", "brain"])
        );

        // 3. windsurf
        let windsurf_path = tmp
            .path()
            .join("codeium")
            .join("windsurf")
            .join("mcp_config.json");
        cmd_mcp_init("windsurf", "brain", Some(&windsurf_path), false, false).unwrap();
        assert!(windsurf_path.exists());
        let content: Value =
            serde_json::from_str(&std::fs::read_to_string(&windsurf_path).unwrap()).unwrap();
        assert_eq!(content["mcpServers"]["vantadb"]["command"], "vanta-cli");
        assert_eq!(
            content["mcpServers"]["vantadb"]["args"],
            json!(["server", "--mcp", "--profile", "brain"])
        );

        // 4. opencode
        let opencode_path = tmp.path().join(".opencode").join("opencode.json");
        cmd_mcp_init("opencode", "brain", Some(&opencode_path), false, false).unwrap();
        assert!(opencode_path.exists());
        let content: Value =
            serde_json::from_str(&std::fs::read_to_string(&opencode_path).unwrap()).unwrap();
        assert_eq!(content["mcp"]["vantadb"]["command"], "vanta-cli");
        assert_eq!(
            content["mcp"]["vantadb"]["args"],
            json!(["server", "--mcp", "--profile", "brain"])
        );
    }

    #[test]
    fn test_dry_run_does_not_create_file() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("dry_run_test.json");

        cmd_mcp_init("cursor", "brain", Some(&target), true, false).unwrap();
        assert!(!target.exists(), "Dry run must not create the target file");
    }

    #[test]
    fn test_corrupt_json_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("corrupt.json");
        std::fs::write(&target, "{ not valid json").unwrap();

        let err = cmd_mcp_init("cursor", "brain", Some(&target), false, false);
        assert!(err.is_err());
        let msg = err.err().unwrap().to_string();
        assert!(msg.contains("Failed to parse existing JSON config"));
    }
}
