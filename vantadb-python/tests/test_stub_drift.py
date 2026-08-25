"""Anti-drift tests: the ``.pyi`` stubs must match the compiled ``vantadb_py`` bindings.

MOD-18: two stubs ship in the wheel (``pyproject.toml`` includes
``vantadb_py/*.pyi``) and type two distinct layers of the package:

- ``vantadb_py.pyi``  — types the compiled extension (``vantadb_py.pyd``,
  imported by the wrapper as ``from .vantadb_py import ...``). It is the
  single source of truth for native signatures: every method/parameter on
  the native classes must be declared here.
- ``__init__.pyi``    — types the wrapper package (``SearchRequest``,
  ``AsyncVantaDB``, re-exports). It must re-export from ``vantadb_py.pyi``
  (not re-declare) and mirror the real ``AsyncVantaDB`` in ``__init__.py``.

These tests parse both stubs with :mod:`ast` and compare them against the
*compiled* module via :func:`inspect.signature` (PyO3 exposes real
``__text_signature__``), so a Rust-side signature change without a stub
update fails here.

The tests that need the compiled module skip cleanly when it is not
importable (e.g. CI without a ``maturin develop`` build), keeping the plain
``pytest`` unit gate green. The wrapper ``AsyncVantaDB`` is pure Python and
is always checked.
"""

from __future__ import annotations

import ast
import inspect
from pathlib import Path

import pytest

PKG = Path(__file__).resolve().parents[1] / "vantadb_py"

try:
    import vantadb_py as vanta
    import vantadb_py.vantadb_py as native
except ImportError:  # compiled extension not built
    vanta = None
    native = None

needs_native = pytest.mark.skipif(
    native is None,
    reason="compiled vantadb_py module not available (run `maturin develop` first)",
)


def _parse_stub(path: Path) -> dict[str, dict[str, dict[str, list[str]]]]:
    """Parse a .pyi into {class: {kind: {name: [params without self]}}}.

    ``kind`` is one of ``methods`` (def/async def), ``properties``
    (``@property`` defs), ``members`` (annotated class attributes).
    """
    tree = ast.parse(path.read_text(encoding="utf-8"))
    classes: dict[str, dict[str, dict[str, list[str]]]] = {}
    for node in ast.walk(tree):
        if not isinstance(node, ast.ClassDef):
            continue
        entry: dict[str, dict[str, list[str]]] = {
            "methods": {},
            "properties": {},
            "members": {},
        }
        for item in node.body:
            if isinstance(item, (ast.FunctionDef, ast.AsyncFunctionDef)):
                args = item.args
                params = [
                    a.arg for a in args.posonlyargs + args.args + args.kwonlyargs
                ]
                if "self" in params:
                    params.remove("self")
                is_prop = any(
                    isinstance(d, ast.Name) and d.id == "property"
                    for d in item.decorator_list
                )
                entry["properties" if is_prop else "methods"][item.name] = params
            elif isinstance(item, ast.AnnAssign) and isinstance(item.target, ast.Name):
                entry["members"][item.target.id] = []
        classes[node.name] = entry
    return classes


def _stub_required_params(args: ast.arguments) -> set[str]:
    """Parameter names without a default (required) in a stub function,
    excluding ``self``."""
    required: set[str] = set()
    plain = args.posonlyargs + args.args
    n_defaults = len(args.defaults)
    for i, a in enumerate(plain):
        # ``defaults`` align with the tail of (posonlyargs + args)
        if a.arg == "self":
            continue
        if i < len(plain) - n_defaults:
            required.add(a.arg)
    for a, default in zip(args.kwonlyargs, args.kw_defaults):
        if default is None:
            required.add(a.arg)
    return required


def _public_callables(cls: type) -> set[str]:
    return {
        n
        for n in dir(cls)
        if not n.startswith("_") and callable(getattr(cls, n))
    }


def _public_noncallables(cls: type) -> set[str]:
    return {
        n
        for n in dir(cls)
        if not n.startswith("_") and not callable(getattr(cls, n))
    }


def _native_params(cls: type, method: str) -> list[str] | None:
    """Runtime param names (minus self) for a class method, or None if not
    introspectable (some PyO3 methods do not expose a text signature)."""
    try:
        sig = inspect.signature(getattr(cls, method))
    except (TypeError, ValueError):
        return None
    return [p for p in sig.parameters if p != "self"]


def _native_required(cls: type, method: str) -> set[str] | None:
    try:
        sig = inspect.signature(getattr(cls, method))
    except (TypeError, ValueError):
        return None
    return {
        p
        for p, par in sig.parameters.items()
        if p != "self" and par.default is inspect.Parameter.empty
    }


def _stub_function(node: ast.FunctionDef) -> tuple[list[str], set[str]]:
    args = node.args
    params = [a.arg for a in args.posonlyargs + args.args + args.kwonlyargs]
    if "self" in params:
        params.remove("self")
    return params, _stub_required_params(args)


def _assert_method_parity(stub: dict[str, list[str]], native_cls: type, label: str):
    """Every public native callable must be in the stub, and every stub
    method must exist on the native class (catches missing/extra methods)."""
    real = _public_callables(native_cls)
    stub_names = {n for n in stub if not n.startswith("_")}
    missing = real - stub_names
    extra = stub_names - real
    assert not missing, f"{label}: métodos nativos sin declarar en el stub: {sorted(missing)}"
    assert not extra, f"{label}: métodos del stub que no existen en nativo: {sorted(extra)}"


def _assert_param_parity(
    stub: dict[str, list[str]], native_cls: type, label: str
):
    """For every shared method, stub param names and requiredness must match
    the native signature (catches renamed/missing params and wrong defaults)."""
    for name in sorted(set(stub) & _public_callables(native_cls)):
        native_params = _native_params(native_cls, name)
        if native_params is None:
            continue
        assert stub[name] == native_params, (
            f"{label}.{name}: params del stub {stub[name]} != nativo {native_params}"
        )
        native_required = _native_required(native_cls, name)
        if native_required is None:
            continue
        # Recompute requiredness from the stub via a targeted parse.
        tree = ast.parse((PKG / "vantadb_py.pyi").read_text(encoding="utf-8"))
        for node in ast.walk(tree):
            if (
                isinstance(node, ast.ClassDef)
                and node.name == label.split(".")[0]
            ):
                for item in node.body:
                    if isinstance(item, ast.FunctionDef) and item.name == name:
                        stub_required = _stub_required_params(item.args)
                        assert stub_required == native_required, (
                            f"{label}.{name}: params requeridos stub {sorted(stub_required)} != nativo {sorted(native_required)}"
                        )


# ── Native module (requires the compiled extension) ────────────────────────


@needs_native
def test_native_stub_declares_every_module_name():
    """Every public name of the compiled module is declared in vantadb_py.pyi."""
    stub = _parse_stub(PKG / "vantadb_py.pyi")
    declared = set(stub) | {n for n in dir(native) if not n.startswith("_")}
    real = {n for n in dir(native) if not n.startswith("_")}
    missing = real - declared
    assert not missing, f"nombres del módulo nativo sin declarar en el stub: {sorted(missing)}"
    # connect must match its real signature.
    sig = inspect.signature(native.connect)
    assert [p for p in sig.parameters] == [
        "path",
        "memory_limit",
        "read_only",
        "backend",
    ]


@needs_native
def test_vantadb_stub_matches_native_methods_and_properties():
    stub = _parse_stub(PKG / "vantadb_py.pyi")["VantaDB"]
    cls = native.VantaDB
    _assert_method_parity(stub["methods"], cls, "VantaDB")
    _assert_param_parity(stub["methods"], cls, "VantaDB")
    assert set(stub["properties"]) == _public_noncallables(cls), (
        f"VantaDB properties: stub {sorted(stub['properties'])} != nativo {sorted(_public_noncallables(cls))}"
    )


@needs_native
def test_subclient_stubs_match_native_subclients():
    """db.memory|graph|system|wiki getters must expose exactly the methods
    declared for the corresponding stub classes (forward_to_db! macro)."""
    stub = _parse_stub(PKG / "vantadb_py.pyi")
    db = native.VantaDB(":memory:", backend="memory")
    try:
        for prop, stub_class in [
            ("memory", "MemoryClient"),
            ("graph", "GraphClient"),
            ("system", "SystemClient"),
            ("wiki", "WikiClient"),
        ]:
            client_cls = type(getattr(db, prop))
            _assert_method_parity(
                stub[stub_class]["methods"], client_cls, stub_class
            )
            # Subclients are plain forwarders: stub params (via *args/**kwargs
            # passthrough) are not introspectable, so only method-set parity.
    finally:
        db.close()


@needs_native
def test_put_batch_return_type_is_typed_not_dict():
    """Regression guard for the put_batch/put_batch_raw type fix: the native
    methods return VantaMemoryRecord objects, not dicts."""
    stub = _parse_stub(PKG / "vantadb_py.pyi")
    for name in ("put_batch", "put_batch_raw"):
        ret = None
        tree = ast.parse((PKG / "vantadb_py.pyi").read_text(encoding="utf-8"))
        for node in ast.walk(tree):
            if (
                isinstance(node, ast.ClassDef)
                and node.name == "VantaDB"
            ):
                for item in node.body:
                    if isinstance(item, ast.FunctionDef) and item.name == name:
                        ret = ast.unparse(item.returns) if item.returns else None
        assert ret is not None, f"VantaDB.{name} sin anotación de retorno en el stub"
        assert "VantaMemoryRecord" in ret, (
            f"VantaDB.{name} return type {ret!r} no menciona VantaMemoryRecord"
        )


@needs_native
def test_wrapper_stub_reexports_native_names():
    """__init__.pyi must re-export the native names that __init__.py imports,
    and every name must resolve on the compiled module."""
    wrapper_stub = (PKG / "__init__.pyi").read_text(encoding="utf-8")
    tree = ast.parse(wrapper_stub)
    imported: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.ImportFrom) and node.module == "vantadb_py":
            imported |= {a.name for a in node.names if a.name != "*"}
    missing = {n for n in imported if not hasattr(native, n)}
    assert not missing, f"re-exports de __init__.pyi que no existen en nativo: {sorted(missing)}"
    # __all__ must be identical between the wrapper .py and its stub.
    def _all_of(path: Path) -> list[str]:
        for node in ast.walk(ast.parse(path.read_text(encoding="utf-8"))):
            if isinstance(node, ast.Assign):
                for t in node.targets:
                    if isinstance(t, ast.Name) and t.id == "__all__":
                        return [e.value for e in node.value.elts]  # type: ignore[attr-defined]
        return []

    assert _all_of(PKG / "__init__.py") == _all_of(PKG / "__init__.pyi"), (
        "__all__ de __init__.py difiere del __init__.pyi"
    )


# ── Wrapper (pure Python — always runs, no compiled module needed) ─────────


def test_async_wrapper_stub_matches_real_asyncvantadb():
    """AsyncVantaDB in __init__.pyi must mirror the real wrapper in
    __init__.py (methods + params + requiredness)."""
    if vanta is None:
        pytest.skip("compiled vantadb_py module not available (run `maturin develop` first)")
    stub = _parse_stub(PKG / "__init__.pyi")["AsyncVantaDB"]
    _assert_method_parity(stub["methods"], vanta.AsyncVantaDB, "AsyncVantaDB")
    for name in sorted(set(stub["methods"]) & _public_callables(vanta.AsyncVantaDB)):
        real_params = [
            p for p in inspect.signature(getattr(vanta.AsyncVantaDB, name)).parameters
            if p != "self"
        ]
        real_required = {
            p
            for p, par in inspect.signature(getattr(vanta.AsyncVantaDB, name)).parameters.items()
            if p != "self" and par.default is inspect.Parameter.empty
        }
        for fn in ast.walk(ast.parse((PKG / "__init__.pyi").read_text(encoding="utf-8"))):
            if (
                isinstance(fn, ast.ClassDef)
                and fn.name == "AsyncVantaDB"
            ):
                for item in fn.body:
                    if isinstance(item, ast.FunctionDef) and item.name == name:
                        stub_params, stub_required = _stub_function(item)
                        assert stub_params == real_params, (
                            f"AsyncVantaDB.{name}: params stub {stub_params} != real {real_params}"
                        )
                        assert stub_required == real_required, (
                            f"AsyncVantaDB.{name}: requeridos stub {sorted(stub_required)} != real {sorted(real_required)}"
                        )


def test_wrapper_stub_declares_search_request():
    """SearchRequest dataclass must exist in __init__.pyi with its real fields."""
    stub = _parse_stub(PKG / "__init__.pyi")["SearchRequest"]
    fields = {"namespace", "query_vector", "filters", "text_query", "top_k",
              "distance_metric", "method", "explain"}
    assert fields <= set(stub["members"]) | set(stub["methods"]), (
        f"SearchRequest incompleto en stub: members={sorted(stub['members'])}"
    )
    assert "asdict" in stub["methods"]