---
title: "Deep audit W4 - Embeddings, providers and integrations"
kind: review
status: active
description: "Auditoria read-only de embeddings/ (manifest, download, verify), providers/ (openai, ollama, litellm) e integrations/ (9 adapters): licencia de modelos, coste de descarga, pins, cobertura por framework."
tags: [audit, reviews, embeddings]
---

# Deep audit W4 - Embeddings + Providers + Integrations

> **Wave:** W4 · **Fecha:** 2026-10-20 · **Modo:** read-only
> **Especificacion:** `docs/dev/plans/2026-10-20-revision-profunda-integral-vantadb.md` §4 (W4)
> **Ejecucion:** subagentes `vanta-engine` (calidad de embedding/recall) + `vanta-audit`
> (contratos/licencias/pins) no disponibles (sin tool `task`).

## Scope

**Cubierto:**

| Area | Ruta | Detalle |
|---|---|---|
| Embeddings | `embeddings/{manifest.json,manifest.lock,download.py,verify.py,sanity_embed.py,sanity_report.json,README.md}` + `embeddings/models/` | 10 modelos en manifest, 4 en lock |
| Providers | `providers/{openai,ollama,litellm}/` + `providers/shared_py.rs` | 3 providers, cada uno con `src/{lib,python}.rs`, `tests/`, `.pyi`, `pyproject.toml` |
| Integrations | `integrations/{langchain,llamaindex,crewai,dspy,haystack,letta,mem0,ollama,openai,vantadb_shared}/` | 9 adapters distribuibles + 1 shared |
| Gates | `integrations/test_pins.py`, `.github/scripts/verify_pyi.py` | - |

**Excluido con motivo:** `embeddings/models/` (artefactos locales, gitignored en `.gitignore:238-239`),
salvo la medicion de tamano real en disco, que si es evidencia.

## Metodologia

- Lectura integra de `embeddings/manifest.json` y `embeddings/manifest.lock`.
- **Verificacion contra la fuente oficial** del modelo (Regla 9 del prompt: "verifica contra la fuente
  oficial"): model card en `https://gemma.ai/models/embeddinggemma/model-card-v2/`, ficha de
  `https://huggingface.co/google/embeddinggemma-2` y de
  `https://huggingface.co/onnx-community/embeddinggemma-2-ONNX` (consultadas el 2026-10-20).
- Medicion real de tamano en disco de `embeddings/models/embeddinggemma-2/onnx/`.
- Lectura de la logica de patrones de descarga en `embeddings/download.py`.
- Lectura de `integrations/test_pins.py` y `.github/scripts/verify_pyi.py`.
- Sin ejecucion de `download.py` (no se quiere disparar descargas de red).

## Hallazgos

| ID | Severidad | Hallazgo | Archivo:línea | Evidencia | Fix propuesto | Esfuerzo |
|---|---|---|---|---|---|---|
| `FIND-382` | High | **Licencia de `embeddinggemma-2` mal declarada en el manifest: dice "Gemma", la fuente oficial dice Apache-2.0** | `embeddings/manifest.json:129` (`"license": "Gemma"`) | Fuente oficial (consultada 2026-10-20): el model card de Google DeepMind dice literalmente "**License : Apache 2.0 \| Authors : Google DeepMind**" (<https://gemma.ai/models/embeddinggemma/model-card-v2/>). Las dos fichas de Hugging Face coinciden: `google/embeddinggemma-2` -> "License: apache-2.0" y `onnx-community/embeddinggemma-2-ONNX` -> "License: apache-2.0". Ademas, el modelo **si** es multimodal (texto+imagen+video+audio, 740M params: 270M texto + 170M vision + 300M audio) y **si** tiene soporte ONNX oficial via `onnx-community`. Consecuencia: el repo declara un modelo como licencia *restrictiva* (Gemma) cuando es *permissiva* (Apache-2.0), lo que (a) asusta a quien integrate, (b) rompe cualquier gate de licencias que confie en el manifest, y (c) deja sin documento la base legal real. `embeddings/download.py:100` exige que el campo `license` **exista** pero nunca valida su **valor** - por eso el error pasa el gate. | Cambiar `"license": "Gemma"` por `"license": "Apache-2.0"` en `embeddings/manifest.json`, y anadir el valor a la lista de licencias validadas por `download.py` (`check_manifest`). Anadir `"modalities": ["text","image","audio","video"]` para que el runtime sepa que solo consume texto. | 1h |
| `FIND-383` | High | **La descarga de `embeddinggemma-2` arrastra 5 888 MB cuando solo hacen falta 1 034 MB** (el 83 % son codificadores de audio/vision que VantaDB no usa) | `embeddings/download.py:30` (`ALLOW_PATTERNS = ["*.json", "*.txt", "tokenizer*", "onnx/*", "*.safetensors"]`) | `MODEL_PATTERNS` (linea 34) esta **vacio** ("Override explicito por modelo... Vacio hoy"), asi que todos los modelos usan el glob `onnx/*`. Como `embeddinggemma-2` es multimodal, `onnx/*` trae **todos** los encoders y todas las variantes de cuantizacion. Medicion real en disco de `embeddings/models/embeddinggemma-2/onnx/`: `model.onnx` 0.4 MB + `model.onnx_data` 1 033.9 MB = **1 034 MB** (lo que VantaDB usa, texto), frente a `audio_encoder.onnx_data` 1 118.4 MB, `vision_encoder.onnx_data` 639.9 MB y 12 variantes cuantizadas mas. **Total del directorio: 5 888 MB**. Cada usuario que ejecute `python embeddings/download.py --only embeddinggemma-2` (o `scripts/setup-embeddinggemma.ps1`) descarga 5.9 GB para usar 1 GB. El comentario de `download.py:26-29` dice que la base se recortó precisamente para no multiplicar la descarga ("descarga 3-4x lo declarado en README"), pero el recorte no cubre el caso multimodal. | Meter `embeddinggemma-2` en `MODEL_PATTERNS` con una lista cerrada: `["*.json", "tokenizer*", "onnx/model.onnx", "onnx/model.onnx_data", "onnx/config.json"]`. Verificar el nombre exacto de `config.json` en el repo (el `.cache` local registra `onnx/model.onnx` + `onnx/model.onnx_data`). | 2-4h |
| `FIND-384` | Medium | El tamano declarado del modelo no coincide con el tamano real | `embeddings/manifest.json:130` (`"size_onnx_mb": 550`) | El manifest declara `size_onnx_mb: 550` para `embeddinggemma-2`. La medicion real del encoder de texto (`model.onnx` 0.4 MB + `model.onnx_data` 1 033.9 MB) da **1 034 MB**, casi el doble de lo declarado. `embeddings/download.py:160-162` imprime `size_onnx_mb` como si fuera la verdad ("la fuente es el manifest", Regla 11), asi que el numero que se muestra al usuario es incorrecto. El mismo campo para `bge-m3` (1 200) y `jina-es-v2-base` (1 100) no pudo contrastarse (modelos no descargados localmente). | Re-medir y corregir `size_onnx_mb` con el valor real (1 034), o definir el campo como "encoder de texto + tokenizer" y documentarlo. Idealmente, que `download.py` mida el tamano real tras descargar y avise si difiere del manifest. | 2h |
| `FIND-402` | Medium | El modelo por defecto esta **hardcodeado en dos sitios** (el manifest y el verificador), asi que cambiarlo exige editar codigo | `embeddings/download.py:123-125`, `embeddings/verify.py:78` | `download.py:123`: `if manifest.get("default") != "multilingual-e5-small": print("[check] FAIL default debe ser multilingual-e5-small")`. `verify.py:78`: `if manifest.get("default") != "multilingual-e5-small":`. El valor por defecto es un **dato** del manifest, pero el gate lo re-declara como literal en Python. Cambiar el default (p.ej. a `embeddinggemma-2` tras la correccion de licencia) rompe el gate con un mensaje que no explica cual es el valor correcto. | Extraer el valor esperado a una constante unica compartida (`embeddings/_manifest_defaults.py` o leerlo del propio manifest y comparar contra una lista de valores permitidos). | 1h |
| `W4-05` | Info | `manifest.lock` es un **subset acumulativo** por diseno, no un bug | `embeddings/manifest.json:4` (`"default"`), `embeddings/manifest.lock` | `manifest.lock` fija 4 de los 10 modelos (`all-MiniLM-L6-v2`, `paraphrase-multilingual-MiniLM-L12-v2`, `multilingual-e5-small`, `embeddinggemma-2`). `download.py:176` lo documenta: "subset acumulativo". `download.py:170-172` valida que repo+rev del lock coincidan con el manifest, y falla si divergen. El lock registra **lo que se ha descargado y fijado**, no lo que se ofrece. Diseno correcto. | Sin accion. | - |
| `W4-06` | Info | Contrato de pins de integrations: gate efectivo, offline, 9 adapters | `integrations/test_pins.py:1-48` | Exige que cada dependencia de framework de los 9 adapters declare un techo (`<`, `<=`, `==`, `~=`). `vantadb-py>=0.6.1,<0.9.0` queda exenta por llevar techo propio. `test_nine_adapters_present` fija la matriz exacta. Corre offline (solo parsea TOML). Nota: el techo `<0.9.0` de `vantadb-py` quedara **obsoleto** al publicarse 0.9.0 - conviene que el gate avise cuando la version del workspace alcanza el techo. | Anadir al gate un check `workspace_version < techo de vantadb-py`. | 2h |
| `W4-07` | Info | Los 3 providers comparten una base comun (`shared_py.rs`) y tienen stub + test + gate de paridad | `providers/shared_py.rs`, `providers/{openai,ollama,litellm}/vantadb_*.pyi` | Cada provider tiene `src/lib.rs`, `src/python.rs`, `tests/test_*.py`, `vantadb_<name>.pyi` y `pyproject.toml`. `.github/scripts/verify_pyi.py` compara **nombres de parametros y presencia de defaults** (no solo `hasattr`) entre stub y runtime, con `REQUIRED_METHODS` de 8 metodos (`embed`, `embed_batch`, `search`, `store`, `delete`, `get`, `list`, `list_namespaces`). Invocado por `ci-fleet.yml:468` y `release-providers.yml` (smoke de wheel). | Sin accion. | - |
| `W4-08` | Low | Comentario obsoleto en el downloader | `embeddings/download.py:32-33` | El comentario dice "los 9 repos exponen model.safetensors", pero el manifest tiene **10** modelos y `qwen3-embedding-8b` esta declarado con `onnx: null` + `exception` (GPU only, ONNX no recomendado) - no expone `model.safetensors` como los otros. El propio gate de `download.py:133-135` lo comprueba. Comentario desalineado con la realidad. | Actualizar el comentario o parametrizarlo por el manifest. | 15min |
| `W4-09` | Info | La carpeta de modelos contiene una base de datos VantaDB viva (`_embed_scratch/`) | `embeddings/models/multilingual-e5-small/_embed_scratch/` | Contiene `data/vanta.shard{0..3}.wal`, `data/vstore_L{0..3}.vanta`, `data/vector_index.bin`, `keyspaces/0..10/` y `lock`/`version`. Es un artefacto de `sanity_embed.py`. Esta gitignored (`.gitignore:238` cubre `/embeddings/models/`), asi que no llega al repo, pero ensucia el directorio destino de `snapshot_download` y puede confundir a `download.py:213` (que comprueba `local_dir.exists() and any(local_dir.rglob("*.onnx"))`). | Apuntar `sanity_embed.py` a un directorio temporal fuera del arbol de modelos. | 1h |
| `W4-10` | Info | EmbeddingGemma 2 esta **ya** en el manifest (no es una incorporacion pendiente) | `embeddings/manifest.json:120-131` | `embeddinggemma-2` -> repo `onnx-community/embeddinggemma-2-ONNX`, rev `daa72c5`, `dim: 768`, `size_onnx_mb: 550`, `size_hf_mb: 600`, `langs: ["es","en","code","100+"]`, `group: "combined"`. El rev coincide con el tree local (`embeddings/models/embeddinggemma-2/.cache/huggingface/trees/daa72c51243991dfcaf9f9137d2c573d8f7790c0.json`). `dim: 768` y `100+` son **correctos** segun la fuente oficial. `scripts/setup-embeddinggemma.ps1` existe y automatiza la descarga ONNX + el modo Ollama + la verificacion de integridad del manifest. | Sin accion de estructura. Corregir licencia y tamano (FIND-382, FIND-384). | - |

## Veredicto

**FAIL**

El subsistema de embeddings tiene **dos defectos High de compliance y de coste** que son verificables
y no requieren opinion: (1) la licencia de `embeddinggemma-2` esta mal declarada - el manifest dice
"Gemma" cuando la fuente oficial (model card de Google DeepMind + las dos fichas de Hugging Face)
dice **Apache 2.0**, y el gate de integridad no valida el valor de la licencia, por lo que el error
pasa inadvertido; y (2) la descarga de ese modelo arrastra **5 888 MB medidos en disco** de los que
VantaDB usa 1 034 MB, porque el patron de descarga `onnx/*` no esta recortado para un modelo
multimodal. Aparte, el tamano declarado (550 MB) no coincide con el real (1 034 MB), el default esta
duplicado entre manifest y verificador, y la cobertura de providers e integrations es solida
(pins con techo verificados offline, stubs `.pyi` con gate de paridad de firmas, 9 adapters con
matriz fijada). El FAIL es por compliance de licencia, no por fallo funcional.

## Indice de hallazgos

| ID | Severidad | Archivo:línea |
|---|---|---|
| `FIND-382` | High | `embeddings/manifest.json:129` |
| `FIND-383` | High | `embeddings/download.py:30` |
| `FIND-384` | Medium | `embeddings/manifest.json:130` |
| `FIND-402` | Medium | `embeddings/download.py:123` |
| `W4-05` | Info | `embeddings/manifest.lock:1` |
| `W4-06` | Info | `integrations/test_pins.py:1` |
| `W4-07` | Info | `providers/shared_py.rs:1` |
| `W4-08` | Low | `embeddings/download.py:33` |
| `W4-09` | Info | `embeddings/models/multilingual-e5-small/_embed_scratch/` |
| `W4-10` | Info | `embeddings/manifest.json:120` |
