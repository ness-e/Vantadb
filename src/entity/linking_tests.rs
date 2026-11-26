//! Tests for deterministic multi-signal entity linking (WIRE-05).

use super::*;
use crate::node::FieldValue;
use std::collections::HashMap;

// ── helpers ──

fn name(v: &str) -> LinkSignal {
    LinkSignal::text(SignalKind::Name, v)
}

fn email(v: &str) -> LinkSignal {
    LinkSignal::text(SignalKind::Email, v)
}

fn phone(v: &str) -> LinkSignal {
    LinkSignal::text(SignalKind::Phone, v)
}

fn emb(v: &[f32]) -> LinkSignal {
    LinkSignal::embedding(v.to_vec())
}

fn entity(id: &str, signals: Vec<LinkSignal>) -> LinkEntity {
    LinkEntity::new(id, signals)
}

fn contribution(contributions: &[SignalContribution], kind: SignalKind) -> &SignalContribution {
    contributions
        .iter()
        .find(|c| c.kind == kind)
        .expect("contribution for kind")
}

// ── determinism + scoring ──

#[test]
fn linking_match_score_is_deterministic() {
    let a = vec![name("Alice Smith"), email("Alice@Example.COM")];
    let b = vec![name("  alice   smith "), email("alice@example.com")];
    let config = LinkConfig::default();

    let (bits_1, contributions_1) = match_score(&a, &b, &config);
    let (bits_2, contributions_2) = match_score(&a, &b, &config);

    assert_eq!(bits_1, bits_2, "same inputs must give bit-identical scores");
    assert_eq!(contributions_1, contributions_2);
    // name (log2(950) ≈ 9.89) + email (log2(1980) ≈ 10.95) both agree.
    assert!((bits_1 - 20.84).abs() < 0.05, "got {bits_1}");
    for kind in [SignalKind::Name, SignalKind::Email] {
        assert_eq!(
            contribution(&contributions_1, kind).outcome,
            SignalOutcome::Agree
        );
    }
}

#[test]
fn linking_auto_link_requires_multiple_signals() {
    let config = LinkConfig::default();

    // A single strong identifier is not enough to auto-link by default:
    let single = [name("Alice Smith")];
    let variant = [name("alice   smith")];
    let (single_bits, _) = match_score(&single, &variant, &config);
    assert!(
        single_bits < config.auto_link_bits,
        "one signal must not auto-link"
    );
    assert!(
        single_bits >= config.review_bits,
        "one strong signal lands in review"
    );

    // Two agreeing signals do:
    let pair_a = [name("Alice Smith"), email("alice@example.com")];
    let pair_b = [name("alice smith"), email("ALICE@example.com")];
    let (pair_bits, _) = match_score(&pair_a, &pair_b, &config);
    assert!(
        pair_bits >= config.auto_link_bits,
        "two signals must auto-link"
    );
}

#[test]
fn linking_conservative_threshold_exceeds_every_single_signal() {
    let config = LinkConfig::default();
    for kind in [
        SignalKind::Name,
        SignalKind::Email,
        SignalKind::Phone,
        SignalKind::Embedding,
    ] {
        assert!(
            config.weight(kind).agree_bits() < config.auto_link_bits,
            "{kind:?} alone must not reach auto_link_bits"
        );
    }
}

#[test]
fn linking_text_normalization_agrees_on_variants() {
    let config = LinkConfig::default();

    let (name_bits, name_c) =
        match_score(&[name("  Alice   SMITH ")], &[name("alice smith")], &config);
    assert_eq!(
        contribution(&name_c, SignalKind::Name).outcome,
        SignalOutcome::Agree
    );
    assert!(
        name_bits > 0.0,
        "name match is positive evidence: {name_bits}"
    );

    let (phone_bits, phone_c) = match_score(
        &[phone("+34 600-123-456")],
        &[phone("34600123456")],
        &config,
    );
    assert_eq!(
        contribution(&phone_c, SignalKind::Phone).outcome,
        SignalOutcome::Agree
    );
    assert!(phone_bits > 0.0);

    let (email_bits, email_c) = match_score(
        &[email("  Alice@Example.com ")],
        &[email("alice@example.com")],
        &config,
    );
    assert_eq!(
        contribution(&email_c, SignalKind::Email).outcome,
        SignalOutcome::Agree
    );
    assert!(email_bits > 0.0);
}

#[test]
fn linking_disagreement_subtracts_weight() {
    let config = LinkConfig::default();

    let (bits, contributions) = match_score(&[name("Alice Smith")], &[name("Bob Jones")], &config);
    let name_contribution = contribution(&contributions, SignalKind::Name);
    assert_eq!(name_contribution.outcome, SignalOutcome::Disagree);
    // log2(0.05/0.999) ≈ -4.32.
    assert!(name_contribution.weight < 0.0);
    assert!((bits - (-4.32)).abs() < 0.05, "got {bits}");
}

#[test]
fn linking_missing_signals_are_neutral() {
    let config = LinkConfig::default();

    // Name only on one side, e-mail only on the other → nothing comparable.
    let (bits, contributions) = match_score(
        &[name("Alice Smith")],
        &[email("alice@example.com")],
        &config,
    );
    assert_eq!(bits, config.prior_bits, "missing signals contribute 0");
    for c in &contributions {
        assert_eq!(c.outcome, SignalOutcome::Missing);
        assert_eq!(c.weight, 0.0);
        assert!(c.evidence.contains("absent"));
    }

    // Empty text normalizes to nothing and is missing, not disagreeing.
    let (empty_bits, empty_c) = match_score(&[name("   ")], &[name("Alice")], &config);
    assert_eq!(
        contribution(&empty_c, SignalKind::Name).outcome,
        SignalOutcome::Missing
    );
    assert_eq!(empty_bits, config.prior_bits);
}

#[test]
fn linking_embedding_threshold_agrees_and_disagrees() {
    let config = LinkConfig::default().with_embedding_threshold(0.90);

    let (agree_bits, agree_c) = match_score(&[emb(&[1.0, 0.0])], &[emb(&[0.98, 0.199])], &config);
    let emb_agree = contribution(&agree_c, SignalKind::Embedding);
    assert_eq!(emb_agree.outcome, SignalOutcome::Agree);
    assert!(
        emb_agree.evidence.contains(">="),
        "evidence: {}",
        emb_agree.evidence
    );
    assert!(agree_bits > 0.0);

    let (disagree_bits, disagree_c) =
        match_score(&[emb(&[1.0, 0.0])], &[emb(&[0.0, 1.0])], &config);
    assert_eq!(
        contribution(&disagree_c, SignalKind::Embedding).outcome,
        SignalOutcome::Disagree
    );
    assert!(disagree_bits < 0.0);
}

#[test]
fn linking_embedding_zero_norm_is_missing() {
    let config = LinkConfig::default();
    // Zero-norm embeddings are undefined for cosine → neutral, never a panic.
    let (bits, contributions) = match_score(&[emb(&[0.0, 0.0])], &[emb(&[1.0, 0.0])], &config);
    assert_eq!(bits, config.prior_bits);
    assert_eq!(
        contribution(&contributions, SignalKind::Embedding).outcome,
        SignalOutcome::Missing
    );

    // Mismatched dimensions are also neutral.
    let (mismatch_bits, mismatch_c) = match_score(&[emb(&[1.0, 0.0])], &[emb(&[1.0])], &config);
    assert_eq!(mismatch_bits, config.prior_bits);
    assert_eq!(
        contribution(&mismatch_c, SignalKind::Embedding).outcome,
        SignalOutcome::Missing
    );
}

#[test]
fn linking_posterior_matches_splink_reference() {
    // Splink's worked example: total match weight 9.48 → probability ≈ 0.999.
    let p = bits_to_posterior(9.48);
    assert!((p - 0.999).abs() < 0.002, "got {p}");

    assert_eq!(bits_to_posterior(0.0), 0.5);
    let (up, down) = (bits_to_posterior(10.0), bits_to_posterior(-10.0));
    assert!(
        (up + down - 1.0).abs() < 1e-4,
        "posterior symmetry: {up} + {down}"
    );
    assert!(bits_to_posterior(-100.0) < 1e-6);
}

// ── reports: clusters + manual marks ──

#[test]
fn linking_manual_mark_duplicate_forces_cluster() {
    let a = entity("a", vec![name("Alice Smith"), email("alice@x.com")]);
    let b = entity("b", vec![name("Alice Smith"), email("alice@x.com")]);
    let c = entity("c", vec![name("Carol Danvers"), email("carol@x.com")]);

    // Without the manual mark, (b, c) scores below the lower threshold.
    let manual = [ManualLink::new("c", "b")];
    let report = link_entities(&[a, b, c], &LinkConfig::default(), &manual).expect("link");

    // Transitive: a–b auto + b–c manual → one cluster {a, b, c}.
    assert_eq!(report.clusters.len(), 1);
    assert_eq!(report.clusters[0].canonical, "a");
    assert_eq!(report.clusters[0].members, vec!["a", "b", "c"]);

    let manual_decision = report
        .decisions
        .iter()
        .find(|d| d.left == "b" && d.right == "c")
        .expect("manual decision is reported");
    assert!(manual_decision.manual);
    assert_eq!(manual_decision.verdict, LinkVerdict::AutoLink);
    // The score is still reported for auditability (it was below threshold).
    assert!(manual_decision.bits < 0.0, "got {}", manual_decision.bits);
    assert!(
        !report
            .decisions
            .iter()
            .any(|d| d.left == "a" && d.right == "c"),
        "distinct pairs are omitted from decisions"
    );
}

#[test]
fn linking_clusters_are_deterministic_and_sorted() {
    let a = entity("a", vec![name("Alice Smith"), email("alice@x.com")]);
    let b = entity("b", vec![name("Alice Smith"), email("alice@x.com")]);
    let c = entity("c", vec![name("Alice Smith"), email("alice@x.com")]);

    let report_1 = link_entities(
        &[c.clone(), a.clone(), b.clone()],
        &LinkConfig::default(),
        &[],
    )
    .expect("link");
    let report_2 = link_entities(&[a, b, c], &LinkConfig::default(), &[]).expect("link");

    assert_eq!(report_1, report_2, "input order must not change the report");
    assert_eq!(report_1.clusters.len(), 1);
    assert_eq!(report_1.clusters[0].members, vec!["a", "b", "c"]);

    // Decisions sorted by (left, right) and structurally deterministic.
    let pairs: Vec<(&str, &str)> = report_1
        .decisions
        .iter()
        .map(|d| (d.left.as_str(), d.right.as_str()))
        .collect();
    let mut sorted = pairs.clone();
    sorted.sort();
    assert_eq!(pairs, sorted);
    for decision in &report_1.decisions {
        assert_eq!(decision.verdict, LinkVerdict::AutoLink);
        assert!(!decision.manual);
    }
}

#[test]
fn linking_review_band_is_reported_not_merged() {
    // One strong signal only → review band: reported with provenance, not merged.
    let a = entity("a", vec![email("alice@x.com")]);
    let b = entity("b", vec![email("alice@x.com")]);
    let report = link_entities(&[a, b], &LinkConfig::default(), &[]).expect("link");

    assert!(report.clusters.is_empty(), "review pairs must not merge");
    assert_eq!(report.decisions.len(), 1);
    assert_eq!(report.decisions[0].verdict, LinkVerdict::Review);
    assert!(report.decisions[0].bits >= LinkConfig::default().review_bits);
    assert!(report.decisions[0].posterior > 0.5);
}

#[test]
fn linking_config_override_changes_score() {
    let a = [name("Alice Smith")];
    let b = [name("Alice Smith")];

    let default_bits = match_score(&a, &b, &LinkConfig::default()).0;
    let custom = LinkConfig::default().with_signal_weight(
        SignalKind::Name,
        SignalWeight::new(0.99, 0.001).expect("valid"),
    );
    let custom_bits = match_score(&a, &b, &custom).0;

    assert!(custom_bits > default_bits);
    assert_eq!(
        custom_bits,
        match_score(&a, &b, &custom).0,
        "still deterministic"
    );
    // log2(990) ≈ 9.95 vs log2(950) ≈ 9.89.
    assert!((custom_bits - 9.95).abs() < 0.05, "got {custom_bits}");
}

#[test]
fn linking_rejects_invalid_input() {
    // Invalid weight pairs are rejected at construction (boundary validation).
    assert!(SignalWeight::new(0.5, 0.6).is_err(), "m < u");
    assert!(SignalWeight::new(1.0, 0.1).is_err(), "m == 1");
    assert!(SignalWeight::new(0.0, 0.0).is_err(), "zeroes");
    assert!(SignalWeight::new(f32::NAN, 0.1).is_err(), "NaN");
    assert!(SignalWeight::new(0.9, 0.05).is_ok());

    let config = LinkConfig::default();
    let a = entity("a", vec![name("Alice")]);

    // Empty id.
    let empty = entity("", vec![name("Alice")]);
    assert!(link_entities(&[empty], &config, &[]).is_err());

    // Duplicate ids.
    let b = entity("a", vec![name("Alice")]);
    assert!(link_entities(&[a, b], &config, &[]).is_err());

    // Manual link to an unknown id, or with equal endpoints.
    let c = entity("c", vec![name("Alice")]);
    let unknown = [ManualLink::new("c", "nope")];
    assert!(link_entities(std::slice::from_ref(&c), &config, &unknown).is_err());
    let self_link = [ManualLink::new("c", "c")];
    assert!(link_entities(std::slice::from_ref(&c), &config, &self_link).is_err());
}

// ── field mapping helper ──

#[test]
fn linking_signals_from_fields_maps_text_and_embedding() {
    let mut fields = HashMap::new();
    fields.insert(
        "full_name".to_string(),
        FieldValue::String("Alice Smith".to_string()),
    );
    fields.insert(
        "contact_email".to_string(),
        FieldValue::String("alice@x.com".to_string()),
    );
    fields.insert(
        "face_emb".to_string(),
        FieldValue::ListFloat(vec![0.1, 0.2, 0.3]),
    );
    fields.insert("age".to_string(), FieldValue::Int(42));

    let map = [
        ("full_name", SignalKind::Name),
        ("contact_email", SignalKind::Email),
        ("face_emb", SignalKind::Embedding),
        ("age", SignalKind::Name), // unsupported value type → skipped
        ("missing_field", SignalKind::Phone), // absent → skipped
    ];
    let signals = signals_from_fields(&fields, &map);

    assert_eq!(signals.len(), 3, "only mappable values produce signals");
    assert_eq!(signals[0].kind, SignalKind::Name);
    assert!(matches!(&signals[0].value, SignalValue::Text(t) if t == "Alice Smith"));
    assert_eq!(signals[1].kind, SignalKind::Email);
    assert_eq!(signals[2].kind, SignalKind::Embedding);
    assert!(matches!(&signals[2].value, SignalValue::Embedding(e) if e.len() == 3 && e[1] == 0.2));

    // Deterministic: same inputs → same signals.
    assert_eq!(signals, signals_from_fields(&fields, &map));
}
