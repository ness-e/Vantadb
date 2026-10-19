#!/usr/bin/env python
"""Clean-venv smoke test for a built VantaDB provider wheel (PROV-12).

Instantiates the provider class, stores a record with a fake embedding and
searches it back. ``embed()`` is never called (no network, no API key needed
beyond the constructor's placeholder string).

Used by ``release-providers.yml`` (build smoke, TestPyPI verify, PyPI verify)
and runnable locally against a wheel built with ``maturin build``.

Usage:
    python provider_wheel_smoke.py <openai|ollama|litellm>

Exit 0 prints ``SMOKE OK``; any failure raises (non-zero exit).
"""
from __future__ import annotations

import importlib
import sys
import tempfile

PROVIDERS = {
    "openai": {
        "module": "vantadb_openai",
        "cls": "VantaDBOpenAI",
        "kwargs": {"api_key": "test-key"},
        "namespace": "openai_store",
    },
    "ollama": {
        "module": "vantadb_ollama",
        "cls": "VantaDBOllama",
        "kwargs": {},
        "namespace": "ollama_store",
    },
    "litellm": {
        "module": "vantadb_litellm",
        "cls": "VantaDBLiteLLM",
        "kwargs": {"api_key": "test-key"},
        "namespace": "litellm_store",
    },
}


def main() -> int:
    if len(sys.argv) != 2 or sys.argv[1] not in PROVIDERS:
        print(f"usage: {sys.argv[0]} <{'|'.join(PROVIDERS)}>", file=sys.stderr)
        return 2
    spec = PROVIDERS[sys.argv[1]]
    mod = importlib.import_module(spec["module"])
    cls = getattr(mod, spec["cls"])

    with tempfile.TemporaryDirectory(prefix="vantadb-provider-smoke-") as tmp:
        store = cls(tmp, **spec["kwargs"])
        emb = [0.1] * 128
        rid = store.store("smoke text", emb, {"src": "smoke"})
        assert ":" in rid, f"unexpected record id: {rid!r}"

        results = store.search(spec["namespace"], emb, top_k=3)
        assert any(r["text"] == "smoke text" for r in results), (
            f"stored record not found in search results: {results!r}"
        )

        # list() round-trip (canonical contract, ADR-0033 D2/D3)
        listed = store.list(spec["namespace"], limit=10)
        assert any(r["text"] == "smoke text" for r in listed["records"]), listed

    version = getattr(mod, "__version__", "?")
    print(f"SMOKE OK: {spec['module']} v{version} record={rid}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
