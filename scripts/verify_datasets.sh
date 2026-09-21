#!/usr/bin/env bash
# verify_datasets.sh — Pre-test gate for heavy-certification workflow.
#
# Verifies the 5 canonical datasets required by tests/certification/* and
# tests/benchmark_datasets.rs. Exits 0 when all present, 1 when any is missing.
#
# Tested in TBH-01 (2026-08-30). See .opencode/skills/campaign-executor/tasks/TBH-01.md.
#
# Usage:
#   bash scripts/verify_datasets.sh         # human-readable table, exit 0/1
#   bash scripts/verify_datasets.sh --json   # machine-readable JSON
#
# Whitelist rationale: tests with #[ignore] that are NOT dataset-related
# (Miri/FFI/croaring ×18 + tests with non-dataset ignore reasons) are not in
# scope here — the gate only protects against silent dataset-skips.

set -euo pipefail

JSON_MODE=0
if [[ "${1:-}" == "--json" ]]; then
    JSON_MODE=1
fi

# Dataset definitions: name | source-script | paths...
DATASETS=(
    "SIFT-1M|dev-tools/scripts/download_sift.py|datasets/sift/sift_base.fvecs|datasets/sift/sift_query.fvecs|datasets/sift/sift_groundtruth.ivecs"
    "GloVe-100|scripts/download_benchmark_datasets.sh|data/benchmark/glove.6B.100d.txt"
    "GloVe-300|scripts/download_benchmark_datasets.sh|data/benchmark/glove.6B.300d.txt"
    "SIFT-128 euclidean subset|scripts/download_ground_truth.py|data/benchmark/sift-128/train.f32|data/benchmark/sift-128/test.f32|data/benchmark/sift-128/test_neighbors.u64|data/benchmark/sift-128/meta.json"
    "GloVe-100 angular subset|scripts/download_ground_truth.py|data/benchmark/glove-100-angular/train.f32|data/benchmark/glove-100-angular/test.f32|data/benchmark/glove-100-angular/test_neighbors.u64|data/benchmark/glove-100-angular/meta.json"
)

declare -a RESULTS_NAME RESULTS_STATUS RESULTS_ACTION

MISSING=0
for entry in "${DATASETS[@]}"; do
    IFS='|' read -r name source p1 p2 p3 p4 <<< "$entry"
    all_ok=1
    missing_paths=""
    for p in "$p1" "$p2" "$p3" "$p4"; do
        [[ -z "$p" ]] && continue
        if [[ ! -e "$p" ]]; then
            all_ok=0
            if [[ -n "$missing_paths" ]]; then
                missing_paths="${missing_paths}, "
            fi
            missing_paths="${missing_paths}${p}"
        fi
    done

    if [[ $all_ok -eq 1 ]]; then
        RESULTS_NAME+=("$name")
        RESULTS_STATUS+=("OK")
        RESULTS_ACTION+=("none")
    else
        RESULTS_NAME+=("$name")
        RESULTS_STATUS+=("MISSING")
        RESULTS_ACTION+=("run ${source} (missing: ${missing_paths})")
        MISSING=$((MISSING + 1))
    fi
done

# Output
if [[ $JSON_MODE -eq 1 ]]; then
    printf '{"missing": %d, "datasets": [' "$MISSING"
    first=1
    for i in "${!RESULTS_NAME[@]}"; do
        if [[ $first -eq 0 ]]; then printf ','; fi
        first=0
        printf '{"name":"%s","status":"%s","action":"%s"}' \
            "${RESULTS_NAME[$i]}" "${RESULTS_STATUS[$i]}" "${RESULTS_ACTION[$i]}"
    done
    printf ']}\n'
else
    echo "=== verify_datasets: $MISSING missing / ${#DATASETS[@]} expected ==="
    printf "| %-26s | %-10s | %s\n" "dataset" "status" "action"
    printf "|%s|%s|%s\n" "----------------------------" "------------" "----------------------------------------"
    for i in "${!RESULTS_NAME[@]}"; do
        printf "| %-26s | %-10s | %s\n" \
            "${RESULTS_NAME[$i]}" "${RESULTS_STATUS[$i]}" "${RESULTS_ACTION[$i]}"
    done
fi

if [[ $MISSING -gt 0 ]]; then
    exit 1
fi
exit 0
