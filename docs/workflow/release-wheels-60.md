# `release-wheels-60.yml` — RELEASE: Wheels — Build & Publish

## ¿Qué hace?

Construye, prueba y publica los wheels Python (vantadb-py) para Linux, macOS y Windows. Incluye smoke tests de importación y funcionales, y verificación de build provenance.

## ¿Cómo lo hace?

6 jobs:

1. **`build-wheels`** (matrix × 3 OS: ubuntu, macos, windows):
   - Configura página de memoria en Windows (8-16GB)
   - Valida coherencia de versiones con `cargo test --test version_coherence`
   - Build wheel con `maturin build --release`
   - Smoke test: instala en virtualenv, importa, corre `test_sdk.py`
   - Sube el wheel como artifact
2. **`publish-testpypi`** (opcional): pública los wheels a TestPyPI usando trusted publishing
3. **`publish-pypi`** (solo tag `v*`): pública a PyPI producción con attestation de build provenance y attach a GitHub Release
4. **`verify-testpypi-install`**: instala desde TestPyPI y corre `verify_published_wheel.py`
5. **`verify-pypi-install`**: instala desde PyPI, corre `verify_published_wheel.py`, y verifica build provenance con `gh attestation verify`

## ¿Qué tests usa?

- `version_coherence` — test Rust de coherencia de versiones
- `test_sdk.py` — tests del SDK Python
- `verify_published_wheel.py` — script de verificación post-publicación

## ¿Qué verifica?

- El wheel compila y se empaqueta correctamente en los 3 OS
- La versión del wheel es coherente con Cargo.toml
- El import `import vantadb_py` funciona
- Los tests del SDK Python pasan
- La instalación desde PyPI/TestPyPI funciona
- Build provenance attestation es verificable (supply chain integrity)

## Funcionalidad final

Release automatizado de la rueda Python de VantaDB a PyPI con cobertura multi-plataforma, smoke tests y verificación de integridad de supply chain.

## ¿Cuándo se ejecuta?

- **Push** de tag `v*.*.*` (publicación a PyPI producción)
- **Pull Request** a `main` con cambios en `src/`, `vantadb-python/`, `Cargo.toml`, `Cargo.lock` o el propio workflow
- **Workflow dispatch** manual con opción de publicar a TestPyPI
