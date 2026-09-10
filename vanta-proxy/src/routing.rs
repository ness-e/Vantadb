//! Task-aware routing por tier (PRX-06, patrón claude-code-router).
//!
//! El clasificador CC ([`crate::session::claude_code::classify_cc_request`])
//! distingue Main / Fork / Sidequery pero solo Sidequery se usaba (bypass).
//! Este módulo mapea cada kind a un tier con modelo y upstream propios, de
//! modo que turns baratos (forks, sidequeries) pueden dirigirse a un modelo
//! o upstream más económico.
//!
//! Seguro por default: [`TierRoutingConfig::default`] tiene `enabled = false`
//! (proxy transparente) y `mode = Shadow` (solo log, sin reescritura). El
//! override por key (`bypass_keys`) cubre el pre-mortem de mala
//! clasificación: una key afectada salta el routing por completo.

use bytes::Bytes;
use serde::Deserialize;
use serde_json::Value;

use crate::config::UpstreamConfig;
use crate::inject::Protocol;
use crate::session::claude_code::{classify_cc_request, CcRequestKind};

/// Tier de una request (1:1 con [`CcRequestKind`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Turn principal de conversación → modelo primario.
    Main,
    /// Request de agente forkeado → tier barato.
    Fork,
    /// Sidequery standalone (TITLE/verify_api_key) → tier barato.
    Sidequery,
}

impl Tier {
    /// Mapeo total desde el clasificador CC.
    pub fn from_cc(kind: CcRequestKind) -> Self {
        match kind {
            CcRequestKind::Main => Tier::Main,
            CcRequestKind::Fork => Tier::Fork,
            CcRequestKind::Sidequery => Tier::Sidequery,
        }
    }
}

/// Modo de aplicación del routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoutingMode {
    /// Solo log (tracing) de la decisión; body y upstreams intactos.
    /// Default: condición de stop del plan (calidad < baseline → shadow).
    #[default]
    Shadow,
    /// Aplica model override + selección de upstream.
    Enforce,
}

/// Config `[routing]` — todo default-off para compat TOML legacy.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct TierRoutingConfig {
    /// Cuando false (default), `resolve` siempre devuelve `None`.
    pub enabled: bool,
    /// Shadow (default) vs Enforce.
    pub mode: RoutingMode,
    /// Override de `model` por tier (`None` = sin reescritura).
    pub main_model: Option<String>,
    /// Override de `model` para forks (tier barato).
    pub fork_model: Option<String>,
    /// Override de `model` para sidequeries.
    pub sidequery_model: Option<String>,
    /// Índice en `upstreams_resolved()` por tier (0 = primario).
    pub main_upstream: usize,
    /// Índice upstream para forks.
    pub fork_upstream: usize,
    /// Índice upstream para sidequeries.
    pub sidequery_upstream: usize,
    /// user_ids (virtual keys) que saltan el routing (pre-mortem: mala
    /// clasificación → override por key).
    pub bypass_keys: Vec<String>,
}

impl Default for TierRoutingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: RoutingMode::Shadow,
            main_model: None,
            fork_model: None,
            sidequery_model: None,
            main_upstream: 0,
            fork_upstream: 0,
            sidequery_upstream: 0,
            bypass_keys: Vec::new(),
        }
    }
}

/// Decisión de routing pura (el caller aplica o solo loguea según `shadow`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteDecision {
    /// Tier clasificado.
    pub tier: Tier,
    /// Override de modelo a aplicar (`None` = mantener el pedido).
    pub model_override: Option<String>,
    /// Índice en la lista resolved de upstreams (ya clampeado).
    pub upstream_index: usize,
    /// True en modo shadow: loguear, no aplicar.
    pub shadow: bool,
}

impl TierRoutingConfig {
    /// Override de modelo configurado para un tier.
    pub fn model_for(&self, tier: Tier) -> Option<&str> {
        match tier {
            Tier::Main => self.main_model.as_deref(),
            Tier::Fork => self.fork_model.as_deref(),
            Tier::Sidequery => self.sidequery_model.as_deref(),
        }
    }

    /// Índice upstream configurado para un tier.
    pub fn upstream_for(&self, tier: Tier) -> usize {
        match tier {
            Tier::Main => self.main_upstream,
            Tier::Fork => self.fork_upstream,
            Tier::Sidequery => self.sidequery_upstream,
        }
    }

    /// Resuelve la decisión de routing. `None` = sin routing (disabled,
    /// bypass por key). `upstream_count` es el largo de
    /// `upstreams_resolved()`; índices fuera de rango se clampean al
    /// primario (fail-safe). Pura y total.
    pub fn resolve(
        &self,
        tier: Tier,
        user_key: &str,
        upstream_count: usize,
    ) -> Option<RouteDecision> {
        if !self.enabled {
            return None;
        }
        if self.bypass_keys.iter().any(|k| k == user_key) {
            return None;
        }
        let max = upstream_count.saturating_sub(1);
        Some(RouteDecision {
            tier,
            model_override: self.model_for(tier).map(str::to_string),
            upstream_index: self.upstream_for(tier).min(max),
            shadow: self.mode == RoutingMode::Shadow,
        })
    }
}

/// Clasifica el tier de un body. `None` salvo Anthropic parseable (el
/// clasificador CC solo vale ahí; el resto del tráfico no se rutea).
/// Falla abierto: bodies no-JSON → `None` (proxy transparente).
pub fn tier_of(protocol: Protocol, body: &[u8]) -> Option<Tier> {
    if !matches!(protocol, Protocol::Anthropic) {
        return None;
    }
    let value: Value = serde_json::from_slice(body).ok()?;
    Some(Tier::from_cc(classify_cc_request(&value)))
}

/// Reescribe el campo `model` de un body JSON. Si el body no parsea o no
/// es un objeto, devuelve los bytes originales (fail-open). Pura y total.
pub fn rewrite_model(body: &[u8], model: &str) -> Vec<u8> {
    let mut value: Value = match serde_json::from_slice(body) {
        Ok(Value::Object(map)) => Value::Object(map),
        _ => return body.to_vec(),
    };
    if let Some(obj) = value.as_object_mut() {
        obj.insert("model".to_string(), Value::String(model.to_string()));
    }
    serde_json::to_vec(&value).unwrap_or_else(|_| body.to_vec())
}

/// Routing resuelto para una request (PRX-06): body posiblemente reescrito
/// más lista de upstreams con el del tier primero (el resto conserva el orden
/// de failover PRX-02).
#[derive(Debug, Clone)]
pub struct ResolvedRoute {
    /// Body a procesar (reescrito en enforce con model override).
    pub body: Bytes,
    /// Upstreams a probar, tier-first.
    pub upstreams: Vec<UpstreamConfig>,
}

impl ResolvedRoute {
    /// Ruta passthrough: body intacto, orden de upstreams default.
    pub fn passthrough(body: Bytes, upstreams: Vec<UpstreamConfig>) -> Self {
        Self { body, upstreams }
    }

    /// Mueve `index` al frente, preservando el orden relativo del resto
    /// (failover intacto). Fuera de rango → sin cambios (fail-safe).
    pub fn prioritize(mut upstreams: Vec<UpstreamConfig>, index: usize) -> Vec<UpstreamConfig> {
        if index < upstreams.len() {
            let cfg = upstreams.remove(index);
            upstreams.insert(0, cfg);
        }
        upstreams
    }
}
