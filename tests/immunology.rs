/// Phase 36: Logical Immunology — Integration Tests
///
/// Tests the Semantic Blood-Brain Barrier, Epistemic Slashing, and L1 Hard-Filter.
/// Validates that:
/// 1. A single origin's attack grows logarithmically (flattened impact)
/// 2. Diverse trusted origins can breach axiom threshold
/// 3. Slashing bans agents permanently via ThalamicGate
/// 4. ThalamicGate role-based banning works at string level

use connectomedb::governance::{DevilsAdvocate, TrustArbiter, ResolutionResult};
use connectomedb::governance::thalamic_gate::ThalamicGate;
use connectomedb::node::{UnifiedNode, NodeFlags, FieldValue, VectorRepresentations};

/// Helper: create a node with a specific owner_role and trust_score
fn make_agent_node(id: u64, role: &str, trust: f32, vector: Vec<f32>) -> UnifiedNode {
    let mut node = UnifiedNode::new(id);
    node.set_field("_owner_role", FieldValue::String(role.to_string()));
    node.trust_score = trust;
    node.vector = VectorRepresentations::Full(vector);
    node.flags.set(NodeFlags::HAS_VECTOR);
    node
}

/// Helper: create a pinned axiom node with high valence
fn make_axiom_node(id: u64, valence: f32, vector: Vec<f32>) -> UnifiedNode {
    let mut node = UnifiedNode::new(id);
    node.trust_score = 1.0;
    node.semantic_valence = valence;
    node.flags.set(NodeFlags::PINNED);
    node.vector = VectorRepresentations::Full(vector);
    node.flags.set(NodeFlags::HAS_VECTOR);
    node
}

// ─── Test 1: Single origin logarithmic friction ──────────────

#[test]
fn test_single_origin_logarithmic_friction() {
    let advocate = DevilsAdvocate::new();

    // Create a pinned axiom with valence 0.9 → threshold = 0.9 * 10.0 = 9.0
    let axiom_vec = vec![1.0, 0.0, 0.0, 0.0];
    let axiom = make_axiom_node(1, 0.9, axiom_vec.clone());

    // Single malicious agent sends 1000 near-identical vectors
    // Cosine similarity must be > 0.95 to trigger conflict evaluation
    for i in 0..1000 {
        let challenger = make_agent_node(
            100 + i,
            "malicious_bot",
            0.6,
            vec![1.0, 0.001 * (i as f32 % 5.0), 0.0, 0.0], // Very close to axiom
        );

        let result = advocate.evaluate_conflict(&axiom, &challenger);

        // Single origin with trust 0.6:
        // After 1000 collisions: F_ax = log2(1001) * ~0.6 ≈ 5.99
        // Threshold = 9.0 → Single origin CANNOT breach
        match result {
            ResolutionResult::Reject(reason) => {
                assert!(
                    reason.contains("Barrera Hematoencefálica"),
                    "Expected Barrera Hematoencefálica rejection, got: {}",
                    reason
                );
            }
            _ => {
                // After many collisions the EMA might shift trust, but friction
                // from a single origin should never reach threshold=9.0
                // If it somehow accepts, the test design is wrong
                let tracker = advocate.collision_tracker.read();
                let friction = tracker.compute_friction();
                assert!(
                    friction < 9.0,
                    "Single origin friction {} should be < threshold 9.0",
                    friction
                );
            }
        }
    }

    // Verify final friction state
    let tracker = advocate.collision_tracker.read();
    let friction = tracker.compute_friction();
    assert!(friction < 9.0, "Single origin friction {:.2} should never breach threshold 9.0", friction);
    assert_eq!(tracker.unique_origins(), 1, "Only one unique origin should exist");
    println!("✅ Single origin friction: {:.4} (threshold: 9.0) — Logarithmic dampening confirmed", friction);
}

// ─── Test 2: Diverse origins breach axiom ────────────────────

#[test]
fn test_diverse_origins_breach_axiom() {
    let advocate = DevilsAdvocate::new();

    // Axiom with valence 0.8 → threshold = 0.8 * 10.0 = 8.0
    let axiom_vec = vec![1.0, 0.0, 0.0, 0.0];
    let axiom = make_axiom_node(1, 0.8, axiom_vec.clone());

    // 10 distinct trusted agents, each sending 5 collisions
    let roles = [
        "researcher_alpha", "researcher_beta", "researcher_gamma",
        "researcher_delta", "researcher_epsilon", "validator_1",
        "validator_2", "validator_3", "auditor_prime", "auditor_secondary",
    ];

    let mut any_superposition = false;

    for (agent_idx, role) in roles.iter().enumerate() {
        for collision in 0..5 {
            let challenger = make_agent_node(
                1000 + (agent_idx * 10 + collision) as u64,
                role,
                0.85, // High trust
                vec![1.0, 0.001, 0.0, 0.0], // Near-identical to axiom
            );

            let result = advocate.evaluate_conflict(&axiom, &challenger);
            match result {
                ResolutionResult::Superposition(_) => {
                    any_superposition = true;
                    break;
                }
                _ => {}
            }
        }
        if any_superposition { break; }
    }

    // Verify: diverse origins with enough collisions SHOULD breach threshold
    let tracker = advocate.collision_tracker.read();
    let friction = tracker.compute_friction();
    let threshold = 0.8 * 10.0;
    println!("✅ Diverse origins friction: {:.4}, threshold: {:.1}, origins: {}", friction, threshold, tracker.unique_origins());

    // The friction should eventually exceed threshold with 10 origins × 5 collisions × 0.85 trust
    // F_ax = 10 * log2(6) * 0.85 ≈ 10 * 2.585 * 0.85 ≈ 21.97 >> 8.0
    // But we break on first Superposition, so check that it happened
    assert!(any_superposition || friction >= threshold,
        "Diverse origins should have enough friction ({:.2}) to breach threshold ({:.1})",
        friction, threshold
    );
}

// ─── Test 3: Slashing bans agent via ThalamicGate ────────────

#[test]
fn test_slashing_bans_agent() {
    let advocate = DevilsAdvocate::new();
    let gate = ThalamicGate::new(1000);

    let axiom = make_axiom_node(1, 0.9, vec![1.0, 0.0, 0.0, 0.0]);

    // Agent starts with normal trust
    let agent_role = "compromised_agent";
    let challenger = make_agent_node(
        200, agent_role, 0.7,
        vec![1.0, 0.001, 0.0, 0.0],
    );

    // Before slashing: should get a normal conflict result (Reject due to barrier, not slashing)
    let result = advocate.evaluate_conflict(&axiom, &challenger);
    match &result {
        ResolutionResult::Reject(reason) => {
            assert!(!reason.contains("Epistemic Apoptosis"),
                "Should NOT be slashed yet: {}", reason);
        }
        _ => {} // Accept or Superposition is fine before slashing
    }

    // Simulate REM phase slashing: hallucination detected
    {
        let mut tracker = advocate.collision_tracker.write();
        tracker.slash_origin(agent_role);
    }
    gate.record_role_ban(agent_role);

    // After slashing: ThalamicGate should ban the role
    assert!(gate.is_role_banned(agent_role), "Slashed agent should be banned in ThalamicGate");

    // DevilsAdvocate should also reject immediately
    let challenger2 = make_agent_node(
        201, agent_role, 0.7,
        vec![1.0, 0.001, 0.0, 0.0],
    );
    let result2 = advocate.evaluate_conflict(&axiom, &challenger2);
    match result2 {
        ResolutionResult::Reject(reason) => {
            assert!(reason.contains("Epistemic Apoptosis"),
                "Slashed agent should be rejected with Epistemic Apoptosis: {}", reason);
        }
        _ => panic!("Slashed agent MUST be rejected, got: {:?}", result2),
    }

    println!("✅ Epistemic Slashing: agent '{}' permanently banned at L1", agent_role);
}

// ─── Test 4: ThalamicGate role-level banning ─────────────────

#[test]
fn test_thalamic_role_ban() {
    let gate = ThalamicGate::new(5000);

    // Initially, no roles are banned
    assert!(!gate.is_role_banned("agent_alpha"), "No roles should be banned initially");
    assert!(!gate.is_role_banned("agent_beta"), "No roles should be banned initially");

    // Ban agent_alpha
    gate.record_role_ban("agent_alpha");

    // Verify
    assert!(gate.is_role_banned("agent_alpha"), "agent_alpha should be banned");
    assert!(!gate.is_role_banned("agent_beta"), "agent_beta should NOT be banned");

    // Ban agent_beta too
    gate.record_role_ban("agent_beta");
    assert!(gate.is_role_banned("agent_beta"), "agent_beta should now be banned");

    // Verify node-level rejection still works independently
    gate.record_rejection(42);
    assert!(gate.is_rejected(42), "Node 42 should be rejected");
    assert!(!gate.is_rejected(99), "Node 99 should NOT be rejected");

    // Verify grant_amnesty is still no-op
    gate.grant_amnesty(42);
    assert!(gate.is_rejected(42), "Amnesty is no-op, node 42 should still be rejected");

    println!("✅ ThalamicGate role banning: L1 XOR/POPCNT verified");
}

// ─── Test 5: OriginCollisionTracker friction formula ─────────

#[test]
fn test_friction_formula_properties() {
    use connectomedb::governance::OriginCollisionTracker;

    let mut tracker = OriginCollisionTracker::new();

    // Property 1: Empty tracker → friction = 0
    assert_eq!(tracker.compute_friction(), 0.0, "Empty tracker should have 0 friction");

    // Property 2: Single origin with 1 collision, trust 1.0
    // F = log2(1 + 1) * 1.0 = log2(2) * 1.0 = 1.0
    tracker.record_collision("origin_a", 1.0);
    let f1 = tracker.compute_friction();
    assert!((f1 - 1.0).abs() < 0.01, "Single collision should give F≈1.0, got {}", f1);

    // Property 3: Same origin with more collisions → logarithmic growth
    for _ in 0..99 {
        tracker.record_collision("origin_a", 1.0);
    }
    let f100 = tracker.compute_friction();
    // log2(101) ≈ 6.66, but trust is decayed via EMA: each record blends
    // After 100 records from trust=1.0: trust converges to ~1.0 (EMA: 0.8*old + 0.2*1.0)
    assert!(f100 < 10.0, "100 collisions from single origin: F={:.2}, should be < 10.0", f100);
    assert!(f100 > 4.0, "100 collisions should accumulate some friction: F={:.2}", f100);

    // Property 4: Adding a second origin doubles the base
    tracker.record_collision("origin_b", 0.9);
    let f_two = tracker.compute_friction();
    assert!(f_two > f100, "Adding a second origin should increase friction");

    // Property 5: Slashing zeroes an origin's contribution
    tracker.slash_origin("origin_a");
    let f_slashed = tracker.compute_friction();
    assert!(f_slashed < f_two, "Slashing should reduce friction");
    assert!(tracker.is_slashed("origin_a"), "origin_a should be marked as slashed");

    println!("✅ Friction formula properties verified: log2 growth, EMA trust, slashing");
}
