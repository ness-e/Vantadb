"""FIND-84: pins con upper-bound por adapter (contrato: upper-bounds en 7/9).

Lee los 9 `integrations/*/pyproject.toml` y exige que cada dependencia de
framework (toda dep que no sea `vantadb-py`) declare un techo `<X`
(PEP 440). `vantadb-py>=0.5.0,<0.6.0` ya trae techo y queda exenta.
Corre offline: solo parsea TOML, sin resolver ni descargar nada.
"""
from __future__ import annotations

import tomllib
from pathlib import Path

import pytest
from packaging.requirements import Requirement

ROOT = Path(__file__).resolve().parent
ADAPTERS = sorted(p.name for p in ROOT.iterdir() if (p / "pyproject.toml").is_file())
# 9 adapters esperados (vantadb_shared no es distribuible: va force-include
# dentro de los wheels ollama/openai y no tiene pyproject propio).
EXPECTED = ["crewai", "dspy", "haystack", "langchain", "letta", "llamaindex", "mem0", "ollama", "openai"]


def _framework_deps(adapter: str) -> list[str]:
    data = tomllib.loads((ROOT / adapter / "pyproject.toml").read_text(encoding="utf-8"))
    deps = data["project"]["dependencies"]
    return [d for d in deps if not d.startswith("vantadb-py")]


def test_nine_adapters_present():
    assert ADAPTERS == EXPECTED, f"matriz cambió: {ADAPTERS}"


@pytest.mark.parametrize("adapter", EXPECTED)
def test_framework_deps_have_upper_bound(adapter: str):
    missing = []
    for dep in _framework_deps(adapter):
        req = Requirement(dep)
        if not any(op in ("<", "<=", "==", "~=") for op in {s.operator for s in req.specifier}):
            missing.append(dep)
    assert not missing, f"{adapter}: sin upper-bound: {missing}"
