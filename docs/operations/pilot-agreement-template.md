---
title: Pilot Program — Agreement Template
type: operations
status: active
tags: [vantadb, operations, pilot, legal, template]
last_reviewed: 2026-07-26
---

# VantaDB Pilot Program Agreement

> **Template version:** 1.0
> **Instructions:** Copy this template for each pilot participant. Fill in the `[bracketed]` fields. Both parties sign.

---

## 1. Parties

- **Company:** VantaDB ([vantadb.com](https://vantadb.com))
- **Participant:** `[Participant Name / Company Name]`
- **Contact email:** `[email]`
- **Project name:** `[project-name]`

---

## 2. Program Term

- **Start date:** `[YYYY-MM-DD]`
- **End date:** `[YYYY-MM-DD]` (default: start + 8 weeks)
- **Early termination:** Either party may terminate with 7 days written notice.

---

## 3. VantaDB Commitments

During the program term, VantaDB will:

1. **Provide software access** — grant the Participant access to VantaDB pre-release builds and the Python SDK.
2. **Provide support** — maintain a direct Slack/Discord channel with 2–4 hr response target (business hours, UTC-5/UTC-8).
3. **Prioritize feedback** — treat Participant-reported P0/P1 bugs as top priority, with initial triage within 48 hours.
4. **Communicate roadmap** — share upcoming changes that may affect Participant's integration.
5. **No license enforcement** — waive any license key checks or evaluation expiration during the program term.

---

## 4. Participant Commitments

During the program term, the Participant will:

1. **Integrate VantaDB** into one real or representative project.
2. **Submit weekly feedback** via the provided form (`docs/operations/pilot-feedback-template.md`), completing at least 6 of 8 weekly submissions.
3. **Run benchmark suite** at weeks 1, 4, and 8 using the provided `vantadb-bench` tooling.
4. **Attend calls** — one 30-min kickoff call and one 30-min midpoint review.
5. **Provide exit report** — a 1-page summary at program end covering what worked, what didn't, and missing features.
6. **Maintain confidentiality** per Section 6 of this agreement.

---

## 5. Intellectual Property

- **Participant IP:** All pre-existing Participant code, data, and project IP remains the sole property of the Participant.
- **Feedback license:** Participant grants VantaDB a perpetual, irrevocable, royalty-free license to use feedback, suggestions, and benchmark results (in anonymized form) for product development and marketing.
- **No obligation:** VantaDB is under no obligation to implement any specific feedback or feature request.

---

## 6. Confidentiality (Mutual NDA)

### 6.1 Definition

"Confidential Information" means any non-public information disclosed by one party to the other, including but not limited to:

- Pre-release software, APIs, and documentation
- Benchmark results and performance data
- Roadmap and feature plans
- Business processes and strategy

### 6.2 Obligations

Each party agrees to:

1. Use Confidential Information **only** to evaluate or participate in the Pilot Program.
2. Not disclose Confidential Information to any third party without the disclosing party's written consent.
3. Protect Confidential Information using at least the same degree of care used for its own confidential information, but no less than reasonable care.
4. Return or destroy all Confidential Information within 30 days of program termination upon request.

### 6.3 Exclusions

Confidential Information does not include information that:

1. Is or becomes publicly available without breach of this agreement.
2. Was rightfully in the receiving party's possession before disclosure.
3. Is independently developed by the receiving party without use of Confidential Information.
4. Is required to be disclosed by law or regulation (with advance notice to the disclosing party).

### 6.4 Duration

Confidentiality obligations survive the program term for **2 years** for performance data and benchmarks, and **perpetually** for trade secrets.

---

## 7. Warranties & Disclaimers

**THE SOFTWARE IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND.** VantaDB is pre-1.0 software under active development. VantaDB makes no warranties regarding fitness for a particular purpose, merchantability, or non-infringement.

---

## 8. Limitation of Liability

Neither party shall be liable for indirect, incidental, or consequential damages arising from this agreement. VantaDB's total liability is limited to **$100 USD**.

---

## 9. Governing Law

This agreement is governed by the laws of `[State/Country]`, excluding conflict-of-law provisions.

---

## 10. Signatures

```
VantaDB:
Name: _________________________
Title: _________________________
Date: _________________________
Signature: _____________________

Participant:
Name: _________________________
Title: _________________________
Date: _________________________
Project: ______________________
Signature: _____________________
```
