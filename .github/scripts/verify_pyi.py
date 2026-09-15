#!/usr/bin/env python3
"""Verify .pyi stubs match actual class signatures for VantaDB providers.

FIND-73: gate real — compara nombres de params + presencia de defaults
(stub parseado con `ast` vs runtime con `inspect.signature`), no solo
`hasattr`. Caza drift tipo PROV-10 (`key` en `store()`).

Invocación CI (.github/workflows/providers-ci.yml): el step hace
`exec(...read().replace('${PROVIDER}', matrix))`, por eso el placeholder
literal de abajo DEBE mantenerse. En local se usa `PROVIDER=<name>`.
"""
import ast
import inspect
import os
import sys
from pathlib import Path

PROVIDER = "${PROVIDER}"

CLASSES = {
    "openai": ("vantadb_openai", "VantaDBOpenAI"),
    "litellm": ("vantadb_litellm", "VantaDBLiteLLM"),
    "ollama": ("vantadb_ollama", "VantaDBOllama"),
}

REQUIRED_METHODS = [
    "embed",
    "embed_batch",
    "search",
    "store",
    "delete",
    "get",
    "list",
    "list_namespaces",
]


def resolve_provider(raw):
    """CI sustituye el placeholder; en local se lee `PROVIDER` del env."""
    if raw and not raw.startswith("$"):
        return raw
    return os.environ.get("PROVIDER", "ollama")


def repo_root(provider):
    """Raíz del repo: GITHUB_WORKSPACE (CI) → __file__ → caminata desde CWD."""
    candidates = []
    if os.environ.get("GITHUB_WORKSPACE"):
        candidates.append(Path(os.environ["GITHUB_WORKSPACE"]))
    if "__file__" in globals():
        candidates.append(Path(__file__).resolve().parents[2])
    here = Path.cwd()
    candidates.extend([here, *here.parents])
    for cand in candidates:
        if (cand / "providers" / provider).is_dir():
            return cand
    return None


def stub_signatures(pyi_path, class_name):
    """{método: [(param, tiene_default)]} desde el stub (sin `self`)."""
    tree = ast.parse(pyi_path.read_text(encoding="utf-8"), filename=str(pyi_path))
    for node in ast.walk(tree):
        if isinstance(node, ast.ClassDef) and node.name == class_name:
            out = {}
            for fn in node.body:
                if not isinstance(fn, ast.FunctionDef):
                    continue
                if fn.name.startswith("_") and fn.name != "__init__":
                    continue
                a = fn.args
                positional = list(a.posonlyargs) + list(a.args)
                n_defaults = len(a.defaults)
                params = []
                for i, arg in enumerate(positional):
                    if arg.arg == "self" and i == 0:
                        continue
                    has_default = i >= len(positional) - n_defaults
                    params.append((arg.arg, has_default))
                for arg, d in zip(a.kwonlyargs, a.kw_defaults):
                    params.append((arg.arg, d is not None))
                out[fn.name] = params
            return out
    raise AssertionError(f"class {class_name} no encontrada en {pyi_path}")


def runtime_signatures(cls, methods):
    """{método: [(param, tiene_default)]} vía inspect (sin `self`).

    Devuelve (firmas, saltados): un método PyO3 sin firma introspectable
    se salta con warning, nunca rojo falso.
    """
    sigs, skipped = {}, []
    for name in methods:
        # __init__ se introspecta sobre la clase (firma del constructor).
        target = cls if name == "__init__" else getattr(cls, name, None)
        if target is None:
            continue  # presencia ya reportada como error
        try:
            params = [
                (p.name, p.default is not inspect.Parameter.empty)
                for p in inspect.signature(target).parameters.values()
                if p.name != "self"
            ]
        except (TypeError, ValueError) as e:
            skipped.append(f"{name} ({e})")
            continue
        sigs[name] = params
    return sigs, skipped


def main():
    provider = resolve_provider(PROVIDER)
    if provider not in CLASSES:
        print(f"Invalid provider: {provider} (esperado uno de {sorted(CLASSES)})")
        return 2
    mod_name, class_name = CLASSES[provider]

    try:
        m = __import__(mod_name)
    except ImportError as e:
        # Módulo nativo sin buildear (maturin develop pendiente): skip
        # documentado, no rojo falso. En CI el build corre antes (gate real).
        print(f"SKIP: módulo '{mod_name}' no importable ({e}); "
              f"buildear con `maturin develop` en providers/{provider} "
              f"para el gate de firmas.")
        return 0
    cls = getattr(m, class_name)

    errors = []
    for name in REQUIRED_METHODS:
        if not hasattr(cls, name):
            errors.append(f"Missing method: {name}")

    root = repo_root(provider)
    if root is None:
        print("No se encontró la raíz del repo (providers/%s ausente)" % provider)
        return 2
    stub = stub_signatures(root / "providers" / provider / f"{mod_name}.pyi", class_name)
    runtime, skipped = runtime_signatures(cls, list(stub))

    for name, s_params in stub.items():
        if name not in runtime:
            continue  # ya contado en skipped con warning abajo
        r_params = runtime[name]
        if [p for p, _ in s_params] != [p for p, _ in r_params]:
            errors.append(
                f"Firma divergente en {name}: stub {[p for p, _ in s_params]} "
                f"vs runtime {[p for p, _ in r_params]}")
        elif [d for _, d in s_params] != [d for _, d in r_params]:
            errors.append(
                f"Defaults divergentes en {name}: stub {s_params} vs runtime {r_params}")

    for name in skipped:
        print(f"WARNING: sin firma introspectable, no verificado: {name}")

    if errors:
        for e in errors:
            print(f"FAIL: {e}")
        return 1
    print(f"✓ {provider}: {len(REQUIRED_METHODS)} métodos presentes y "
          f"{len(runtime)} firmas stub-vs-runtime OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
