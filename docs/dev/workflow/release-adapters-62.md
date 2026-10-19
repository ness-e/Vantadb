---
title: "`release-adapters.yml` — RELEASE: Adapters — PyPI Publish"
kind: runbook
status: active
description: "Publica en PyPI los adapters de integración de VantaDB con frameworks de IA (LangChain, LlamaIndex, Mem0, CrewAI, DSPy, Haystack, Letta). Corre sus tests y luego pública los wheels"
tags: [vantadb, ci, release-adapters]
---

# `release-adapters.yml` — RELEASE: Adapters — PyPI Publish

## ¿Qué hace?

Publica en PyPI los adapters de integración de VantaDB con frameworks de IA (LangChain, LlamaIndex, Mem0, CrewAI, DSPy, Haystack, Letta). Corre sus tests y luego pública los wheels. Los twins OpenAI/Ollama quedan source-only (retirados del release — FIND-273, 2026-10-06).

## ¿Cómo lo hace?

4 jobs secuenciales:

1. **`test-adapters`** (matrix × 7 adapters): por cada adapter:
   - Instala `vantadb-python` (editable)
   - Instala el adapter (`integrations/<adapter>/`)
   - Ejecuta `python -m pytest tests/ -v`
2. **`publish-adapter`** (matrix × 7 adapters, depende de tests): build con `python -m build` y sube el dist como artifact
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

Release automatizado de los 7 adapters de integración a PyPI con tests, build, attestation de seguridad y publicación.

## ¿Cuándo se ejecuta?

- **Push** de tag `adapters-v*.*.*` (publicación a PyPI producción)
- **Workflow dispatch** manual con opción de publicar a TestPyPI
