"""PY-03: import identity contract.

Canonical import is ``import vantadb`` (must be silent). The legacy alias
``import vantadb_py`` must emit a DeprecationWarning that points users at the
canonical name and states the removal timeline. Each check runs in a fresh
interpreter because module-import warnings fire only on first import.

Timeline: the alias is removed in **1.0.0**, not 0.6.0. The original target was
unreachable by construction — this deprecation shipped *in* 0.6.0
(`CHANGELOG.md` `## [0.6.0]`), violating VERSIONING's "at least one MINOR".
1.0.0 is a **deferral** pending owner sign-off, not a VERSIONING constraint: a
0.x MINOR removal is legal (`VERSIONING.md` §Pre-1.0) but requires renaming the
native module. `docs/api/DEPRECATIONS.md` registers the entry and its target.
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
        if mode == "functional":
            import vantadb

dep = [w for w in caught if issubclass(w.category, DeprecationWarning)]
msg = str(dep[0].message) if dep else ""

if mode == "canonical":
    assert not dep, f"import vantadb must not emit DeprecationWarning, got: {msg}"
elif mode == "legacy":
    assert dep, "import vantadb_py must emit DeprecationWarning"
    assert "import vantadb" in msg, f"warning must point to 'import vantadb': {msg!r}"
    assert "1.0.0" in msg, f"warning must state removal timeline: {msg!r}"
else:  # functional — the deprecated name must keep serving the REAL objects
    # `vantadb` does `from vantadb_py import *`, so this identity only proves the
    # shim was not emptied. The native check proves `Client` is the compiled
    # class, not a Python re-implementation.
    import vantadb_py.vantadb_py as native

    assert vantadb.Client is vantadb_py.Client, (
        "the canonical shim must re-export the alias' objects, not copies"
    )
    assert vantadb_py.Client is native.Client, (
        "the package must re-export the compiled Client, not define its own"
    )
    assert vantadb.__version__ == vantadb_py.__version__, (
        "legacy alias and canonical name must report the same version"
    )

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
        check=False,  # returncode asserted explicitly below
    )
    assert proc.returncode == 0 and "OK" in proc.stdout, (
        f"[{mode}] failed\nstdout: {proc.stdout}\nstderr: {proc.stderr}"
    )


def test_canonical_import_vantadb_is_silent():
    """import vantadb must not trigger any DeprecationWarning."""
    _run_fresh_interpreter("canonical")


def test_legacy_alias_vantadb_py_warns_with_timeline():
    """import vantadb_py must warn, point at `import vantadb`, give the real target."""
    _run_fresh_interpreter("legacy")


def test_legacy_alias_stays_functional_until_removal_target():
    """The deprecated name keeps serving the compiled objects until removal.

    `vantadb.Client is vantadb_py.Client` only guards against an emptied shim
    (`vantadb/__init__.py` does `from vantadb_py import *`); the native check
    (`vantadb_py.Client is vantadb_py.vantadb_py.Client`) is the one that would
    fail if the package started defining a Python copy of the compiled class.
    """
    _run_fresh_interpreter("functional")
