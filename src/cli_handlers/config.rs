//! Configuration command handlers — show, set, init, doctor (FIND-385).

use super::fmt::*;
use crate::error::{ChainedError, Error, Result};
use console::Term;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigLayer {
    CliFlag,
    Environment,
    ProjectFile,
    UserGlobalFile,
    DefaultValue,
}

impl ConfigLayer {
    pub fn label(self) -> &'static str {
        match self {
            ConfigLayer::CliFlag => "CLI Flag",
            ConfigLayer::Environment => "Environment (VANTADB_*)",
            ConfigLayer::ProjectFile => "Project File (./vanta.toml)",
            ConfigLayer::UserGlobalFile => "User File (~/.vanta/config.toml)",
            ConfigLayer::DefaultValue => "Default",
        }
    }
}

struct ConfigItem {
    key: &'static str,
    env_var: &'static str,
    value: String,
    layer: ConfigLayer,
    is_secret: bool,
}

fn user_config_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".vanta").join("config.toml"))
}

fn project_config_path() -> PathBuf {
    PathBuf::from("vanta.toml")
}

fn read_toml_map(path: &Path) -> BTreeMap<String, toml::Value> {
    if !path.exists() {
        return BTreeMap::new();
    }
    match std::fs::read_to_string(path) {
        Ok(content) => match toml::from_str::<toml::Table>(&content) {
            Ok(table) => {
                let mut map = BTreeMap::new();
                flatten_toml_table("", &table, &mut map);
                map
            }
            Err(_) => BTreeMap::new(),
        },
        Err(_) => BTreeMap::new(),
    }
}

fn flatten_toml_table(prefix: &str, table: &toml::Table, map: &mut BTreeMap<String, toml::Value>) {
    for (k, v) in table {
        let full_key = if prefix.is_empty() {
            k.clone()
        } else {
            format!("{prefix}.{k}")
        };
        match v {
            toml::Value::Table(sub) => flatten_toml_table(&full_key, sub, map),
            other => {
                map.insert(full_key.clone(), other.clone());
                // Also insert unqualified leaf key for easy flat lookups
                map.insert(k.clone(), other.clone());
            }
        }
    }
}

fn normalize_model_path(val: &str) -> String {
    let trimmed = val.trim();
    if !trimmed.is_empty() && !trimmed.contains('/') && !trimmed.contains('\\') {
        format!("embeddings/models/{trimmed}/onnx")
    } else {
        trimmed.to_string()
    }
}

fn resolve_item(
    key: &'static str,
    env_var: &'static str,
    default_val: &str,
    is_secret: bool,
    cli_override: Option<&str>,
    project_map: &BTreeMap<String, toml::Value>,
    user_map: &BTreeMap<String, toml::Value>,
) -> ConfigItem {
    let (raw_val, layer) = if let Some(cli) = cli_override {
        (cli.to_string(), ConfigLayer::CliFlag)
    } else if let Ok(env_val) = std::env::var(env_var) {
        if !env_val.is_empty() {
            (env_val, ConfigLayer::Environment)
        } else if let Some(proj_val) = project_map.get(key) {
            (toml_val_to_str(proj_val), ConfigLayer::ProjectFile)
        } else if let Some(user_val) = user_map.get(key) {
            (toml_val_to_str(user_val), ConfigLayer::UserGlobalFile)
        } else {
            (default_val.to_string(), ConfigLayer::DefaultValue)
        }
    } else if let Some(proj_val) = project_map.get(key) {
        (toml_val_to_str(proj_val), ConfigLayer::ProjectFile)
    } else if let Some(user_val) = user_map.get(key) {
        (toml_val_to_str(user_val), ConfigLayer::UserGlobalFile)
    } else {
        (default_val.to_string(), ConfigLayer::DefaultValue)
    };

    let value = if key == "local_model_path" {
        normalize_model_path(&raw_val)
    } else {
        raw_val
    };

    ConfigItem {
        key,
        env_var,
        value,
        layer,
        is_secret,
    }
}

fn toml_val_to_str(v: &toml::Value) -> String {
    match v {
        toml::Value::String(s) => s.clone(),
        toml::Value::Integer(i) => i.to_string(),
        toml::Value::Float(f) => f.to_string(),
        toml::Value::Boolean(b) => b.to_string(),
        other => other.to_string(),
    }
}

/// Show effective configuration with layers and masked secrets.
pub fn cmd_config_show(db_path: &str, reveal_secrets: bool, json_output: bool) -> Result<()> {
    let proj_path = project_config_path();
    let project_map = read_toml_map(&proj_path);
    let user_path = user_config_path();
    let user_map = user_path.as_deref().map(read_toml_map).unwrap_or_default();

    let cli_db = if db_path != "./db" && db_path != "vantadb_data" {
        Some(db_path)
    } else {
        None
    };

    let items = vec![
        resolve_item(
            "storage_path",
            "VANTADB_STORAGE_PATH",
            "vantadb_data",
            false,
            cli_db,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "host",
            "VANTADB_HOST",
            "127.0.0.1",
            false,
            None,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "port",
            "VANTADB_PORT",
            "8080",
            false,
            None,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "embedding_provider",
            "VANTADB_EMBEDDING_PROVIDER",
            "ollama",
            false,
            None,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "strict_embeddings",
            "VANTADB_STRICT_EMBEDDINGS",
            "true",
            false,
            None,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "local_model_path",
            "VANTADB_LOCAL_MODEL",
            "embeddings/models/multilingual-e5-small/onnx",
            false,
            None,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "sync_mode",
            "VANTADB_SYNC_MODE",
            "periodic",
            false,
            None,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "wal_shards",
            "VANTADB_WAL_SHARDS",
            "4",
            false,
            None,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "self_healing",
            "VANTADB_SELF_HEALING",
            "true",
            false,
            None,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "self_healing_quiet_secs",
            "VANTADB_SELF_HEALING_QUIET_SECS",
            "300",
            false,
            None,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "api_key",
            "VANTADB_API_KEY",
            "",
            true,
            None,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "alt_api_key",
            "VANTADB_ALT_API_KEY",
            "",
            true,
            None,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "jwt_secret",
            "VANTADB_JWT_SECRET",
            "",
            true,
            None,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "openai_api_key",
            "VANTADB_OPENAI_API_KEY",
            "",
            true,
            None,
            &project_map,
            &user_map,
        ),
        resolve_item(
            "encryption_key",
            "VANTADB_ENCRYPTION_KEY",
            "",
            true,
            None,
            &project_map,
            &user_map,
        ),
    ];

    if json_output {
        let mut map = serde_json::Map::new();
        for item in &items {
            let display_val = if item.is_secret && !reveal_secrets && !item.value.is_empty() {
                "******".to_string()
            } else {
                item.value.clone()
            };
            map.insert(
                item.key.to_string(),
                json!({
                    "value": display_val,
                    "layer": item.layer.label(),
                    "env_var": item.env_var,
                    "is_secret": item.is_secret,
                }),
            );
        }
        return print_json(&serde_json::Value::Object(map));
    }

    let term = Term::stdout();
    let _ = term.write_line("");
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to(
            "╔═══════════════════════════════════════════════════════════════════════════╗"
        )
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to(
            "║               VantaDB Effective Configuration (5 Layers)                  ║"
        )
    ));
    let _ = term.write_line(&format!(
        "{}",
        header_style().apply_to(
            "╚═══════════════════════════════════════════════════════════════════════════╝"
        )
    ));
    let _ = term.write_line("");

    for item in &items {
        let display_val = if item.is_secret && !reveal_secrets && !item.value.is_empty() {
            "******".to_string()
        } else if item.value.is_empty() {
            "(not set)".to_string()
        } else {
            item.value.clone()
        };

        let layer_styled = match item.layer {
            ConfigLayer::CliFlag => warning_style().apply_to(item.layer.label()),
            ConfigLayer::Environment => success_style().apply_to(item.layer.label()),
            ConfigLayer::ProjectFile => info_style().apply_to(item.layer.label()),
            ConfigLayer::UserGlobalFile => info_style().apply_to(item.layer.label()),
            ConfigLayer::DefaultValue => header_style().apply_to(item.layer.label()),
        };

        let _ = term.write_line(&format!(
            "  {:<26} = {:<32} [{}]",
            item.key, display_val, layer_styled
        ));
    }

    let _ = term.write_line("");
    print_info("Use `vanta config set <key> <value>` to edit project configuration.");
    if !reveal_secrets {
        print_info("Secrets are masked. Use `--reveal-secrets` to view full values.");
    }
    let _ = term.write_line("");

    Ok(())
}

/// Set a configuration value in local or global configuration file.
pub fn cmd_config_set(key: &str, value: &str, global: bool, json_output: bool) -> Result<()> {
    let target_path = if global {
        user_config_path().ok_or_else(|| {
            Error::Cli(ChainedError::msg(
                "Could not determine user home directory for global config",
            ))
        })?
    } else {
        project_config_path()
    };

    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::Cli(ChainedError::msg(format!(
                    "Failed to create directory {}: {e}",
                    parent.display()
                )))
            })?;
        }
    }

    let mut table: toml::Table = if target_path.exists() {
        let content = std::fs::read_to_string(&target_path).map_err(|e| {
            Error::Cli(ChainedError::msg(format!(
                "Failed to read config file {}: {e}",
                target_path.display()
            )))
        })?;
        toml::from_str(&content).unwrap_or_default()
    } else {
        toml::Table::new()
    };

    let (target_key, target_value) = match key {
        "model"
        | "embeddings.model"
        | "local_model"
        | "embeddings.local_model"
        | "local_model_path"
        | "embeddings.local_model_path" => ("local_model_path", normalize_model_path(value)),
        _ => (key, value.to_string()),
    };

    let parsed_val = if target_value == "true" {
        toml::Value::Boolean(true)
    } else if target_value == "false" {
        toml::Value::Boolean(false)
    } else if let Ok(i) = target_value.parse::<i64>() {
        toml::Value::Integer(i)
    } else if let Ok(f) = target_value.parse::<f64>() {
        toml::Value::Float(f)
    } else {
        toml::Value::String(target_value.clone())
    };

    table.insert(target_key.to_string(), parsed_val);

    let serialized = toml::to_string_pretty(&table).map_err(|e| {
        Error::Cli(ChainedError::msg(format!(
            "Failed to serialize TOML config: {e}"
        )))
    })?;

    std::fs::write(&target_path, serialized).map_err(|e| {
        Error::Cli(ChainedError::msg(format!(
            "Failed to write config file {}: {e}",
            target_path.display()
        )))
    })?;

    if json_output {
        print_json(&json!({
            "command": "config_set",
            "key": target_key,
            "value": target_value,
            "file": target_path.display().to_string(),
            "global": global,
            "success": true
        }))?;
    } else {
        print_success(&format!(
            "Set `{target_key} = {target_value}` in {}",
            target_path.display()
        ));
    }

    Ok(())
}

/// Initialize an annotated template vanta.toml file.
pub fn cmd_config_init(force: bool, json_output: bool) -> Result<()> {
    let path = project_config_path();
    if path.exists() && !force {
        return Err(Error::Cli(ChainedError::msg(
            "vanta.toml already exists in current directory. Use --force to overwrite.",
        )));
    }

    let template = r#"# VantaDB Unified Configuration File (vanta.toml)
# Precedence: CLI flags > Environment variables > ./vanta.toml > ~/.vanta/config.toml > Defaults

[storage]
# Directory path for persistent storage
storage_path = "vantadb_data"
# Number of WAL shards (default: 4)
wal_shards = 4
# Synchronization mode: periodic, durable, immediate
sync_mode = "periodic"
# Memory limit for engine (e.g. "2GB", "500MB", or bytes integer)
# memory_limit = "2GB"

[server]
# Bind address and port for HTTP / MCP server
host = "127.0.0.1"
port = 8080
# Require authentication: refuse to serve requests without an API key
require_auth = false
# Rate limit in requests per minute per IP
rate_limit_rpm = 600

[embeddings]
# Embedding provider: ollama, openai, or local (ONNX)
embedding_provider = "ollama"
# Strict embeddings policy: fail closed instead of using synthetic fallback vectors
strict_embeddings = true
# Local ONNX model path (used when embedding_provider = "local")
local_model_path = "embeddings/models/multilingual-e5-small/onnx"

[maintenance]
# Automated background self-healing maintenance (vacuum during quiet periods)
self_healing = true
# Minimum idle duration in seconds before self-healing executes (default: 300s = 5m)
self_healing_quiet_secs = 300
"#;

    std::fs::write(&path, template).map_err(|e| {
        Error::Cli(ChainedError::msg(format!(
            "Failed to write template to {}: {e}",
            path.display()
        )))
    })?;

    if json_output {
        print_json(&json!({
            "command": "config_init",
            "file": path.display().to_string(),
            "success": true
        }))?;
    } else {
        print_success(&format!(
            "Initialized template configuration in {}",
            path.display()
        ));
    }

    Ok(())
}

/// Run comprehensive pre-flight health and configuration checks.
pub fn cmd_config_doctor(db_path: &str, json_output: bool) -> Result<()> {
    let mut checks = Vec::new();

    // 1. Storage directory check
    let storage_dir = Path::new(db_path);
    let storage_status = if storage_dir.exists() {
        let probe_file = storage_dir.join(".doctor_probe");
        match std::fs::write(&probe_file, b"probe") {
            Ok(_) => {
                let _ = std::fs::remove_file(probe_file);
                (
                    true,
                    format!("Directory exists and is writable ({db_path})"),
                )
            }
            Err(e) => (false, format!("Directory exists but write failed: {e}")),
        }
    } else {
        match std::fs::create_dir_all(storage_dir) {
            Ok(_) => {
                let probe_file = storage_dir.join(".doctor_probe");
                let _ = std::fs::write(&probe_file, b"probe");
                let _ = std::fs::remove_file(probe_file);
                (true, format!("Directory created and writable ({db_path})"))
            }
            Err(e) => (
                false,
                format!("Directory missing and cannot be created: {e}"),
            ),
        }
    };
    checks.push(("Storage Directory", storage_status.0, storage_status.1));

    // 2. Lock check
    let lock_file = storage_dir.join("vanta.lock");
    let lock_status = if lock_file.exists() {
        let metadata = crate::storage::lock::read_lock_metadata(&lock_file);
        if let Some(meta) = metadata {
            if crate::storage::lock::is_process_alive(meta.pid) {
                (
                    false,
                    format!("Locked by active PID {} ({})", meta.pid, meta.process_name),
                )
            } else {
                (
                    true,
                    format!(
                        "Orphaned lock detected from dead PID {} (auto-recoverable)",
                        meta.pid
                    ),
                )
            }
        } else {
            (
                true,
                "Lock file exists without metadata (legacy lock)".to_string(),
            )
        }
    } else {
        (
            true,
            "Storage lock is free (clean shutdown state)".to_string(),
        )
    };
    checks.push(("Storage Lock", lock_status.0, lock_status.1));

    // 3. Strict embeddings & model physical integrity check
    let strict = std::env::var("VANTADB_STRICT_EMBEDDINGS")
        .map(|v| v != "false" && v != "0")
        .unwrap_or(true);
    let health = crate::embedding_health::embedding_health();
    let (emb_ok, emb_desc) = match health.fallback {
        Some(true) => {
            let detail = health.detail.unwrap_or_else(|| "unusable".to_string());
            (
                !strict,
                format!(
                    "Provider: {} (DEGRADED: {}) | Strict mode: {} (fail-closed)",
                    health.provider, detail, strict
                ),
            )
        }
        Some(false) => {
            let model_info = health.model_dir.unwrap_or_else(|| "default".to_string());
            (
                true,
                format!(
                    "Provider: {} | Verified at '{}' | Strict mode: {}",
                    health.provider, model_info, strict
                ),
            )
        }
        None => {
            let detail = health.detail.unwrap_or_else(|| "configured".to_string());
            (
                true,
                format!(
                    "Provider: {} ({}) | Strict mode: {}",
                    health.provider, detail, strict
                ),
            )
        }
    };
    checks.push(("Embeddings Health", emb_ok, emb_desc));

    // 4. Server Port check
    let port = std::env::var("VANTADB_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);
    let port_status = match std::net::TcpListener::bind(("127.0.0.1", port)) {
        Ok(_) => (true, format!("Port {port} is available for binding")),
        Err(e) => (false, format!("Port {port} is unavailable: {e}")),
    };
    checks.push(("Network Port", port_status.0, port_status.1));

    // 5. Hardware Capabilities
    let hw = crate::hardware::HardwareCapabilities::global();
    let ram_gb = hw.total_memory / (1024 * 1024 * 1024);
    let hw_desc = format!(
        "Profile: {:?} | Memory: {} GB | Cores: {} | Instructions: {:?}",
        hw.profile, ram_gb, hw.logical_cores, hw.instructions
    );
    checks.push(("Hardware Acceleration", true, hw_desc));

    let all_ok = checks.iter().all(|c| c.1);

    if json_output {
        let json_checks: Vec<_> = checks
            .into_iter()
            .map(|(name, ok, desc)| {
                json!({
                    "check": name,
                    "ok": ok,
                    "description": desc,
                })
            })
            .collect();
        print_json(&json!({
            "command": "config_doctor",
            "healthy": all_ok,
            "checks": json_checks
        }))?;
    } else {
        let term = Term::stdout();
        let _ = term.write_line("");
        let _ = term.write_line(&format!(
            "{}",
            header_style().apply_to(
                "╔═══════════════════════════════════════════════════════════════════════════╗"
            )
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style().apply_to(
                "║                   VantaDB System Doctor Diagnostics                       ║"
            )
        ));
        let _ = term.write_line(&format!(
            "{}",
            header_style().apply_to(
                "╚═══════════════════════════════════════════════════════════════════════════╝"
            )
        ));
        let _ = term.write_line("");

        for (name, ok, desc) in &checks {
            if *ok {
                let _ = term.write_line(&format!(
                    "  {} {:<24} {}",
                    success_style().apply_to("✓"),
                    name,
                    desc
                ));
            } else {
                let _ = term.write_line(&format!(
                    "  {} {:<24} {}",
                    error_style().apply_to("✗"),
                    name,
                    error_style().apply_to(desc)
                ));
            }
        }
        let _ = term.write_line("");
        if all_ok {
            print_success("All system pre-flight checks passed! System is healthy.");
        } else {
            print_error("Some diagnostic checks reported warnings or issues. Review above.");
        }
        let _ = term.write_line("");
    }

    Ok(())
}
