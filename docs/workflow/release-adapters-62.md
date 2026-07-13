# `release-adapters-62.yml` — RELEASE: Adapters — PyPI Publish

## ¿Qué hace?

Publica en PyPI los adapters de integración de VantaDB con frameworks de IA (LangChain, LlamaIndex, Mem0, CrewAI, DSPy, Haystack, Letta, OpenAI, Ollama). Corre sus tests y luego pública los wheels.

## ¿Cómo lo hace?

4 jobs secuenciales:

1. **`test-adapters`** (matrix × 9 adapters): por cada adapter:
   - Instala `vantadb-python` (editable)
   - Instala el adapter (`integrations/<adapter>/`)
   - Ejecuta `python -m pytest tests/ -v`
2. **`publish-adapter`** (matrix × 9 adapters, depende de tests): build con `python -m build` y sube el dist como artifact
3. **`publish-testpypi`** (opcional, depende de publish): pública a TestPyPI con trusted publishing (solo si `publish_testpypi: true`)
4. **`publish-pypi`** (solo con tag `adapters-v*`, depende de publish): pública a PyPI producción con:
   - Attestation de build provenance
   - Upload a GitHub Release
   - Publicación a PyPI vía `pypa/gh-action-pypi-publish`

## ¿Qué tests usa?

Por cada adapter, ejecuta sus tests unitarios con `pytest tests/`.

## ¿Qué verifica?

- Los tests de cada adapter pasan con el core actual
- Los wheels se construyen correctamente
- (Opcional) publicación a TestPyPI funciona
- La publicación a PyPI produce attestations verificables

## Funcionalidad final

Release automatizado de los 9 adapters de integración a PyPI con tests, build, attestation de seguridad y publicación.

## ¿Cuándo se ejecuta?

- **Push** de tag `adapters-v*.*.*` (publicación a PyPI producción)
- **Workflow dispatch** manual con opción de publicar a TestPyPI
