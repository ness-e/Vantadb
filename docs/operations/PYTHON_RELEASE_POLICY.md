# Python Release Policy & Wheel Engineering

This document outlines the standard release policy and engineering practices for the VantaDB Python SDK (`vantadb-python`).

## Release Scope

The repository is prepared for Python wheel publishing, but this task does not
publish to PyPI. Production publication requires a signed `vMAJOR.MINOR.PATCH`
tag and a configured PyPI Trusted Publisher or equivalent repository token.

## PEP-440 Version Compliance

All releases of `vantadb_py` must strictly adhere to the [PEP-440 specification](https://peps.python.org/pep-0440/).

### Versioning Format

VantaDB uses the standard semantic versioning format for production releases:
`MAJOR.MINOR.PATCH` (e.g., `0.1.1`)

For pre-releases, the following formats are permitted:

- **Alpha:** `MAJOR.MINOR.PATCHaN` (e.g., `0.1.2a1`)
- **Beta:** `MAJOR.MINOR.PATCHbN` (e.g., `0.1.2b1`)
- **Release Candidate:** `MAJOR.MINOR.PATCHrcN` (e.g., `0.1.2rc1`)

### Tagging & Release Sync

- Every Python SDK release must be associated with a Git tag matching `vMAJOR.MINOR.PATCH`.
- `Cargo.toml`, `vantadb-python/Cargo.toml`, and
  `vantadb-python/pyproject.toml` must all match the release tag exactly.
- `cargo test --test version_coherence --all-features` is the local guardrail
  for version drift before building wheels.

---

## Secure Artifact Signing via Sigstore

To ensure supply-chain integrity, all built Python wheels (`.whl`) and source distributions (`.tar.gz`) must be signed using [Sigstore](https://www.sigstore.dev/).

### Keyless Signing Flow

1. The release workflow uses GitHub Actions OIDC (`id-token: write`) to establish a temporary identity.
2. The `sigstore/gh-action-sigstore-python` action requests an ephemeral signing certificate from the Fulcio Certificate Authority.
3. The built wheels are signed, generating:
   - A signature file (`.whl.sig`)
   - An identity certificate (`.whl.crt`)
4. Signed assets are published to GitHub Releases alongside the binary wheels for public verification.

### Verification Command

Downstream users can verify the integrity of a VantaDB wheel using the following command:

```bash
python -m sigstore verify github \
  --cert vantadb_py-0.1.1-py3-none-any.whl.crt \
  --signature vantadb_py-0.1.1-py3-none-any.whl.sig \
  vantadb_py-0.1.1-py3-none-any.whl
```

---

## Standard Release Flow

### 1. Local preflight

```bash
cargo test --test version_coherence --all-features
python -m pytest vantadb-python/tests/test_sdk.py -v
python -m maturin build --manifest-path ./vantadb-python/Cargo.toml --out ./target/wheels --release
```

Install the newest local wheel in a clean virtual environment and rerun the
Python SDK smoke suite before staging.

### 2. TestPyPI staging

Run the `Python Wheels` workflow manually with `publish_testpypi=true`.

The workflow builds wheels on Linux, macOS, and Windows, runs wheel smoke tests,
then uploads the merged wheel set to TestPyPI. Validate installation from
TestPyPI in a clean environment:

```bash
python -m pip install --index-url https://test.pypi.org/simple/ --extra-index-url https://pypi.org/simple vantadb-py
python -m pytest vantadb-python/tests/test_sdk.py -v
```

### 3. Production PyPI

Create and push a release tag only after TestPyPI validation:

```bash
git tag -s v0.1.1 -m "VantaDB Python 0.1.1"
git push origin v0.1.1
```

Tag pushes run the production publish job. That job signs wheels with Sigstore,
attaches `.whl`, `.sig`, and `.crt` assets to the GitHub Release, then publishes
to PyPI through `pypa/gh-action-pypi-publish`.

### 4. Rollback

PyPI files are immutable. If a release is bad:

- Yank the affected release on PyPI instead of deleting files.
- Publish a patched version with a higher patch or post-release version.
- Leave GitHub Release assets intact and add a release note with the yanked
  reason and replacement version.

---

## PyPI Fallback Rules

In the event of network disruption, token expiration, or service outages on primary package registries, the following fallback guidelines apply:

### 1. TestPyPI Staging Validation

- Before releasing to production PyPI, wheels must be built and validated against **TestPyPI**.
- Manual triggering via `workflow_dispatch` is available to test builds on TestPyPI with:

  ```bash
  publish_testpypi: true
  ```

### 2. Manual Fallback Releases

If the automated OIDC publishing fails due to registry maintenance:

- Release engineers may build wheels locally inside isolated Docker containers to guarantee reproducible builds across OS targets.
- Local wheels must be signed using user-managed Sigstore credentials.
- Publishing must be done using `twine`:

  ```bash
  python -m twine upload --repository pypi target/wheels/*
  ```

### 3. Source Fallback (Direct path / git URL)

In critical rescue scenarios where PyPI is offline, the SDK can be installed directly from the GitHub repository source using Maturin's in-place compilation fallback:

```bash
pip install git+https://github.com/DevpNess/Vantadb.git#subdirectory=vantadb-python
```

This requires local installation of the Rust toolchain on the target system.
