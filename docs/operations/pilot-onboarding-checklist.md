---
title: Pilot Onboarding Checklist
type: operations
status: active
tags: [vantadb, operations, pilot, onboarding, checklist]
last_reviewed: 2026-07-26
---

# VantaDB Pilot — Onboarding Checklist

> **Purpose:** Step-by-step checklist to get each pilot participant from sign-up to a working integration.
> **Owner:** Pilot program lead
> **Estimated completion time:** 45–60 minutes

---

## Phase 0: Pre-Onboarding (Day -3 to Day 0)

| # | Task | Owner | Done |
|---|---|---|---|
| 0.1 | Send pilot agreement + NDA for signature | Program lead | ☐ |
| 0.2 | Receive signed agreement, file in `./pilot-participants/` | Program lead | ☐ |
| 0.3 | Schedule kickoff call (30 min, within first week) | Program lead | ☐ |
| 0.4 | Add participant to private Slack/Discord channel | Program lead | ☐ |
| 0.5 | Share welcome email with: channel invite, onboarding checklist link, and docs links | Program lead | ☐ |

---

## Phase 1: Kickoff Call (Day 0)

| # | Agenda Item | Done |
|---|---|---|
| 1.1 | Introductions — participant project, team, goals for pilot | ☐ |
| 1.2 | Confirm pilot structure: duration, check-in cadence, expectations | ☐ |
| 1.3 | Walk through onboarding checklist | ☐ |
| 1.4 | Confirm OS / Python version / hardware spec | ☐ |
| 1.5 | Identify primary integration path (new project? existing migration?) | ☐ |
| 1.6 | Schedule midpoint check-in (week 4) | ☐ |
| 1.7 | Share calendar hold for exit close-out (week 8) | ☐ |

---

## Phase 2: Environment Setup (Day 0–1)

| # | Task | Verification | Done |
|---|---|---|---|
| 2.1 | Install Python 3.10+ | `python --version` | ☐ |
| 2.2 | Install Rust nightly (if building from source) | `rustc --version` | ☐ |
| 2.3 | Install VantaDB Python wheel | `pip install vantadb-py` | ☐ |
| 2.4 | Verify wheel installs without compiler warnings | `pip show vantadb-py` | ☐ |
| 2.5 | Clone or download example repo | `git clone https://github.com/vantadb/vantadb-examples` | ☐ |
| 2.6 | Run `hello_vantadb.py` example | Script completes without error | ☐ |
| 2.7 | Confirm database files created on disk | `ls -la ./vantadb_data/` (or equivalent) | ☐ |

---

## Phase 3: Integration (Day 1–7)

| # | Task | Verification | Done |
|---|---|---|---|
| 3.1 | Participant creates a new Python file or integrates into existing project | Code compiles / imports | ☐ |
| 3.2 | Create VantaDB instance with chosen config (`distance_metric`, `dimension`) | `db = VantaDB(...)` runs without error | ☐ |
| 3.3 | Implement first `put` with a test vector and payload | `put()` returns successfully | ☐ |
| 3.4 | Implement first `search_memory` (vector-only) | Returns correct nearest neighbors | ☐ |
| 3.5 | Implement hybrid search with text query | Returns relevant results with RRF scoring | ☐ |
| 3.6 | Test durability: insert data, restart process, read back | Data persists across restarts | ☐ |
| 3.7 | Run `db.flush()` and verify fsync persistence | File timestamps update | ☐ |
| 3.8 | Run `db.rebuild_index()` with non-trivial data (>100 vectors) | Completes without error | ☐ |

---

## Phase 4: Benchmark Baseline (Week 1)

| # | Task | Verification | Done |
|---|---|---|---|
| 4.1 | Install benchmark tooling | `pip install vantadb-bench` | ☐ |
| 4.2 | Run `vantadb-bench quick` — small dataset (1K vectors, 10 queries) | Results file produced | ☐ |
| 4.3 | Run `vantadb-bench full` — large dataset (100K vectors, 100 queries) | Results file produced | ☐ |
| 4.4 | Share benchmark output files with program lead | Files received | ☐ |
| 4.5 | Submit week-1 feedback form | Form submitted | ☐ |

---

## Phase 5: Ongoing Participation (Weeks 2–8)

| # | Task | Cadence | Done |
|---|---|---|---|
| 5.1 | Submit weekly feedback form | Weekly (Friday) | ☐ |
| 5.2 | Attend midpoint check-in call | Week 4 | ☐ |
| 5.3 | Run final benchmark suite | Week 8 | ☐ |
| 5.4 | Submit exit report (1-page summary) | Week 8 | ☐ |
| 5.5 | Attend close-out call | Week 8 | ☐ |

---

## Phase 6: Post-Pilot (Week 8+)

| # | Task | Owner | Done |
|---|---|---|---|
| 6.1 | Send post-pilot survey (NPS + qualitative) | Program lead | ☐ |
| 6.2 | Process exit report and archive | Program lead | ☐ |
| 6.3 | Decide: convert to paid license, extend pilot, or conclude | Program lead + participant | ☐ |
| 6.4 | Publish testimonial / case study (with participant consent) | Program lead | ☐ |
| 6.5 | Remove participant from pilot channel / add to alumni channel | Program lead | ☐ |
| 6.6 | Archive agreement and NDA per retention policy | Program lead | ☐ |

---

## Hardware & Environment Reference

Record the participant's environment for support context.

| Field | Value |
|---|---|
| Operating System | |
| Python version | |
| CPU | |
| RAM | |
| Storage type | |
| Rust version (if applicable) | |

---

_After completing all phases, file this checklist in `./pilot-participants/[participant-name]/`._
