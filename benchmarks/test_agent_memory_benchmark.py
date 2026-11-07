"""Contract checks for the synthetic trace, percentile and fusion calculations."""

import collections
import json
import math
import tempfile
import threading
import time
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

import agent_memory_vs_sqlite_lancedb as bench


class WorkloadTests(unittest.TestCase):
    def test_trace_is_reproducible_and_exactly_twenty_percent_writes(self):
        trace = bench.build_workload(1000, 23)
        self.assertEqual(
            bench.encode_workload(trace),
            bench.encode_workload(bench.build_workload(1000, 23)),
        )
        counts = collections.Counter(op["kind"] for op in trace["operations"])
        self.assertEqual(counts, {"write": 200, "read": 800})
        self.assertEqual(len(trace["records"]), bench.INITIAL_RECORDS + 200)
        available = set(range(1, bench.INITIAL_RECORDS + 1))
        witnesses = 0
        for operation in trace["operations"]:
            identifier = operation["id"]
            if operation["kind"] == "write":
                self.assertNotIn(identifier, available)
                available.add(identifier)
            else:
                self.assertIn(identifier, available)
                witnesses += operation["read_your_write"]
        self.assertEqual(witnesses, 200)
        for record in trace["records"]:
            self.assertEqual(len(record["vector"]), 384)
            self.assertTrue(math.isclose(sum(v * v for v in record["vector"]), 1.0))
            self.assertIn(record["session_id"], range(bench.SESSIONS))
            self.assertIn(record["role"], ("user", "assistant"))

    def test_remainder_does_not_add_an_extra_write(self):
        trace = bench.build_workload(1001, 23)
        self.assertEqual(sum(op["kind"] == "write" for op in trace["operations"]), 200)
        self.assertEqual(len(trace["operations"]), 1001)

    def test_nearest_rank_percentiles_and_missing_samples(self):
        self.assertEqual(bench.percentile([4, 1, 3, 2], 50), 2)
        self.assertEqual(bench.percentile(list(range(1, 101)), 95), 95)
        self.assertEqual(bench.percentile(list(range(1, 101)), 99), 99)
        self.assertIsNone(bench.percentile([], 50))

    def test_rrf_uses_both_branches_and_stable_ties(self):
        self.assertEqual(bench.reciprocal_rank_fusion([[1, 2], [2, 3]]), [2, 1, 3])
        self.assertEqual(bench.reciprocal_rank_fusion([[3], [1]], limit=1), [1])
        self.assertEqual(bench.reciprocal_rank_fusion([]), [])

    def test_lock_classifier_does_not_match_unrelated_messages(self):
        self.assertTrue(bench.is_lock_error(RuntimeError("failed to acquire lock")))
        self.assertFalse(bench.is_lock_error(RuntimeError("invalid FTS block size")))
        self.assertFalse(bench.is_lock_error(ValueError("shape mismatch")))

    def test_cli_turn_bounds(self):
        self.assertEqual(bench.turns_argument("1000"), 1000)
        self.assertEqual(bench.turns_argument("10000"), 10000)
        for value in ("999", "10001"):
            with self.assertRaises(bench.argparse.ArgumentTypeError):
                bench.turns_argument(value)

    def test_optional_setup_failure_is_explicitly_skipped(self):
        fake_psutil = SimpleNamespace(
            Process=lambda: SimpleNamespace(
                memory_info=lambda: SimpleNamespace(rss=1000)
            )
        )
        trace = {"records": [], "initial_records": 0, "turns": 1000}
        for exception in (
            bench.MissingDependency("Optional dependency not installed"),
            RuntimeError("Optional extension could not be loaded"),
        ):
            with tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / "trace.json"
                path.write_text(json.dumps(trace), "utf-8")
                with (
                    mock.patch.object(
                        bench, "require_module", return_value=fake_psutil
                    ),
                    mock.patch.dict(
                        bench.ADAPTERS, {"sqlite-vec": mock.Mock(side_effect=exception)}
                    ),
                ):
                    result = bench.run_worker("sqlite-vec", path)
                self.assertEqual(result["status"], "skipped")
                self.assertIn(str(exception), result["reason"])
                self.assertNotIn("write", result)
                self.assertNotIn("hybrid_read", result)

    def test_missing_transitive_dependency_is_not_reported_as_top_level_absence(self):
        failure = ModuleNotFoundError("missing transitive module", name="numpy")
        with (
            mock.patch.object(bench.importlib, "import_module", side_effect=failure),
            self.assertRaises(ModuleNotFoundError),
        ):
            bench.require_module("lancedb", "lancedb")

    def test_contention_probe_uses_distinct_keys_and_observes_overlap(self):
        class FixtureAdapter:
            def __init__(self):
                self.ids = set()
                self.mutex = threading.Lock()

            def write(self, row):
                time.sleep(0.001)
                if row["id"] % 3 == 0:
                    error = bench.sqlite3.OperationalError("database is locked")
                    error.sqlite_errorcode = bench.sqlite3.SQLITE_BUSY
                    raise error
                with self.mutex:
                    if row["id"] in self.ids:
                        raise AssertionError("duplicate fixture key")
                    self.ids.add(row["id"])

        adapter = FixtureAdapter()
        specification = {"workers": 4, "records": [{"id": i} for i in range(1, 33)]}
        with self.assertLogs(bench.logger, level="ERROR"):
            result = bench.run_contention_probe(adapter, specification)
        self.assertEqual(result["status"], "completed")
        self.assertEqual(result["attempted_writes"], 32)
        self.assertEqual(result["successful_writes"], 22)
        self.assertEqual(result["lock_errors"], 10)
        self.assertEqual(result["observed_lock_error_rate"], 10 / 32)
        self.assertEqual(result["other_write_errors"], 0)
        self.assertTrue(result["overlap_observed"])
        self.assertEqual(result["lifecycle_errors"], [])
        self.assertEqual(len(adapter.ids), 22)

    def test_contention_records_are_separate_from_the_eighty_twenty_trace(self):
        trace = bench.build_workload(1000, 23)
        primary_ids = {row["id"] for row in trace["records"]}
        probe = trace["contention_probe"]
        probe_ids = {row["id"] for row in probe["records"]}
        self.assertEqual(len(probe_ids), 100)
        self.assertFalse(primary_ids & probe_ids)
        self.assertTrue(all(len(row["vector"]) == 384 for row in probe["records"]))
        self.assertEqual(probe["workers"], 4)
        self.assertEqual(probe["writes_per_worker"], 25)


if __name__ == "__main__":
    unittest.main()
