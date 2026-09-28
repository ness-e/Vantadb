//! Deterministic multi-signal entity linking (Fellegi-Sunter + embeddings).
//!
//! Implements the deterministic slice of MGR-05 / WIRE-05: probabilistic record
//! linkage without an LLM judge. Each candidate carries a small set of
//! comparison signals (normalized text identifiers and/or embedding vectors);
//! a pair's match weight is the classic Fellegi-Sunter log-likelihood ratio
//! sum:
//!
//! ```text
//! W = prior_bits + Σ_i log2(m_i / u_i)            (signal i agrees)
//!                 + Σ_i log2((1 - m_i) / (1 - u_i)) (signal i disagrees)
//!                 + 0                                (signal absent on either side)
//! Pr(match | observations) = 2^W / (1 + 2^W)
//! ```
//!
//! Decision rule (Fellegi & Sunter, 1969): `W >= auto_link_bits` links,
//! `W < review_bits` separates, and the band in between is flagged for review.
//! The default configuration is deliberately conservative: `auto_link_bits`
//! is set above the strongest single-signal agreement weight, so an automatic
//! merge requires **at least two independent signals** to agree.
//!
//! Determinism contract: identical inputs produce identical scores and
//! identical reports — no `HashMap` iteration order leaks into results, no
//! floating-point reordering, no time/env inputs. [`link_entities`] never
//! mutates stored data: clusters are a derived view (reversible).
//!
//! Sources: Splink's *The Fellegi-Sunter Model* (UK Ministry of Justice)
//! `https://moj-analytical-services.github.io/splink/topic_guides/theory/fellegi_sunter.html`
//! and Wikipedia *Record linkage* (Fellegi & Sunter, JASA 1969)
//! `https://en.wikipedia.org/wiki/Record_linkage`.

use crate::error::{Error, Result};
use crate::node::FieldValue;
use std::collections::{BTreeMap, HashMap};

// ── Signals ──

/// Kind of comparison signal used by the matcher.
///
/// Comparison uses one signal per kind (the first occurrence in the input
/// order wins); kinds not present on both sides are neutral (weight 0).
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SignalKind {
    /// Person/entity name; compared after trim + lowercase + whitespace collapse.
    Name,
    /// E-mail address; compared after trim + lowercase.
    Email,
    /// Phone number; compared on ASCII digits only (separators ignored).
    Phone,
    /// Dense embedding; compared by cosine similarity against a configurable
    /// threshold (default `0.90`).
    Embedding,
}

impl SignalKind {
    /// Stable wire value of the kind (for reports/logs).
    pub fn as_str(&self) -> &'static str {
        match self {
            SignalKind::Name => "name",
            SignalKind::Email => "email",
            SignalKind::Phone => "phone",
            SignalKind::Embedding => "embedding",
        }
    }
}

/// Value carried by a [`LinkSignal`].
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SignalValue {
    /// Textual identifier (name, e-mail, phone).
    Text(String),
    /// Dense embedding vector.
    Embedding(Vec<f32>),
}

/// One comparison signal of a candidate.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkSignal {
    /// Signal kind (selects the comparator and the m/u weights).
    pub kind: SignalKind,
    /// Signal value.
    pub value: SignalValue,
}

impl LinkSignal {
    /// Build a textual signal (use for [`SignalKind::Name`], `Email` or `Phone`).
    pub fn text(kind: SignalKind, value: impl Into<String>) -> Self {
        Self {
            kind,
            value: SignalValue::Text(value.into()),
        }
    }

    /// Build an embedding signal ([`SignalKind::Embedding`]).
    pub fn embedding(value: Vec<f32>) -> Self {
        Self {
            kind: SignalKind::Embedding,
            value: SignalValue::Embedding(value),
        }
    }
}

// ── Weights & configuration ──

/// Fellegi-Sunter m/u probabilities for one signal kind.
///
/// `m` = P(signal agrees | records match); `u` = P(signal agrees | records do
/// not match, by chance). Constructed through [`SignalWeight::new`], which
/// enforces `0 < u < m < 1` (validate at the boundary, trust internally).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignalWeight {
    /// Agreement probability among true matches.
    pub m: f32,
    /// Coincidental agreement probability among non-matches.
    pub u: f32,
}

impl SignalWeight {
    /// Validate and build a weight pair.
    ///
    /// # Errors
    /// [`Error::InvalidInput`] unless `m` and `u` are finite with
    /// `0 < u < m < 1`.
    pub fn new(m: f32, u: f32) -> Result<Self> {
        let valid =
            m.is_finite() && u.is_finite() && m > 0.0 && m < 1.0 && u > 0.0 && u < 1.0 && m > u;
        if !valid {
            return Err(Error::InvalidInput(format!(
                "SignalWeight requires finite 0 < u < m < 1 (got m={m}, u={u})"
            )));
        }
        Ok(Self { m, u })
    }

    /// Agreement linkage weight `log2(m/u)`.
    pub(crate) fn agree_bits(&self) -> f32 {
        (self.m / self.u).log2()
    }

    /// Disagreement linkage weight `log2((1-m)/(1-u))`.
    pub(crate) fn disagree_bits(&self) -> f32 {
        ((1.0 - self.m) / (1.0 - self.u)).log2()
    }
}

fn default_weight(kind: SignalKind) -> SignalWeight {
    // Conservative starting points (overridable per kind): `m` reflects data
    // reliability, `u` reflects identifier cardinality/coincidence. Invariant
    // 0 < u < m < 1 holds for every entry (mirrors the Splink guidance that
    // estimated values are dataset-dependent — EM estimation is out of scope).
    match kind {
        SignalKind::Name => SignalWeight { m: 0.95, u: 0.001 },
        SignalKind::Email => SignalWeight { m: 0.99, u: 0.0005 },
        SignalKind::Phone => SignalWeight { m: 0.95, u: 0.0005 },
        SignalKind::Embedding => SignalWeight { m: 0.90, u: 0.05 },
    }
}

/// Deterministic Fellegi-Sunter configuration.
///
/// Build from [`LinkConfig::default`] and override with the `with_*` builders
/// (the struct cannot be constructed literally outside this module because
/// `overrides` is private — that keeps every custom [`SignalWeight`] validated).
///
/// Defaults: `auto_link_bits = 12.0` (posterior ≈ 0.99976, above the strongest
/// single-signal agreement weight ≈ 10.95 → requires ≥ 2 agreeing signals),
/// `review_bits = 4.0`, `prior_bits = 0.0`, `embedding_threshold = 0.90`.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkConfig {
    /// Upper threshold: pairs at or above are auto-linked.
    pub auto_link_bits: f32,
    /// Lower threshold: pairs below are distinct; the band in between is reviewed.
    pub review_bits: f32,
    /// Prior match-weight term `log2(λ/(1-λ))`; 0.0 = neutral prior (pairs are
    /// already candidates, e.g. after blocking/top-K; λ estimation is out of scope).
    pub prior_bits: f32,
    /// Cosine threshold for [`SignalKind::Embedding`] agreement.
    pub embedding_threshold: f32,
    overrides: Vec<(SignalKind, SignalWeight)>,
}

impl Default for LinkConfig {
    fn default() -> Self {
        Self {
            auto_link_bits: 12.0,
            review_bits: 4.0,
            prior_bits: 0.0,
            embedding_threshold: 0.90,
            overrides: Vec::new(),
        }
    }
}

impl LinkConfig {
    /// Override the m/u weights of one signal kind (replaces any prior override).
    pub fn with_signal_weight(mut self, kind: SignalKind, weight: SignalWeight) -> Self {
        self.overrides.retain(|(k, _)| *k != kind);
        self.overrides.push((kind, weight));
        self.overrides.sort_by_key(|(k, _)| *k);
        self
    }

    /// Override the auto-link (upper) threshold.
    pub fn with_auto_link_bits(mut self, bits: f32) -> Self {
        self.auto_link_bits = bits;
        self
    }

    /// Override the review (lower) threshold.
    pub fn with_review_bits(mut self, bits: f32) -> Self {
        self.review_bits = bits;
        self
    }

    /// Override the prior match-weight term.
    pub fn with_prior_bits(mut self, bits: f32) -> Self {
        self.prior_bits = bits;
        self
    }

    /// Override the embedding cosine agreement threshold.
    pub fn with_embedding_threshold(mut self, threshold: f32) -> Self {
        self.embedding_threshold = threshold;
        self
    }

    /// Effective weight table for `kind` (override or conservative default).
    pub fn weight(&self, kind: SignalKind) -> SignalWeight {
        self.overrides
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, w)| *w)
            .unwrap_or_else(|| default_weight(kind))
    }
}

// ── Candidates + outcomes ──

/// One candidate record for matching (entity, mention, memory record, ...).
#[derive(Debug, Clone, PartialEq)]
pub struct LinkEntity {
    /// Caller-defined identifier, unique within one [`link_entities`] call.
    pub id: String,
    /// Comparison signals (one per kind is used; extra signals are ignored).
    pub signals: Vec<LinkSignal>,
}

impl LinkEntity {
    /// Build a candidate from its id and signals.
    pub fn new(id: impl Into<String>, signals: Vec<LinkSignal>) -> Self {
        Self {
            id: id.into(),
            signals,
        }
    }
}

/// Outcome of comparing one signal kind between two candidates.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalOutcome {
    /// Both sides present and the comparator agrees.
    Agree,
    /// Both sides present and the comparator disagrees.
    Disagree,
    /// At least one side is absent (neutral, weight 0).
    Missing,
}

/// Per-signal provenance for one pair comparison.
#[derive(Debug, Clone, PartialEq)]
pub struct SignalContribution {
    /// Compared signal kind.
    pub kind: SignalKind,
    /// Agreement outcome.
    pub outcome: SignalOutcome,
    /// Additive linkage weight contributed by this signal.
    pub weight: f32,
    /// Deterministic human-readable evidence (no raw PII values).
    pub evidence: String,
}

/// Linkage verdict for one candidate pair.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkVerdict {
    /// Pair at or above the upper threshold (or forced by a [`ManualLink`]).
    AutoLink,
    /// Pair inside the review band — flagged, not merged.
    Review,
    /// Pair below the lower threshold — not reported in [`LinkReport::decisions`].
    Distinct,
}

/// One scored pair with full provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct LinkDecision {
    /// Lexicographically smaller member id.
    pub left: String,
    /// Lexicographically larger member id.
    pub right: String,
    /// Total match weight in bits (prior + Σ signal contributions).
    pub bits: f32,
    /// Posterior match probability `2^bits / (1 + 2^bits)`.
    pub posterior: f32,
    /// Verdict (manual pairs are always [`LinkVerdict::AutoLink`]).
    pub verdict: LinkVerdict,
    /// `true` when the pair was forced through [`ManualLink`] (`mark_duplicate`).
    pub manual: bool,
    /// Per-signal provenance, in fixed kind order.
    pub contributions: Vec<SignalContribution>,
}

/// Manual duplicate marking (`mark_duplicate`, MGR-05 step 1).
///
/// A manual pair merges its endpoints into one cluster regardless of score and
/// is reported with `manual: true` (reversible: the report is a view, no stored
/// data is mutated). Both endpoints must exist among the linked candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualLink {
    /// One endpoint id.
    pub left: String,
    /// Other endpoint id.
    pub right: String,
}

impl ManualLink {
    /// Build a manual link between two candidate ids.
    pub fn new(left: impl Into<String>, right: impl Into<String>) -> Self {
        Self {
            left: left.into(),
            right: right.into(),
        }
    }
}

/// A resolved group of candidate ids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkCluster {
    /// Representative id (lexicographically smallest member).
    pub canonical: String,
    /// Member ids, sorted ascending.
    pub members: Vec<String>,
}

/// Deterministic output of [`link_entities`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LinkReport {
    /// Scored pairs with verdict `AutoLink` or `Review` (plus every manual
    /// pair), sorted by `(left, right)`. `Distinct` pairs are omitted.
    pub decisions: Vec<LinkDecision>,
    /// Clusters with two or more members (auto-linked + manual), sorted by
    /// `canonical`. Singletons are omitted (no linkage).
    pub clusters: Vec<LinkCluster>,
}

// ── Scoring ──

/// Compare one candidate pair and return `(bits, contributions)`.
///
/// Pure and deterministic: identical inputs produce bit-identical weights.
/// Expected-value kinds that are malformed (wrong value type, empty text,
/// zero-norm or mismatched embeddings) are treated as missing (neutral).
pub fn match_score(
    left: &[LinkSignal],
    right: &[LinkSignal],
    config: &LinkConfig,
) -> (f32, Vec<SignalContribution>) {
    const KINDS: [SignalKind; 4] = [
        SignalKind::Name,
        SignalKind::Email,
        SignalKind::Phone,
        SignalKind::Embedding,
    ];

    let mut total = config.prior_bits;
    let mut contributions = Vec::with_capacity(KINDS.len());
    for kind in KINDS {
        let (outcome, weight, evidence) = compare_signals(
            kind,
            first_signal(left, kind),
            first_signal(right, kind),
            config,
        );
        total += weight;
        contributions.push(SignalContribution {
            kind,
            outcome,
            weight,
            evidence,
        });
    }
    (total, contributions)
}

/// First signal of the requested kind (input order decides when duplicated).
fn first_signal(signals: &[LinkSignal], kind: SignalKind) -> Option<&LinkSignal> {
    signals.iter().find(|signal| signal.kind == kind)
}

fn compare_signals(
    kind: SignalKind,
    left: Option<&LinkSignal>,
    right: Option<&LinkSignal>,
    config: &LinkConfig,
) -> (SignalOutcome, f32, String) {
    let (Some(left), Some(right)) = (left, right) else {
        return (
            SignalOutcome::Missing,
            0.0,
            format!("{} absent", kind.as_str()),
        );
    };
    let weight = config.weight(kind);
    match kind {
        SignalKind::Embedding => {
            let (SignalValue::Embedding(a), SignalValue::Embedding(b)) =
                (&left.value, &right.value)
            else {
                return (
                    SignalOutcome::Missing,
                    0.0,
                    format!("{} malformed value", kind.as_str()),
                );
            };
            match cosine_similarity(a, b) {
                Some(cosine) if cosine >= config.embedding_threshold => (
                    SignalOutcome::Agree,
                    weight.agree_bits(),
                    format!(
                        "embedding cosine {cosine:.4} >= {:.4}",
                        config.embedding_threshold
                    ),
                ),
                Some(cosine) => (
                    SignalOutcome::Disagree,
                    weight.disagree_bits(),
                    format!(
                        "embedding cosine {cosine:.4} < {:.4}",
                        config.embedding_threshold
                    ),
                ),
                None => (
                    SignalOutcome::Missing,
                    0.0,
                    "embedding undefined (zero norm or dimension mismatch)".to_string(),
                ),
            }
        }
        _ => {
            let (SignalValue::Text(a), SignalValue::Text(b)) = (&left.value, &right.value) else {
                return (
                    SignalOutcome::Missing,
                    0.0,
                    format!("{} malformed value", kind.as_str()),
                );
            };
            let (normalized_a, normalized_b) = (normalize_text(kind, a), normalize_text(kind, b));
            if normalized_a.is_empty() || normalized_b.is_empty() {
                return (
                    SignalOutcome::Missing,
                    0.0,
                    format!("{} empty after normalization", kind.as_str()),
                );
            }
            if normalized_a == normalized_b {
                (
                    SignalOutcome::Agree,
                    weight.agree_bits(),
                    format!("{} matches after normalization", kind.as_str()),
                )
            } else {
                (
                    SignalOutcome::Disagree,
                    weight.disagree_bits(),
                    format!("{} values differ", kind.as_str()),
                )
            }
        }
    }
}

/// Kind-specific deterministic normalization (see [`SignalKind`]).
fn normalize_text(kind: SignalKind, value: &str) -> String {
    match kind {
        SignalKind::Email => value.trim().to_lowercase(),
        SignalKind::Phone => value.chars().filter(char::is_ascii_digit).collect(),
        // Name (and any future text kind): trim, collapse whitespace, lowercase.
        _ => value
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase(),
    }
}

/// Deterministic cosine similarity, `None` when undefined (dims differ, zero
/// norm, non-finite component).
fn cosine_similarity(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.is_empty() || a.len() != b.len() {
        return None;
    }
    let (mut dot, mut norm_a, mut norm_b) = (0.0_f32, 0.0_f32, 0.0_f32);
    for (x, y) in a.iter().zip(b.iter()) {
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }
    if norm_a == 0.0 || norm_b == 0.0 {
        return None;
    }
    let cosine = dot / (norm_a.sqrt() * norm_b.sqrt());
    cosine.is_finite().then_some(cosine)
}

/// Deterministic posterior match probability from a match weight in bits.
///
/// `Pr(match) = 2^bits / (1 + 2^bits)`, evaluated in the numerically stable
/// form for both signs of `bits`.
pub fn bits_to_posterior(bits: f32) -> f32 {
    if bits.is_nan() {
        return 0.5;
    }
    if bits >= 0.0 {
        1.0 / (1.0 + (-bits).exp2())
    } else {
        let e = bits.exp2();
        e / (1.0 + e)
    }
}

fn verdict_for(bits: f32, config: &LinkConfig) -> LinkVerdict {
    if bits >= config.auto_link_bits {
        LinkVerdict::AutoLink
    } else if bits >= config.review_bits {
        LinkVerdict::Review
    } else {
        LinkVerdict::Distinct
    }
}

// ── Linking + clustering ──

/// Link every candidate pair with the given configuration and manual marks.
///
/// Pairs scoring at or above `auto_link_bits` merge their endpoints; pairs in
/// the review band are reported with provenance but never merged; pairs below
/// `review_bits` are omitted. [`ManualLink`]s (`mark_duplicate`) force a merge
/// regardless of score and are tagged `manual: true` in the report.
///
/// # Errors
/// [`Error::InvalidInput`] when a candidate id is empty, ids are not unique,
/// or a [`ManualLink`] references an unknown id or links an id to itself.
pub fn link_entities(
    entities: &[LinkEntity],
    config: &LinkConfig,
    manual: &[ManualLink],
) -> Result<LinkReport> {
    let ids = validate_entities(entities)?;

    // Deterministic pair enumeration: sort by id, then all i < j pairs → the
    // decision keys are already in (left, right) order.
    let mut ordered: Vec<&LinkEntity> = entities.iter().collect();
    ordered.sort_by(|a, b| a.id.cmp(&b.id));

    let mut decisions: BTreeMap<(String, String), LinkDecision> = BTreeMap::new();
    for (index, left) in ordered.iter().enumerate() {
        for right in ordered.iter().skip(index + 1) {
            let (bits, contributions) = match_score(&left.signals, &right.signals, config);
            let verdict = verdict_for(bits, config);
            if verdict == LinkVerdict::Distinct {
                continue;
            }
            decisions.insert(
                (left.id.clone(), right.id.clone()),
                LinkDecision {
                    left: left.id.clone(),
                    right: right.id.clone(),
                    bits,
                    posterior: bits_to_posterior(bits),
                    verdict,
                    manual: false,
                    contributions,
                },
            );
        }
    }

    for mark in manual {
        let (left_id, right_id) = if mark.left <= mark.right {
            (&mark.left, &mark.right)
        } else {
            (&mark.right, &mark.left)
        };
        if left_id == right_id {
            return Err(Error::InvalidInput(
                "manual link endpoints must differ".into(),
            ));
        }
        let (Some(left), Some(right)) = (
            find_entity(&ordered, left_id),
            find_entity(&ordered, right_id),
        ) else {
            return Err(Error::InvalidInput(format!(
                "manual link references unknown id ({} <-> {})",
                mark.left, mark.right
            )));
        };
        let (bits, contributions) = match_score(&left.signals, &right.signals, config);
        decisions.insert(
            (left_id.clone(), right_id.clone()),
            LinkDecision {
                left: left_id.clone(),
                right: right_id.clone(),
                bits,
                posterior: bits_to_posterior(bits),
                verdict: LinkVerdict::AutoLink,
                manual: true,
                contributions,
            },
        );
    }

    // Merge the auto-linked (and manual) edges and collect clusters.
    let mut disjoint = DisjointSet::new(ids.iter().cloned());
    for ((left, right), decision) in &decisions {
        if decision.verdict == LinkVerdict::AutoLink {
            disjoint.union(left, right);
        }
    }
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for id in &ids {
        groups
            .entry(disjoint.find(id))
            .or_default()
            .push(id.clone());
    }
    let clusters = groups
        .into_iter()
        .filter(|(_, members)| members.len() >= 2)
        .map(|(canonical, members)| LinkCluster { canonical, members })
        .collect();

    Ok(LinkReport {
        decisions: decisions.into_values().collect(),
        clusters,
    })
}

fn find_entity<'a>(ordered: &[&'a LinkEntity], id: &str) -> Option<&'a LinkEntity> {
    ordered.iter().find(|entity| entity.id == id).copied()
}

/// Validate candidate ids (boundary): non-empty and unique.
fn validate_entities(entities: &[LinkEntity]) -> Result<std::collections::BTreeSet<String>> {
    let mut ids = std::collections::BTreeSet::new();
    for entity in entities {
        if entity.id.is_empty() {
            return Err(Error::InvalidInput(
                "link entity id must be non-empty".into(),
            ));
        }
        if !ids.insert(entity.id.clone()) {
            return Err(Error::InvalidInput(format!(
                "duplicate link entity id: {}",
                entity.id
            )));
        }
    }
    Ok(ids)
}

/// Union-find over candidate ids; every union keeps the lexicographically
/// smallest id as the component root (canonical, deterministic).
struct DisjointSet {
    parent: BTreeMap<String, String>,
}

impl DisjointSet {
    fn new(ids: impl IntoIterator<Item = String>) -> Self {
        Self {
            parent: ids.into_iter().map(|id| (id.clone(), id)).collect(),
        }
    }

    fn find(&mut self, id: &str) -> String {
        let parent = self
            .parent
            .get(id)
            .cloned()
            .unwrap_or_else(|| id.to_string());
        if parent == id {
            return parent;
        }
        let root = self.find(&parent);
        self.parent.insert(id.to_string(), root.clone());
        root
    }

    fn union(&mut self, left: &str, right: &str) {
        let (root_left, root_right) = (self.find(left), self.find(right));
        if root_left == root_right {
            return;
        }
        let (min, max) = if root_left < root_right {
            (root_left, root_right)
        } else {
            (root_right, root_left)
        };
        self.parent.insert(max, min);
    }
}

/// Map entity field values to link signals following `map` order.
///
/// Text kinds accept [`FieldValue::String`]; [`SignalKind::Embedding`] accepts
/// [`FieldValue::ListFloat`]. Missing fields and unsupported value types are
/// skipped (they simply produce no signal for that kind). Output order follows
/// `map` (deterministic), so callers control which field wins per kind.
pub fn signals_from_fields(
    fields: &HashMap<String, FieldValue>,
    map: &[(&str, SignalKind)],
) -> Vec<LinkSignal> {
    let mut signals = Vec::new();
    for (field, kind) in map {
        let Some(value) = fields.get(*field) else {
            continue;
        };
        match (kind, value) {
            (SignalKind::Embedding, FieldValue::ListFloat(values)) => {
                signals.push(LinkSignal::embedding(
                    values.iter().map(|value| *value as f32).collect(),
                ));
            }
            (
                SignalKind::Name | SignalKind::Email | SignalKind::Phone,
                FieldValue::String(text),
            ) => {
                signals.push(LinkSignal::text(*kind, text.clone()));
            }
            _ => {}
        }
    }
    signals
}

#[cfg(test)]
#[path = "linking_tests.rs"]
mod tests;
