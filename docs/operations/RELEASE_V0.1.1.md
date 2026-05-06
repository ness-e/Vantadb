# v0.1.1 Release Readiness

`v0.1.1` is a clarity and release-hygiene patch release. It does not add a new
runtime API and does not change the v0.1.x product boundary.

## Release Intent

- Keep VantaDB positioned as an embedded local-first persistent memory engine.
- Improve first-run adoption through a focused quickstart.
- Make Python packaging metadata cleaner for wheel and TestPyPI validation.
- Keep production PyPI publication, signing, and managed distribution deferred.
- Keep experimental surfaces visible but clearly marked as not part of the MVP.

## Local Validation Checklist

Run these before creating a tag or release draft:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --profile audit --workspace
cargo nextest run --profile experimental --workspace --features experimental
python -m maturin build --manifest-path ./vantadb-python/Cargo.toml --out ./target/wheels --release
```

Install the generated wheel in a clean local virtual environment:

```bash
python -m venv .release-venv
source .release-venv/bin/activate
python -m pip install --upgrade pip pytest
python -m pip install --force-reinstall "$(ls -t ./target/wheels/vantadb_py-*.whl | head -n 1)"
python -m pytest vantadb-python/tests/test_sdk.py -v
deactivate
```

Windows PowerShell equivalent:

```powershell
python -m venv .release-venv
.\.release-venv\Scripts\Activate.ps1
python -m pip install --upgrade pip pytest
$wheel = (Get-ChildItem .\target\wheels\vantadb_py-*.whl | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
python -m pip install --force-reinstall $wheel
python -m pytest vantadb-python/tests/test_sdk.py -v
deactivate
```

## GitHub Workflow Checklist

- Run `Python Wheels` manually with `publish_testpypi=false`.
- Confirm Linux, macOS, and Windows wheels build and smoke-install.
- Run `Python Wheels` manually with `publish_testpypi=true` only after
  confirming the `TEST_PYPI_API_TOKEN` secret is configured.
- Validate TestPyPI install in a clean environment.
- Do not publish to production PyPI in this release.

## Draft Release Notes

### v0.1.1

This patch release improves release readiness and external adoption for the
v0.1.x MVP.

Added:

- A 5-minute quickstart covering CLI, Python source install, vector search,
  BM25 text search, Hybrid Retrieval v1, JSONL export, and text-index audit.
- Cleaner Python package metadata for `vantadb-py` while keeping the import name
  `vantadb_py`.
- Draft public issue backlog for packaging, docs, search quality, benchmarks,
  backup/restore, and Python distribution policy.

Changed:

- Coordinated Rust crate, Python crate, and Python package metadata version to
  `0.1.1`.
- Reinforced that VantaDB remains embedded-first/local-first and that
  experimental surfaces are not part of the v0.1.x MVP.

Deferred:

- Production PyPI publication.
- Signing and installers.
- Managed cloud, enterprise, plugins, and advanced search-quality claims.

## Publication Guardrails

Do not create the `v0.1.1` tag, GitHub release, GitHub issues, TestPyPI upload,
or PyPI publication without explicit approval.
