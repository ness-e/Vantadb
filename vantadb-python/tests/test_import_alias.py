"""PY-03: import identity contract.

Canonical import is ``import vantadb`` (must be silent). The legacy alias
``import vantadb_py`` must emit a DeprecationWarning that points users at the
canonical name and states the removal timeline. Each check runs in a fresh
interpreter because module-import warnings fire only on first import.
"""

import subprocess
import sys

_SCRIPT = r"""
import sys, warnings

mode = sys.argv[1]
with warnings.catch_warnings(record=True) as caught:
    warnings.simplefilter("always")
    if mode == "canonical":
        import vantadb  # noqa: F401
    else:
        import vantadb_py  # noqa: F401

dep = [w for w in caught if issubclass(w.category, DeprecationWarning)]
msg = str(dep[0].message) if dep else ""

if mode == "canonical":
    assert not dep, f"import vantadb must not emit DeprecationWarning, got: {msg}"
else:
    assert dep, "import vantadb_py must emit DeprecationWarning"
    assert "import vantadb" in msg, f"warning must point to 'import vantadb': {msg!r}"
    assert "0.6.0" in msg, f"warning must state removal timeline: {msg!r}"

print("OK")
"""


def _run_fresh_interpreter(mode: str) -> None:
    proc = subprocess.run(
        [sys.executable, "-c", _SCRIPT, mode],
        capture_output=True,
        text=True,
        timeout=120,
        encoding="utf-8",
        errors="replace",
    )
    assert proc.returncode == 0 and "OK" in proc.stdout, (
        f"[{mode}] failed\nstdout: {proc.stdout}\nstderr: {proc.stderr}"
    )


def test_canonical_import_vantadb_is_silent():
    """import vantadb must not trigger any DeprecationWarning."""
    _run_fresh_interpreter("canonical")


def test_legacy_alias_vantadb_py_warns_with_timeline():
    """import vantadb_py must warn, point at `import vantadb`, give timeline."""
    _run_fresh_interpreter("legacy")
