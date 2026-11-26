---
title: "Fase 1 — Inventario y estadísticas verificables"
kind: report
description: "Censo del worktree, historia, deuda, dependencias y límites de medición del análisis integral."
---

# Fase 1 — Inventario y estadísticas verificables

[HECHO] Captura inicial: **2026-11-06 13:33:38 UTC**, HEAD `08c6dc603c009edf8d9b76cee14e7006a8a89ad7`, rama `develop`. Se excluyen otros worktrees y no se ejecutaron tests, builds ni arranques.

## Alcance y método

[HECHO] Universo: `git ls-files -z`, conservando archivos presentes. SHA-256 de las rutas ordenadas unidas por LF: `cc54fbd4a6812df198dcc5348105224e19cd325a3799c0bd61a5f62bda8baf54`. Hay **1.455 archivos tracked presentes**. La wiki nueva y estos reportes no forman parte de esa captura.

[HECHO] El contador de lectura, Python estándar, reconoce extensiones `.rs .py .ts .tsx .js .mjs .cjs .ps1 .sh .rb .c .h .yml .yaml .toml .md .json .html .css`; rechaza contenido con NUL y cuenta `splitlines()` y líneas con `strip()` no vacío. Se reconocieron **1.355 archivos**, **438.933 líneas físicas** y **394.961 líneas no vacías (LNV)**. Los otros 100 archivos no se clasifican como texto reconocido; no significa que todos sean binarios.

[HECHO] **No son SLOC**: incluyen comentarios, docstrings y código de tests inline. `tokei`/`cloc` no están en PATH y no se instalaron. La medición exacta solicitada con esas herramientas permanece **pendiente**; no se presenta este fallback como equivalente. Tampoco se midió cobertura de ejecución.

[HECHO] La documentación local gitignored se leyó como evidencia cuando correspondía, pero no pertenece al censo tracked. En particular, `.gitignore:269` excluye `docs/dev/reports/`; los cinco entregables no aparecerán en `git diff` normal ni en los índices que filtran ignorados.

## Lenguajes

[HECHO] Resultados del contador definido arriba:

| Lenguaje/formato | Archivos | Líneas físicas | LNV |
|---|---:|---:|---:|
| Rust | 716 | 289.702 | 264.780 |
| Python | 127 | 26.092 | 21.923 |
| TypeScript | 54 | 13.728 | 12.455 |
| JavaScript, incluido mjs/cjs | 47 | 14.347 | 13.231 |
| PowerShell | 42 | 6.212 | 5.662 |
| Shell | 11 | 770 | 668 |
| Ruby | 1 | 59 | 53 |
| Markdown | 238 | 50.598 | 39.854 |
| JSON | 48 | 23.937 | 23.937 |
| YAML | 27 | 9.449 | 8.712 |
| TOML | 40 | 2.651 | 2.325 |
| HTML | 4 | 1.388 | 1.361 |
| **Total** | **1.355** | **438.933** | **394.961** |

## Tamaño por módulo

[HECHO] Cada fila es disjunta: archivos directos de `src/` en `src/root`, sus directorios en filas propias; el resto por primer componente. Se cuentan todos los formatos reconocidos, no únicamente código.

| Módulo | Archivos | Líneas físicas | LNV |
|---|---:|---:|---:|
| src/root | 52 | 36.300 | 33.022 |
| src/agentic | 2 | 601 | 538 |
| src/api | 2 | 115 | 101 |
| src/backends | 6 | 2.957 | 2.700 |
| src/bin | 3 | 503 | 445 |
| src/cli_handlers | 20 | 7.450 | 6.855 |
| src/crypto | 1 | 1.289 | 1.191 |
| src/entity | 8 | 3.002 | 2.688 |
| src/graphrag | 6 | 497 | 436 |
| src/hardware | 1 | 764 | 681 |
| src/index | 32 | 14.747 | 13.556 |
| src/metrics | 5 | 2.631 | 2.397 |
| src/node | 9 | 2.319 | 2.098 |
| src/parser | 3 | 3.091 | 2.783 |
| src/physical_plan | 10 | 2.938 | 2.664 |
| src/sdk | 50 | 28.052 | 25.932 |
| src/serialization | 1 | 4 | 4 |
| src/server | 16 | 7.267 | 6.696 |
| src/shred | 1 | 745 | 648 |
| src/skills | 1 | 560 | 485 |
| src/storage | 29 | 20.664 | 18.922 |
| src/testing | 2 | 140 | 129 |
| src/tui | 4 | 1.082 | 989 |
| src/utils | 4 | 621 | 552 |
| src/vector | 4 | 1.189 | 1.059 |
| src/wiki | 7 | 2.482 | 2.239 |
| vanta-memory | 165 | 43.176 | 39.353 |
| vanta-proxy | 61 | 18.745 | 17.342 |
| vantadb-ffi-core | 2 | 246 | 228 |
| vantadb-mcp | 49 | 27.575 | 25.757 |
| vantadb-node | 25 | 10.807 | 10.327 |
| vantadb-python | 34 | 10.879 | 9.470 |
| vantadb-server | 13 | 3.269 | 2.939 |
| vantadb-ts | 51 | 13.779 | 12.763 |
| vantadb-wasm | 21 | 9.264 | 8.415 |
| integrations | 72 | 7.058 | 5.821 |
| providers | 19 | 2.532 | 2.244 |
| tests | 112 | 33.874 | 30.117 |
| benches | 25 | 5.511 | 5.019 |
| benchmarks | 17 | 4.852 | 4.255 |
| evals | 16 | 16.622 | 15.999 |
| fuzz | 5 | 138 | 116 |
| examples | 24 | 3.873 | 3.199 |
| docs | 186 | 43.628 | 34.928 |
| scripts | 49 | 12.574 | 11.436 |
| dev-tools | 40 | 9.296 | 8.707 |
| skills | 25 | 3.646 | 2.922 |
| .github | 30 | 6.861 | 6.153 |
| .githooks | 1 | 73 | 70 |
| .cargo | 2 | 54 | 47 |
| .config | 1 | 108 | 103 |
| raíz, archivos directos | 16 | 5.761 | 4.919 |
| Formula | 2 | 109 | 87 |
| completions | 1 | 1.023 | 1.018 |
| data | 1 | 60 | 47 |
| datasets | 1 | 60 | 43 |
| e2e | 3 | 235 | 191 |
| embeddings | 6 | 1.188 | 1.078 |
| metricas | 1 | 47 | 38 |

### Tests frente a código

[HECHO] En las extensiones de código se identificaron 998 archivos. La partición **por rutas**, no por AST, arroja 84.012 LNV de tests/fuzz, 232.662 LNV fuera de rutas de test y 2.098 LNV de declaraciones/generado identificado. Ratio de esa partición: `84012 / 232662 = 0,36109` (**36,11%**).

[HECHO] Se clasifican tests mediante componentes `test`, `tests`, `__tests__`, `fuzz` o nombres `test_*`, `*.test`, `*_test`, `*.spec` y equivalentes. Los módulos `#[cfg(test)]` dentro de archivos de producción permanecen en el denominador. Benchmarks y ejemplos fuera de esos patrones también permanecen allí.

[INFERENCIA] Por ello, 36,11% no es el ratio SLOC tests/producción ni un límite matemático de cobertura; es un indicador de separación por rutas. La estimación financiera utiliza su propio universo explícito y no confunde estas categorías.

## Inventario de paquetes

[HECHO] Fuente: manifiestos presentes, no nombres deducidos de README.

| Manifiesto | Nombre | Versión declarada | Publicación crates.io |
|---|---|---|---|
| Cargo.toml | vantadb | workspace 0.9.0 | permitida |
| vanta-memory/Cargo.toml | vanta-memory | workspace 0.9.0 | permitida; release-plz puede retenerla |
| vanta-proxy/Cargo.toml | vanta-proxy | workspace 0.9.0 | false |
| vantadb-ffi-core/Cargo.toml | vantadb-ffi-core | workspace 0.9.0 | false |
| vantadb-mcp/Cargo.toml | vantadb-mcp | workspace 0.9.0 | false |
| vantadb-python/Cargo.toml | vantadb_py | workspace 0.9.0 | false; canal Python separado |
| vantadb-server/Cargo.toml | vantadb-server | workspace 0.9.0 | false |
| vantadb-wasm/Cargo.toml | vantadb-wasm | workspace 0.9.0 | permitida; canal npm separado |
| vantadb-node/Cargo.toml | vantadb-node | 0.9.0 | false; fuera del workspace |
| providers/openai/Cargo.toml | vantadb-openai | 0.9.0 | false; independiente |
| providers/ollama/Cargo.toml | vantadb-ollama | 0.9.0 | false; independiente |
| providers/litellm/Cargo.toml | vantadb-litellm | 0.9.0 | false; independiente |
| fuzz/Cargo.toml | vantadb-fuzz | 0.0.0 | false; excluido |
| vantadb-wasm/threads-kernel/Cargo.toml | vantadb-wasm-threads-kernel | 0.1.0 | false; independiente |

[HECHO] Los ocho primeros son miembros del workspace. Default-members: `.`, `vantadb-python`, `vanta-memory`, `vantadb-server`, `vantadb-mcp`. `vantadb-ts` es npm, no crate. Las integraciones Python presentes cubren LangChain, LlamaIndex, CrewAI, DSPy, Haystack, Letta, mem0, Ollama y OpenAI, más utilidades compartidas; ver [ecosistema](../../../wiki/modules/ecosystem.md).

[HECHO] Evals tracked: harnesses de memoria/LongMemEval/LoCoMo, runners y casos de agente; datasets externos grandes no se cuentan como fuente del producto. Su existencia no acredita resultados recientes.

## Historia y churn

[HECHO] `git rev-list --count HEAD`: **4.250 commits** alcanzables, incluidos **54 merges**. `git log --reverse --format=%aI HEAD` comienza con fecha de autor `2026-04-02T00:05:20-04:00`; es evidencia de historia importada, no una fecha contable de comienzo de trabajo. El repositorio no es shallow.

[HECHO] `git shortlog -sne HEAD`, con normalización habitual de Git: Eros Nessy 4.005; github-actions[bot] 172; dependabot[bot] 63; Devin AI 6; ness-e 4. Son **cinco identidades**, no cinco personas independientes. No se fusionan identidades adicionales por suposición ni se divulgan emails.

[HECHO] Churn corregido y acotado a **rutas tracked actualmente presentes**, sin merges y con `--no-renames`: añadidas + eliminadas. Se descartan rutas históricas ausentes, `desktop/` y `web/`; no se atribuye al producto actual la eliminación de otros worktrees. No es historia completa anterior a renames.

| Ventana | Desde UTC | Añadidas | Eliminadas | Cambios binarios, separados |
|---|---|---:|---:|---:|
| [HECHO] 30 días calendario | 2026-10-07 00:00 | 80.158 | 22.728 | 1 |
| [HECHO] 90 días calendario | 2026-08-08 00:00 | 701.768 | 115.545 | 31 |
| [HECHO] Toda historia de rutas actuales | sin filtro temporal | 888.815 | 153.151 | 31 |

| Módulo | Añadidas 30d | Eliminadas 30d | Añadidas 90d | Eliminadas 90d |
|---|---:|---:|---:|---:|
| [HECHO] src/storage | 4.314 | 459 | 23.830 | 9.056 |
| [HECHO] src/sdk | 1.881 | 262 | 41.877 | 14.395 |
| [HECHO] src/index | 2.867 | 316 | 21.287 | 1.191 |
| [HECHO] src/root | 6.856 | 446 | 37.876 | 10.424 |
| [HECHO] src/server | 468 | 87 | 19.817 | 6.378 |
| [HECHO] vanta-memory | 1.864 | 363 | 77.998 | 1.767 |
| [HECHO] vanta-proxy | 431 | 152 | 38.117 | 952 |
| [HECHO] vantadb-mcp | 2.112 | 123 | 53.936 | 8.394 |
| [HECHO] vantadb-python | 1.580 | 773 | 16.218 | 4.729 |
| [HECHO] vantadb-node | 4.100 | 1.850 | 18.227 | 4.209 |
| [HECHO] vantadb-ts | 2.096 | 531 | 14.111 | 2.624 |
| [HECHO] vantadb-wasm | 1.144 | 756 | 11.299 | 1.939 |
| [HECHO] docs | 10.946 | 7.545 | 99.741 | 23.213 |
| [HECHO] tests | 3.500 | 473 | 60.132 | 3.864 |
| [HECHO] evals | 2.168 | 70 | 31.229 | 83 |
| [HECHO] integrations | 159 | 95 | 5.291 | 1.562 |

[INFERENCIA] La concentración de contribuciones y la amplitud de superficies sugieren riesgo de mantenimiento por bus factor; las identidades y el churn no miden por sí mismos dedicación humana, calidad ni coste.

## Deuda explícita y dependencias

[HECHO] Búsqueda de palabras completas uppercase en extensiones de código: **4 TODO, 0 FIXME, 0 HACK, en tres archivos**. Tres son deuda explícita de adaptación: `integrations/langchain/vantadb_langchain/vectorstore.py:243`, `integrations/llamaindex/vantadb_llamaindex/vectorstore.py:179,311`. El cuarto, `src/sdk/types/record.rs:1176`, dice «TODO campo nuevo» en español: **falso positivo semántico**. Recuento útil tras lectura: tres marcadores, no una medición exhaustiva de deuda.

[HECHO] `Cargo.lock` raíz contiene **627 entradas de paquetes**, 619 registry, ninguna git y 30 nombres con varias versiones. No cubre necesariamente resoluciones independientes de Node/providers/kernel. El manifiesto raíz declara 65 dependencias normales y 16 dev en sus tablas generales; no incorpora aquí el cómputo de tablas target específicas.

[HECHO] Muestra de frescura consultada el 2026-11-06 en `https://crates.io/api/v1/crates/<nombre>`: releases estables no yanked; no se ejecutó `cargo update` ni auditoría de vulnerabilidades.

| Dependencia / fuente | Lock raíz | Última estable observada | Publicación UTC |
|---|---|---|---|
| [fjall](https://crates.io/api/v1/crates/fjall) | 3.1.12 | 3.1.13 | 2026-10-25 |
| [pyo3](https://crates.io/api/v1/crates/pyo3) | 0.30.1 | 0.30.1 | 2026-10-14 |
| [tokio](https://crates.io/api/v1/crates/tokio) | 1.54.1 | 1.54.1 | 2026-10-21 |
| [axum](https://crates.io/api/v1/crates/axum) | 0.8.9 | 0.8.9 | 2026-04-14 |
| [arc-swap](https://crates.io/api/v1/crates/arc-swap) | 1.9.3 | 1.9.3 | 2026-10-18 |
| [parking_lot](https://crates.io/api/v1/crates/parking_lot) | 0.12.5 | 0.13.0 | 2026-10-21 |
| [serde](https://crates.io/api/v1/crates/serde) | 1.0.229 | 1.0.229 | 2026-07-18 |
| [serde_json](https://crates.io/api/v1/crates/serde_json) | 1.0.154 | 1.0.154 | 2026-10-11 |
| [wasm-bindgen](https://crates.io/api/v1/crates/wasm-bindgen) | 0.2.126 | 0.2.130 | 2026-10-30 |
| [reqwest](https://crates.io/api/v1/crates/reqwest) | 0.12.28 | 0.13.5 | 2026-09-08 |
| [tantivy](https://crates.io/api/v1/crates/tantivy) | 0.27.0 | 0.27.0 | 2026-10-17 |

[HECHO] La tabla anterior contiene observaciones externas por fila, no una recomendación de actualizar. `napi` no aparece en el lock raíz; su ausencia allí no implica que Node no lo use. Frescura npm/Python transitiva y vulnerabilidades: **no evaluadas exhaustivamente**.

[INFERENCIA] Priorizar evaluación de parches del backend y compatibilidad de bindgen antes que perseguir todas las major releases; validar MSRV/ABI y pruebas de cada transporte, sin modificar controles de seguridad de dependencias.

## Estado de herramientas y cobertura

| Herramienta | Resultado observado | Consecuencia |
|---|---|---|
| [HECHO] CodeGraph | CLI: 972 archivos, 22.676 nodos, 89.956 edges; declara actualizado | Descubrimiento, no cobertura total |
| [HECHO] CBM index_status | ready; 27.304 nodos, 163.689 edges | Snapshot vivo distinto del artefacto local; no sumar ambos grafos |
| [HECHO] CBM parse parcial | 16 archivos PowerShell; 0 skipped declarados | Lectura directa de rangos; ausencia de flag no prueba completitud |
| [HECHO] CBM exclusiones | 82 directorios y 62 archivos excluidos por diseño | Incluye tooling local y docs/dev gitignored; no prueba ausencia |
| [HECHO] CBM check_index_coverage | Diez rutas críticas sin issue registrado, todas metadata_changed | Leer fuente actual; no reindexar sin autorización |
| [HECHO] MCP vantadb | Falló el listado de herramientas en Plan y de nuevo en Code | Dogfooding **no verificado**; fallback revisión estática, sin tocar BD |
| [HECHO] websearch | Búsqueda y fetch operativos | Contraste con fuentes primarias |
| [HECHO] metasearch compare_engines | Bing/DuckDuckGo devolvieron listas vacías | Sin corroboración de esa consulta; fallback Argus + fuentes oficiales |
| [HECHO] Argus | Extracción de LanceDB operativa, redirige a docs.lancedb.com | Se consultó el destino final |
| [HECHO] tokei/cloc | No encontrados en PATH | SLOC exacto pendiente |
| [HECHO] Subagentes | Cuatro revisiones estáticas completadas; grafo no disponible en sus runtimes | Hallazgos contrastados por lead; no fingir uso del grafo en cada agente |
| [HECHO] Python stdout | Una lectura falló por cp1252 ante Unicode | Repetir lectura con PYTHONIOENCODING=utf-8; sin efecto en repositorio |

## Receta de reproducción y comprobaciones pendientes

[HECHO] Los comandos de historia usados fueron `git rev-parse HEAD`, `git rev-parse --is-shallow-repository`, `git rev-list --count HEAD`, `git rev-list --merges --count HEAD`, `git shortlog -sne HEAD` y `git log HEAD --no-merges --format= --numstat --no-renames -z --since=<inicio UTC>`. Se parsearon campos NUL, se filtró cada ruta por pertenencia al conjunto tracked presente y se agregaron añadidas/eliminadas por módulo; binarios `-` se contaron aparte.

[HECHO] Receta del censo: enumerar rutas NUL, leer bytes sin NUL como UTF-8-sig con sustitución, aplicar extensiones declaradas, calcular `len(text.splitlines())` y `sum(bool(line.strip()) for line in text.splitlines())`; agrupar con la función siguiente. Ningún dato se obtuvo ejecutando código del producto.

```python
def modulo(path):
    parts = path.split("/")
    if parts[0] == "src":
        return "src/" + parts[1] if len(parts) > 2 else "src/root"
    return parts[0] if len(parts) > 1 else "(root)"
```

[HECHO] Para reproducción exacta, usar el HEAD indicado y el manifiesto de rutas de esa captura; archivos eliminados, untracked o gitignored no entran. La lectura de manifiestos usa `tomllib`; la deuda usa `r"\b(TODO|FIXME|HACK)\b"` y la partición de test descrita arriba. La tabla de módulos permite verificar sumas sin depender de capturas externas efímeras.

[INFERENCIA] Comprobaciones manuales pendientes: obtener tokei/cloc con versión y exclusiones registradas; separar tests inline mediante herramienta estructural; validar cobertura con suite autorizada por el usuario; ampliar frescura a locks independientes. No se han sustituido por un PASS.

[HECHO] Navegación: [reporte maestro](REPORT.md) · [fase técnica](fase2-tecnico.md) · [adopción](fase3-adopcion.md) · [finanzas](fase4-financiero.md).
