//! Handler for `vanta-cli mcp init` (GTM-03).
//!
//! Configures VantaDB MCP server for IDEs and AI coding clients in <30s.
//! Supported clients: `claude-code`, `cursor`, `windsurf`, `opencode`.

use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::cli_handlers::fmt::{print_json, print_success, print_warning};
use crate::error::{ChainedError, Error, Result};

/// Minimum `vanta-cli` version that supports the `brain` MCP profile (FIND-626).
const MIN_BRAIN_PROFILE_VERSION: (u64, u64, u64) = (0, 10, 0);

/// Short timeout for probing the resolved `vanta-cli --version` binary.
const VERSION_PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Polling interval while waiting for the version probe subprocess to exit.
const VERSION_PROBE_POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Result of probing a resolved CLI binary for its semantic version.
#[derive(Debug, Clone, PartialEq, Eq)]
enum BinaryVersionProbe {
    Version((u64, u64, u64)),
    TimedOut,
    Unverifiable(String),
}

/// Extracts the first `major.minor.patch` semantic version triple from `--version` output.
fn parse_semver_triple(output: &str) -> Option<(u64, u64, u64)> {
    for token in output.split_whitespace() {
        let trimmed = token.trim_start_matches(['v', 'V']);
        let core = trimmed.split(['-', '+']).next()?;
        let mut parts = core.split('.');
        let major = parts.next()?.parse::<u64>().ok();
        let minor = parts.next().and_then(|p| p.parse::<u64>().ok());
        let patch = parts.next().and_then(|p| p.parse::<u64>().ok());
        if let (Some(maj), Some(min), Some(pat), None) = (major, minor, patch, parts.next()) {
            return Some((maj, min, pat));
        }
    }
    None
}

/// Runs `<binary> --version` with a strict timeout so `mcp init` never hangs.
fn probe_binary_version(binary: &Path, timeout: Duration) -> BinaryVersionProbe {
    let mut child = match Command::new(binary)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => return BinaryVersionProbe::Unverifiable(e.to_string()),
    };

    let deadline = Instant::now() + timeout;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return BinaryVersionProbe::TimedOut;
                }
                std::thread::sleep(VERSION_PROBE_POLL_INTERVAL);
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return BinaryVersionProbe::Unverifiable(e.to_string());
            }
        }
    };

    let mut stdout = String::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_string(&mut stdout);
    }
    let mut stderr = String::new();
    if let Some(mut err) = child.stderr.take() {
        let _ = err.read_to_string(&mut stderr);
    }

    if !status.success() {
        return BinaryVersionProbe::Unverifiable(format!("exited with status {status}"));
    }

    match parse_semver_triple(&stdout).or_else(|| parse_semver_triple(&stderr)) {
        Some(version) => BinaryVersionProbe::Version(version),
        None => BinaryVersionProbe::Unverifiable("unrecognized --version output".to_string()),
    }
}

/// Evaluates whether the probed binary warrants a non-fatal warning for the requested profile.
fn brain_profile_binary_warning(
    profile: &str,
    binary: &Path,
    probe: &BinaryVersionProbe,
) -> Option<String> {
    if !profile.eq_ignore_ascii_case("brain") {
        return None;
    }

    match probe {
        BinaryVersionProbe::Version(ver) if *ver >= MIN_BRAIN_PROFILE_VERSION => None,
        BinaryVersionProbe::Version((maj, min, pat)) => Some(format!(
            "Resolved '{}' binary is v{maj}.{min}.{pat}, which predates the 'brain' MCP profile (requires vanta-cli >= 0.10.0). Older binaries ignore 'brain' or serve the legacy tool surface; upgrade vanta-cli to >= 0.10.0.",
            binary.display()
        )),
        BinaryVersionProbe::TimedOut => Some(format!(
            "Timed out probing '{} --version'; unable to verify support for the 'brain' MCP profile (requires vanta-cli >= 0.10.0).",
            binary.display()
        )),
        BinaryVersionProbe::Unverifiable(reason) => Some(format!(
            "Could not verify '{}' version ({reason}); the 'brain' MCP profile requires vanta-cli >= 0.10.0.",
            binary.display()
        )),
    }
}

/// Supported client names
pub const SUPPORTED_CLIENTS: &[&str] = &["claude-code", "cursor", "windsurf", "opencode"];

/// Resolves the default configuration file path for a supported IDE or coding client.
///
/// Clients whose config lives in the home directory (`claude-code`, `cursor`,
/// `windsurf`) return an error when neither `HOME` nor `USERPROFILE` carries a
/// value, telling the caller to pass `--config` explicitly. A CWD-relative
/// fallback used to be resolved here, and `mcp init` would write the client's
/// config into whatever directory it was launched from and report success
/// (FIND-751).
pub fn resolve_default_config_path(client: &str) -> Result<PathBuf> {
    resolve_default_config_path_from(client, home_env(), Path::new(LOCAL_MCP_CONFIG).exists())
}

/// Project-scoped Claude Code config; when present it wins over the home one.
const LOCAL_MCP_CONFIG: &str = ".mcp.json";

/// `HOME`, then `USERPROFILE`. `utils::env_chain` filters each value so `HOME=`
/// falls through to `USERPROFILE` instead of stopping the chain (FIND-751).
fn home_env() -> Option<OsString> {
    crate::utils::env_chain(std::env::var_os("HOME"), std::env::var_os("USERPROFILE"))
}

/// Takes the home value and the local-config probe instead of reading the
/// environment/filesystem so both branches are testable without `set_var` (racy
/// under nextest) or a CWD-dependent test. Filters again on purpose: the test
/// hands over the raw `Some("")`, so the helper must reject it too.
fn resolve_default_config_path_from(
    client: &str,
    home: Option<OsString>,
    local_mcp_exists: bool,
) -> Result<PathBuf> {
    let home = crate::utils::env_value(home).map(PathBuf::from);

    match client {
        "claude-code" => {
            if local_mcp_exists {
                Ok(PathBuf::from(LOCAL_MCP_CONFIG))
            } else {
                Ok(require_home(client, home)?.join(".claude.json"))
            }
        }
        "cursor" => Ok(require_home(client, home)?.join(".cursor").join("mcp.json")),
        "windsurf" => Ok(require_home(client, home)?
            .join(".codeium")
            .join("windsurf")
            .join("mcp_config.json")),
        "opencode" => Ok(PathBuf::from(".opencode").join("opencode.json")),
        other => Err(Error::InvalidInput(format!(
            "Unsupported client '{other}'. Supported clients: {}",
            SUPPORTED_CLIENTS.join(", ")
        ))),
    }
}

/// Rejects a missing home instead of yielding a path relative to the CWD.
fn require_home(client: &str, home: Option<PathBuf>) -> Result<PathBuf> {
    home.ok_or_else(|| {
        Error::Cli(ChainedError::msg(format!(
            "Cannot determine the {client} config directory: HOME and USERPROFILE are both \
             unset or empty. Pass --config <path> to choose the config file explicitly."
        )))
    })
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
    cmd_mcp_init_with_binary(
        client,
        profile,
        config_path,
        dry_run,
        json_output,
        Path::new("vanta-cli"),
        VERSION_PROBE_TIMEOUT,
    )
    .map(|_| ())
}

fn cmd_mcp_init_with_binary(
    client: &str,
    profile: &str,
    config_path: Option<&Path>,
    dry_run: bool,
    json_output: bool,
    binary: &Path,
    probe_timeout: Duration,
) -> Result<Option<String>> {
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

    let warning = if profile.eq_ignore_ascii_case("brain") {
        let probe = probe_binary_version(binary, probe_timeout);
        brain_profile_binary_warning(profile, binary, &probe)
    } else {
        None
    };
    if let Some(ref msg) = warning {
        print_warning(msg);
    }

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
        return Ok(warning);
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

    Ok(warning)
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

    // FIND-751: `var_os` returns `Some("")` for `HOME=`, and `PathBuf::from("")`
    // joined with a file name is a path relative to the CWD. The resolved path
    // is the first thing the handler computes (`target_path`), and its `?` runs
    // before any read/write, so an error here means no file is written and no
    // success is reported.
    #[test]
    fn test_empty_home_never_resolves_a_cwd_relative_config() {
        for client in ["cursor", "windsurf"] {
            let absent = resolve_default_config_path_from(client, None, false);
            assert!(
                absent.is_err(),
                "{client}: an unset HOME must not resolve a config path"
            );

            let empty = resolve_default_config_path_from(client, Some(OsString::new()), false);
            let msg = empty
                .expect_err("an empty HOME must not resolve a CWD-relative config")
                .to_string();
            assert!(
                msg.contains("--config") && msg.contains("USERPROFILE"),
                "{client}: the error must be actionable, got: {msg}"
            );
        }

        // `claude-code` keeps its project-scoped `.mcp.json` branch, so it only
        // needs a home when no local config exists.
        assert!(
            resolve_default_config_path_from("claude-code", None, false).is_err(),
            "claude-code: an unset HOME must not resolve a CWD-relative config"
        );
        assert!(
            resolve_default_config_path_from("claude-code", Some(OsString::new()), false).is_err(),
            "claude-code: an empty HOME must not resolve a CWD-relative config"
        );
        assert_eq!(
            resolve_default_config_path_from("claude-code", None, true).unwrap(),
            PathBuf::from(".mcp.json"),
            "the project-scoped config still wins without a home"
        );
    }

    #[test]
    fn test_home_value_resolves_under_home() {
        let home = if cfg!(windows) {
            PathBuf::from(r"C:\Users\tester")
        } else {
            PathBuf::from("/home/tester")
        };

        assert_eq!(
            resolve_default_config_path_from("cursor", Some(home.clone().into_os_string()), false)
                .unwrap(),
            home.join(".cursor").join("mcp.json")
        );
        assert_eq!(
            resolve_default_config_path_from(
                "windsurf",
                Some(home.clone().into_os_string()),
                false
            )
            .unwrap(),
            home.join(".codeium")
                .join("windsurf")
                .join("mcp_config.json")
        );
        assert_eq!(
            resolve_default_config_path_from(
                "claude-code",
                Some(home.clone().into_os_string()),
                false
            )
            .unwrap(),
            home.join(".claude.json")
        );
    }

    #[test]
    fn test_opencode_config_is_project_scoped_regardless_of_home() {
        let expected = PathBuf::from(".opencode").join("opencode.json");
        assert_eq!(
            resolve_default_config_path_from("opencode", None, false).unwrap(),
            expected
        );
        assert_eq!(
            resolve_default_config_path_from("opencode", Some(OsString::new()), false).unwrap(),
            expected
        );
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

    fn write_fake_version_binary(dir: &Path, name: &str, version_line: &str) -> PathBuf {
        #[cfg(windows)]
        {
            let path = dir.join(format!("{name}.cmd"));
            std::fs::write(&path, format!("@echo off\r\necho {version_line}\r\n")).unwrap();
            path
        }
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            let path = dir.join(name);
            std::fs::write(&path, format!("#!/bin/sh\necho \"{version_line}\"\n")).unwrap();
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms).unwrap();
            path
        }
    }

    fn write_slow_binary(dir: &Path, name: &str) -> PathBuf {
        #[cfg(windows)]
        {
            // Pure in-process cmd loop (no grandchild process) so `child.kill()`
            // releases every handle immediately without triggering nextest leak-fail.
            let path = dir.join(format!("{name}.cmd"));
            std::fs::write(&path, "@echo off\r\n:loop\r\ngoto loop\r\n").unwrap();
            path
        }
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            let path = dir.join(name);
            // `exec` replaces the shell process so `child.kill()` kills `sleep` directly.
            std::fs::write(&path, "#!/bin/sh\nexec sleep 2\n").unwrap();
            let mut perms = std::fs::metadata(&path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms).unwrap();
            path
        }
    }

    #[test]
    fn test_brain_profile_warns_when_binary_predates_0_10_0() {
        let tmp = tempfile::tempdir().unwrap();
        let old_bin = write_fake_version_binary(tmp.path(), "vanta_old", "vanta-cli 0.6.1");
        let target = tmp.path().join("cursor_old.json");

        let warning = cmd_mcp_init_with_binary(
            "cursor",
            "brain",
            Some(&target),
            false,
            false,
            &old_bin,
            Duration::from_secs(2),
        )
        .expect("old binary must emit a warning without failing mcp init");

        assert!(target.exists(), "config file must still be written");
        let msg = warning.expect("expected warning when binary version < 0.10.0");
        assert!(
            msg.contains("0.6.1"),
            "warning should cite detected version: {msg}"
        );
        assert!(
            msg.contains("0.10.0"),
            "warning should cite minimum version 0.10.0: {msg}"
        );
        assert!(
            msg.contains("brain"),
            "warning should reference the brain profile: {msg}"
        );

        // Boundary check: 0.9.0 also predates 0.10.0
        let v090_bin = write_fake_version_binary(tmp.path(), "vanta_090", "vanta-cli 0.9.0");
        let target_090 = tmp.path().join("cursor_090.json");
        let warn_090 = cmd_mcp_init_with_binary(
            "cursor",
            "brain",
            Some(&target_090),
            true,
            false,
            &v090_bin,
            Duration::from_secs(2),
        )
        .expect("0.9.0 binary must warn without failing");
        assert!(
            warn_090.as_deref().is_some_and(|w| w.contains("0.9.0")),
            "expected warning for 0.9.0, got: {warn_090:?}"
        );
    }

    #[test]
    fn test_brain_profile_no_warning_when_binary_meets_minimum_version() {
        let tmp = tempfile::tempdir().unwrap();
        let exact_bin = write_fake_version_binary(tmp.path(), "vanta_0100", "vanta-cli 0.10.0");
        let target = tmp.path().join("cursor_0100.json");

        let warning = cmd_mcp_init_with_binary(
            "cursor",
            "brain",
            Some(&target),
            false,
            false,
            &exact_bin,
            Duration::from_secs(2),
        )
        .expect("valid binary must succeed");
        assert!(target.exists());
        assert!(
            warning.is_none(),
            "0.10.0 binary must not emit a warning, got: {warning:?}"
        );

        let newer_bin = write_fake_version_binary(tmp.path(), "vanta_100", "vanta-cli 1.0.0");
        let warn_newer = cmd_mcp_init_with_binary(
            "claude-code",
            "brain",
            Some(&target),
            true,
            false,
            &newer_bin,
            Duration::from_secs(2),
        )
        .expect("newer binary must succeed");
        assert!(
            warn_newer.is_none(),
            "1.0.0 binary must not emit a warning, got: {warn_newer:?}"
        );
    }

    #[test]
    fn test_non_brain_profile_skips_version_warning_on_older_binary() {
        let tmp = tempfile::tempdir().unwrap();
        let old_bin = write_fake_version_binary(tmp.path(), "vanta_old_full", "vanta-cli 0.6.1");
        let target = tmp.path().join("cursor_full.json");

        let warning = cmd_mcp_init_with_binary(
            "cursor",
            "full",
            Some(&target),
            true,
            false,
            &old_bin,
            Duration::from_secs(2),
        )
        .expect("non-brain profile must succeed");
        assert!(
            warning.is_none(),
            "non-brain profile should not emit brain version warning, got: {warning:?}"
        );
    }

    #[test]
    fn test_brain_profile_degraded_warning_on_probe_timeout() {
        let tmp = tempfile::tempdir().unwrap();
        let slow_bin = write_slow_binary(tmp.path(), "vanta_slow");
        let target = tmp.path().join("cursor_slow.json");

        let start = std::time::Instant::now();
        let warning = cmd_mcp_init_with_binary(
            "cursor",
            "brain",
            Some(&target),
            true,
            false,
            &slow_bin,
            Duration::from_millis(120),
        )
        .expect("timed-out probe must degrade to a warning without failing");
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_secs(2),
            "probe timeout must bound execution time, took {elapsed:?}"
        );
        let msg = warning.expect("expected degraded warning on timeout");
        assert!(
            msg.contains("Timed out") && msg.contains("0.10.0"),
            "unexpected timeout warning: {msg}"
        );
    }
}
