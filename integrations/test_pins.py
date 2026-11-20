"""FIND-84 + FIND-582 M-15: pins con upper-bound por adapter y pin de `vantadb-py` gateado.

Lee los 9 `integrations/*/pyproject.toml` y exige dos cosas:

1. Cada dependencia de framework (toda dep que no sea `vantadb-py`) declara un
   techo `<X` (PEP 440). `vantadb-py` queda exenta de este gate porque se
   verifica aparte (ver 2).
2. M-15: el pin de `vantadb-py` sí está gateado — cada adapter debe declarar
   piso `>=` y techo `<` explícitos. El techo canónico es `<0.10.0` (no
   `<0.9.0` como decía la versión anterior de este docstring) y los pisos
   permitidos son `0.6.1` (7 adapters) y `0.5.0` (los providers
   `ollama`/`openai`, compatibles con una minor anterior). Cambiar un piso es
   una decisión deliberada: actualiza `EXPECTED_VANTADB_FLOORS`.

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

# M-15: canonical ceiling + allowed floors for the `vantadb-py` pin.
VANTADB_CEILING = "<0.10.0"
EXPECTED_VANTADB_FLOORS = {
    "crewai": "0.6.1",
    "dspy": "0.6.1",
    "haystack": "0.6.1",
    "langchain": "0.6.1",
    "letta": "0.6.1",
    "llamaindex": "0.6.1",
    "mem0": "0.6.1",
    "ollama": "0.5.0",
    "openai": "0.5.0",
}


def _deps(adapter: str) -> list[str]:
    data = tomllib.loads((ROOT / adapter / "pyproject.toml").read_text(encoding="utf-8"))
    return data["project"]["dependencies"]


def _framework_deps(adapter: str) -> list[str]:
    return [d for d in _deps(adapter) if not d.startswith("vantadb-py")]


def _vantadb_req(adapter: str) -> Requirement:
    pins = [d for d in _deps(adapter) if d.startswith("vantadb-py")]
    assert len(pins) == 1, f"{adapter}: expected exactly one vantadb-py dep, got {pins}"
    return Requirement(pins[0])


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


@pytest.mark.parametrize("adapter", EXPECTED)
def test_vantadb_pin_has_floor_and_ceiling(adapter: str):
    """M-15: the `vantadb-py` pin is gated too — explicit floor + canonical
    ceiling on every adapter (previously only framework ceilings were gated
    and the `>=0.5.0` provider floor went unchecked)."""
    req = _vantadb_req(adapter)
    specifiers = {(s.operator, s.version) for s in req.specifier}
    floors = {v for op, v in specifiers if op in (">=", ">")}
    ceilings = {v for op, v in specifiers if op in ("<", "<=")}
    assert floors, f"{adapter}: vantadb-py sin piso (>=): {req}"
    assert ceilings, f"{adapter}: vantadb-py sin techo (<): {req}"
    assert ("<", VANTADB_CEILING.lstrip("<")) in specifiers, (
        f"{adapter}: techo de vantadb-py {req} != canónico {VANTADB_CEILING}"
    )
    expected_floor = EXPECTED_VANTADB_FLOORS[adapter]
    assert expected_floor in floors, (
        f"{adapter}: piso de vantadb-py {sorted(floors)} != esperado >={expected_floor}"
    )
