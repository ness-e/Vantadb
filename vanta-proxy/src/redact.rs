//! PII/secret redaction on egress (PRX-07).
//!
//! Scans the final pre-forward request bytes for well-known secret shapes
//! (AWS keys, generic API tokens, emails) plus operator-configured regex
//! patterns, then applies the configured [`RedactMode`].
//!
//! Anti-ReDoS design: every built-in detector is a hand-rolled linear scan
//! (no backtracking possible). Custom patterns compile with bounded
//! `size_limit`/`dfa_size_limit`, and bodies over `max_scan_bytes` fail open
//! (transparent proxy invariant). Findings never carry matched values —
//! only [`RedactKind`] labels — so logs and 422 responses can't echo secrets.

use serde::Deserialize;

use crate::error::ProxyError;

/// Default scan cap: bodies larger than this fail open (unchanged + warn).
fn default_max_scan_bytes() -> usize {
    2 * 1024 * 1024
}

/// What to do when a scan finds PII/secrets. Default is `Mask`: false
/// positives keep flowing (pre-mortem), only with values hidden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RedactMode {
    /// Reject with 422 `redaction_blocked` (kinds only, never values).
    Block,
    /// Replace matches with `[REDACTED_*]` placeholders and forward.
    #[default]
    Mask,
    /// Forward unchanged, `tracing::warn!` kinds + counts only.
    Log,
}

/// Egress redaction configuration. Disabled by default so the wire stays a
/// transparent proxy unless explicitly opted in (same invariant as
/// cache/routing/cost-enforce).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct RedactConfig {
    /// Master switch. `false` (default) → scan/apply are identity functions.
    pub enabled: bool,
    /// Action on findings. Default `Mask` (FP-safe).
    pub mode: RedactMode,
    /// Operator regex patterns (matched as `Custom`). Compiled once at
    /// construction with bounded size limits — keep them linear.
    pub patterns: Vec<String>,
    /// Bodies larger than this fail open. Default 2 MiB.
    #[serde(default = "default_max_scan_bytes")]
    pub max_scan_bytes: usize,
}

impl Default for RedactConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: RedactMode::Mask,
            patterns: Vec::new(),
            max_scan_bytes: default_max_scan_bytes(),
        }
    }
}

/// Label of a finding. Carries no matched text by construction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactKind {
    AwsKey,
    AwsSecret,
    Token,
    Email,
    Custom,
}

impl RedactKind {
    /// Stable label used in 422 responses and warn logs (never a value).
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AwsKey => "aws_key",
            Self::AwsSecret => "aws_secret",
            Self::Token => "token",
            Self::Email => "email",
            Self::Custom => "custom",
        }
    }

    /// Wire placeholder substituted in `Mask` mode.
    #[must_use]
    pub fn mask(self) -> &'static str {
        match self {
            Self::AwsKey => "[REDACTED_AWS_KEY]",
            Self::AwsSecret => "[REDACTED_SECRET]",
            Self::Token => "[REDACTED_TOKEN]",
            Self::Email => "[REDACTED_EMAIL]",
            Self::Custom => "[REDACTED_CUSTOM]",
        }
    }
}

/// One match: byte range over the scanned body plus its label.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Finding {
    pub kind: RedactKind,
    pub start: usize,
    pub end: usize,
}

/// Outcome of [`Redactor::apply`].
#[derive(Debug, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// Bytes to forward (masked, or original in log/disabled/passthrough).
    Pass(Vec<u8>),
    /// Blocked: kind labels only — never matched values.
    Block(Vec<String>),
}

/// Outcome of on-write redaction (VER-03): what gets persisted and what is
/// recorded for traceability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteRedaction {
    /// Text to persist — masked when findings exist (Block/Mask modes).
    pub text: String,
    /// Deduped kind labels of the findings (never values).
    pub kinds: Vec<String>,
    /// `true` when `text` differs from the input (mask actually applied).
    pub masked: bool,
}

/// Compiled redactor: build once from [`RedactConfig`], reuse per request.
pub struct Redactor {
    enabled: bool,
    mode: RedactMode,
    customs: Vec<regex::Regex>,
    max_scan_bytes: usize,
}

impl Redactor {
    /// Compile custom patterns with bounded size limits.
    ///
    /// # Errors
    /// [`ProxyError::Config`] when a custom pattern doesn't compile
    /// (fail-closed: the proxy refuses to start with a bad pattern).
    pub fn new(cfg: &RedactConfig) -> Result<Self, ProxyError> {
        let mut customs = Vec::with_capacity(cfg.patterns.len());
        for p in &cfg.patterns {
            let re = regex::RegexBuilder::new(p)
                .size_limit(1 << 20)
                .dfa_size_limit(1 << 20)
                .build()
                .map_err(|e| ProxyError::Config(format!("invalid redact pattern: {e}")))?;
            customs.push(re);
        }
        Ok(Self {
            enabled: cfg.enabled,
            mode: cfg.mode,
            customs,
            max_scan_bytes: cfg.max_scan_bytes,
        })
    }

    /// Scan `body`, returning findings sorted by offset. Empty when disabled,
    /// non-UTF-8, or over the scan cap (fail-open paths).
    #[must_use]
    pub fn scan(&self, body: &[u8]) -> Vec<Finding> {
        if !self.enabled || body.len() > self.max_scan_bytes {
            return Vec::new();
        }
        let Ok(text) = std::str::from_utf8(body) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        scan_aws_keys(text, &mut out);
        scan_aws_secrets(text, &mut out);
        scan_tokens(text, &mut out);
        scan_emails(text, &mut out);
        for re in &self.customs {
            for m in re.find_iter(text) {
                out.push(Finding {
                    kind: RedactKind::Custom,
                    start: m.start(),
                    end: m.end(),
                });
            }
        }
        out.sort_by_key(|f| (f.start, f.end));
        out
    }

    /// Apply the configured mode. `Block` + findings → [`ApplyOutcome::Block`]
    /// with deduped kind labels; everything else forwards bytes.
    #[must_use]
    pub fn apply(&self, body: &[u8]) -> ApplyOutcome {
        let findings = self.scan(body);
        if findings.is_empty() {
            return ApplyOutcome::Pass(body.to_vec());
        }
        let kinds = dedup_kinds(&findings);
        match self.mode {
            RedactMode::Block => {
                tracing::warn!(kinds = ?kinds, count = findings.len(), "egress redaction blocked request");
                ApplyOutcome::Block(kinds)
            }
            RedactMode::Log => {
                tracing::warn!(kinds = ?kinds, count = findings.len(), "egress redaction findings (log mode)");
                ApplyOutcome::Pass(body.to_vec())
            }
            RedactMode::Mask => ApplyOutcome::Pass(mask_body(body, &findings)),
        }
    }

    /// `true` when redaction is enabled (gates on-write provenance fields).
    #[must_use]
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// On-write application (VER-03): redact a to-be-persisted turn.
    ///
    /// Reuses the egress detectors and stays consistent with egress modes:
    /// `Block` and `Mask` mask findings (a completed turn is never dropped or
    /// blocked — which is why `Block` masks here instead of rejecting), and
    /// `Log` observes only, the same invariant as the wire. Disabled or no
    /// findings → the text passes through unchanged.
    #[must_use]
    pub fn for_write(&self, text: &str) -> WriteRedaction {
        // F-03: fail-open over the scan cap is explicit on the write path —
        // oversized turns persist unscanned (and therefore unsealed).
        if self.enabled && text.len() > self.max_scan_bytes {
            tracing::warn!(
                bytes = text.len(),
                cap = self.max_scan_bytes,
                "on-write redaction skipped: turn exceeds scan cap \
                 (persisted unscanned, no envelope)"
            );
            return WriteRedaction {
                text: text.to_string(),
                kinds: Vec::new(),
                masked: false,
            };
        }
        let findings = self.scan(text.as_bytes());
        if findings.is_empty() {
            return WriteRedaction {
                text: text.to_string(),
                kinds: Vec::new(),
                masked: false,
            };
        }
        let kinds = dedup_kinds(&findings);
        match self.mode {
            RedactMode::Block | RedactMode::Mask => {
                // Masked output is valid UTF-8 by construction (matches are
                // char-boundary slices; placeholders are ASCII). The fallback
                // is lossy (F-02): it must NEVER re-emit the clear original
                // alongside `masked: true`.
                let masked =
                    String::from_utf8_lossy(&mask_body(text.as_bytes(), &findings)).into_owned();
                WriteRedaction {
                    text: masked,
                    kinds,
                    masked: true,
                }
            }
            RedactMode::Log => WriteRedaction {
                text: text.to_string(),
                kinds,
                masked: false,
            },
        }
    }
}

/// Deduped kind labels preserving first-seen order.
fn dedup_kinds(findings: &[Finding]) -> Vec<String> {
    let mut kinds = Vec::new();
    for f in findings {
        let label = f.kind.as_str().to_string();
        if !kinds.contains(&label) {
            kinds.push(label);
        }
    }
    kinds
}

/// Substitute matches with `[REDACTED_*]` placeholders. All matches are
/// ASCII-only (built-ins) or regex char-boundary matches, so byte slicing is
/// safe; overlapping matches resolve to the earliest starter.
fn mask_body(body: &[u8], findings: &[Finding]) -> Vec<u8> {
    let mut out = Vec::with_capacity(body.len());
    let mut cursor = 0;
    for f in findings {
        if f.start < cursor || f.end > body.len() || f.start >= f.end {
            continue;
        }
        out.extend_from_slice(&body[cursor..f.start]);
        out.extend_from_slice(f.kind.mask().as_bytes());
        cursor = f.end;
    }
    out.extend_from_slice(&body[cursor..]);
    out
}

fn is_aws_key_char(b: u8) -> bool {
    b.is_ascii_uppercase() || b.is_ascii_digit()
}

/// `AKIA` + 16 uppercase/digits (AWS access key ID shape).
fn scan_aws_keys(text: &str, out: &mut Vec<Finding>) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i + 20 <= bytes.len() {
        if &bytes[i..i + 4] == b"AKIA" && bytes[i + 4..i + 20].iter().all(|&b| is_aws_key_char(b)) {
            out.push(Finding {
                kind: RedactKind::AwsKey,
                start: i,
                end: i + 20,
            });
            i += 20;
        } else {
            i += 1;
        }
    }
}

fn is_secret_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'/' || b == b'+' || b == b'='
}

/// 40-char secret-ish run shortly after an `aws_secret` marker
/// (case-insensitive).
fn scan_aws_secrets(text: &str, out: &mut Vec<Finding>) {
    let bytes = text.as_bytes();
    let needle = b"aws_secret";
    let mut i = 0;
    while i + needle.len() <= bytes.len() {
        if bytes[i..i + needle.len()].eq_ignore_ascii_case(needle) {
            let window_end = (i + needle.len() + 120).min(bytes.len());
            let mut j = i + needle.len();
            while j + 40 <= window_end {
                if bytes[j..j + 40].iter().all(|&b| is_secret_char(b)) {
                    out.push(Finding {
                        kind: RedactKind::AwsSecret,
                        start: j,
                        end: j + 40,
                    });
                    j += 40;
                } else {
                    j += 1;
                }
            }
            i += needle.len();
        } else {
            i += 1;
        }
    }
}

const TOKEN_PREFIXES: &[&str] = &[
    "sk-",
    "ghp_",
    "gho_",
    "github_pat_",
    "xoxb-",
    "xoxp-",
    "xoxa-",
    "xoxs-",
];

fn is_token_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'~' || b == b'.'
}

/// Known token prefixes + a run of token chars (suffix ≥ 10).
fn scan_tokens(text: &str, out: &mut Vec<Finding>) {
    let bytes = text.as_bytes();
    for prefix in TOKEN_PREFIXES {
        let p = prefix.as_bytes();
        let mut i = 0;
        while i + p.len() <= bytes.len() {
            if &bytes[i..i + p.len()] == p {
                let mut j = i + p.len();
                while j < bytes.len() && is_token_char(bytes[j]) {
                    j += 1;
                }
                if j - (i + p.len()) >= 10 {
                    out.push(Finding {
                        kind: RedactKind::Token,
                        start: i,
                        end: j,
                    });
                    i = j;
                } else {
                    i += 1;
                }
            } else {
                i += 1;
            }
        }
    }
}

fn is_local_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'%' | b'+' | b'-')
}

fn is_domain_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'.' || b == b'-'
}

/// `local@domain.tld` heuristic: non-empty local part, domain with a dot
/// and a ≥2-letter TLD.
fn scan_emails(text: &str, out: &mut Vec<Finding>) {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            let mut start = i;
            while start > 0 && is_local_char(bytes[start - 1]) {
                start -= 1;
            }
            let mut end = i + 1;
            while end < bytes.len() && is_domain_char(bytes[end]) {
                end += 1;
            }
            if start < i && end > i + 1 {
                let domain = &text[i + 1..end];
                if let Some(dot) = domain.rfind('.') {
                    let tld = &domain[dot + 1..];
                    if !tld.is_empty()
                        && tld.len() >= 2
                        && tld.bytes().all(|b| b.is_ascii_alphabetic())
                    {
                        out.push(Finding {
                            kind: RedactKind::Email,
                            start,
                            end,
                        });
                        i = end;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_mask() -> RedactConfig {
        RedactConfig {
            enabled: true,
            ..RedactConfig::default()
        }
    }

    #[test]
    fn default_config_is_transparent() {
        let cfg = RedactConfig::default();
        assert!(!cfg.enabled);
        assert_eq!(cfg.mode, RedactMode::Mask);
        assert_eq!(cfg.max_scan_bytes, 2 * 1024 * 1024);
    }

    #[test]
    fn toml_parses_with_defaults() {
        let cfg: RedactConfig = toml::from_str("enabled = true\n").expect("must parse");
        assert!(cfg.enabled);
        assert_eq!(cfg.mode, RedactMode::Mask);
        assert!(cfg.patterns.is_empty());
    }

    #[test]
    fn non_utf8_fails_open() {
        let r = Redactor::new(&enabled_mask()).expect("builds");
        let raw = vec![0xff, 0xfe, b'A'];
        assert!(matches!(r.apply(&raw), ApplyOutcome::Pass(_)));
    }

    #[test]
    fn overlapping_matches_resolve_to_earliest() {
        let body = b"AKIAIOSFODNN7EXAMPLE";
        let findings = vec![
            Finding {
                kind: RedactKind::AwsKey,
                start: 0,
                end: 20,
            },
            Finding {
                kind: RedactKind::Custom,
                start: 4,
                end: 10,
            },
        ];
        assert_eq!(mask_body(body, &findings), b"[REDACTED_AWS_KEY]");
    }

    // ── VER-03: on-write application ─────────────────────────────────────────

    #[test]
    fn for_write_masks_in_block_and_mask_modes() {
        for mode in [RedactMode::Block, RedactMode::Mask] {
            let r = Redactor::new(&RedactConfig {
                enabled: true,
                mode,
                ..RedactConfig::default()
            })
            .expect("builds");
            let out = r.for_write("deploy AKIAIOSFODNN7EXAMPLE now");
            assert!(
                out.masked,
                "{mode:?} must mask on write (never drop a turn)"
            );
            assert!(out.text.contains("[REDACTED_AWS_KEY]"));
            assert!(!out.text.contains("AKIAIOSFODNN7EXAMPLE"));
            assert_eq!(out.kinds, vec!["aws_key"]);
        }
    }

    #[test]
    fn for_write_log_mode_observes_without_mutating() {
        let r = Redactor::new(&RedactConfig {
            enabled: true,
            mode: RedactMode::Log,
            ..RedactConfig::default()
        })
        .expect("builds");
        let out = r.for_write("deploy AKIAIOSFODNN7EXAMPLE now");
        assert!(
            !out.masked,
            "log observes, never mutates the persisted text"
        );
        assert_eq!(out.text, "deploy AKIAIOSFODNN7EXAMPLE now");
        assert_eq!(out.kinds, vec!["aws_key"], "kinds stay traceable");
    }

    #[test]
    fn for_write_disabled_is_identity() {
        let r = Redactor::new(&RedactConfig::default()).expect("builds");
        assert!(!r.enabled());
        let out = r.for_write("deploy AKIAIOSFODNN7EXAMPLE now");
        assert!(!out.masked);
        assert!(out.kinds.is_empty());
        assert_eq!(out.text, "deploy AKIAIOSFODNN7EXAMPLE now");
    }

    #[test]
    fn for_write_masks_multibyte_text_without_losing_non_pii() {
        let r = Redactor::new(&enabled_mask()).expect("builds");
        let out = r.for_write("contacto jane.doe@example.com señor 日本語 ok");
        assert!(out.masked);
        assert!(out.text.contains("[REDACTED_EMAIL]"));
        assert!(!out.text.contains("jane.doe@example.com"));
        assert!(
            out.text.contains("señor 日本語"),
            "non-PII multibyte text must survive masking: {}",
            out.text
        );
    }

    #[test]
    fn for_write_oversize_fails_open_unmasked() {
        // F-03: over the scan cap the write path persists unscanned — declared
        // fail-open, never a partial mask.
        let cfg = RedactConfig {
            enabled: true,
            mode: RedactMode::Mask,
            max_scan_bytes: 16,
            ..RedactConfig::default()
        };
        let r = Redactor::new(&cfg).expect("builds");
        let text = format!("AKIAIOSFODNN7EXAMPLE {}", "x".repeat(32));
        let out = r.for_write(&text);
        assert!(!out.masked, "oversize fails open");
        assert!(out.kinds.is_empty());
        assert_eq!(out.text, text, "persisted unscanned (declared cap)");
    }
}
