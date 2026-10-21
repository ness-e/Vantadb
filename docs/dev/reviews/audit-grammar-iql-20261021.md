---
title: "Auditoria profunda de la gramatica IQL (src/parser) - 2026-10-21"
kind: review
status: active
description: "Auditoria de solo lectura de src/parser (gramatica, lexer, AST) y su contrato con el planner y el executor. 14 hallazgos (1 critico, 8 altos), 3 formas documentadas que no parsean, 11 comportamientos aceptados que la doc no documenta y 6 divergencias de semantica. Cierra FIND-502."
tags: [vantadb, audit, reviews, parser, iql, docs]
---

# Auditoría profunda de la gramática IQL (`src/parser/`)

> **Campaña:** auditoría read-only del 2026-10-21, disparada por `FIND-502` (la campaña del
> 2026-10-20 declaró `src/parser/` como **no cubierto en profundidad**).
> **Alcance real:** `src/parser/mod.rs` (49 líneas de código + 1.865 de tests), `src/parser/grammar.rs` (832),
> `src/parser/lexer.rs` (180) y su contrato con `src/query.rs`, `src/planner.rs`, `src/physical_plan/`,
> `src/executor.rs`.
> **Método:** lectura + `grep` + `codegraph`, y la fuente de `nom 7.1.3` en el registry local.
> **Sin `cargo`** (había otro agente compilando y la página de Windows se agota): lo verificado
> leyendo, lo no medido está marcado como tal.
> **Skills:** `security-and-hardening`, `code-review-and-quality`, `doubt-driven-development`.

## 1. Inventario real de la gramática

### 1.1 Statements (`grammar.rs:756-778`)

| # | Statement | Parser | Nota |
|---|---|---|---|
| 1 | `INSERT MESSAGE <SYSTEM\|USER\|ASSISTANT> "<contenido>" TO THREAD#<u128>` | `:385-409` | `ASSISTANT` (`:392`) **no** está en `RESERVED_KEYWORDS` |
| 2 | `INSERT NODE#<u128> TYPE <tipo> { <campo>: <valor>, ... } [VECTOR [f32,...]]` | `:289-314` | `VECTOR` opcional; `{}` admite campos vacíos |
| 3 | `UPDATE NODE#<u128> SET <campo> = <valor>, ...` \| `UPDATE NODE#<u128> SET VECTOR [f32,...]` | `:323-353` | **`SET` es obligatorio** (`:327`) |
| 4 | `DELETE NODE#<u128>` | `:355-360` | |
| 5 | `RELATE NODE#<a> --"<etiqueta>"--> NODE#<b> [WEIGHT <f32>]` | `:362-383` | El label usa la forma estructural `--"…"-->`, no un string literal |
| 6 | `SELECT` | `:556-722` | |
| 7 | `FROM` / `MATCH` | `:196-269` | Intercambiables |

### 1.2 Condiciones (`grammar.rs:67-97`) — 3 variantes, 2 semánticas muy distintas

| Variante | Sintaxis | Línea | Significado real |
|---|---|---|---|
| `VectorSim` | `<campo> ~ "<texto>" , min = <float>` | `:70-81` | Embedding + HNSW, score ≥ `min` |
| `TextMatch` | `<campo> ~ "<texto>"` | `:83-86` | **Léxico**: `PhysicalTextFilter` → `text_contains_query` sobre el campo `String` (`physical_plan/filter.rs:137-146`), sin score, sin índice, y **falso silencioso** si el campo no es `String` |
| `Relational` | `<campo> <relop> <literal>` | `:92-95` | |

Relop: `==`, `!=`, `>=`, `<=`, `>`, `<`, `=`. **No hay `AND`/`OR`/`NOT` como operadores booleanos**
(`AND` es separador de lista, `:209`; `NOT` no existe). No hay paréntesis ni negación en el `WHERE` de
`FROM`/`MATCH` (los paréntesis solo en subqueries de `SELECT`).

### 1.3 Literales (`lexer.rs:158-172`)

`String` (doble comilla obligatoria) · `Bool` · `Null` · `Int(i64)` (con fallback a `Float` fuera de
rango) · `Float(f64)`. **`float`/`double` de `nom` acepta `nan`, `inf`, `infinity`**
(`nom-7.1.3/src/number/complete.rs::recognize_float_or_exceptions`).

### 1.4 Cláusulas de `FROM`/`MATCH` (en este orden)

`FROM|MATCH` → `<entidad>` → `[SIGUE min..max "<etiqueta>" [TYPE t] [AS alias]]` → `[alias]` →
`[AS OF ms]` → `[WHERE c AND c…]` → `[FETCH f, f…]` → `[RANK BY f [DESC]]` → `[WITH TEMPERATURE f]` →
`[ROLE "<rol>"]` → `[PROFILE modo [rrf_k n] [candidate_k n]]` → `[AS OF ms]` → `[LIMIT n [OFFSET m]]` → `[AS OF ms]`.

### 1.5 `SELECT`

`*` | campos | agregados → `FROM <entidad> [alias]` → `[JOIN … ON l = r]…` → `[AS OF]` →
`[WHERE item AND item…]` → `[GROUP BY <campo>]` → `[WITH TEMPERATURE]` → `[AS OF]` → `[LIMIT/OFFSET]` → `[AS OF]`.
**No** tiene `PROFILE` ni `ROLE`.

### 1.6 Agregados (`:500-520`)

`COUNT(*)` · `COUNT(<campo>)` · `SUM(<campo>)`. UPPERCASE, con `(` como frontera de identificador.

### 1.7 Flags (`mod.rs:20-48`)

`IQL_VERSION = 4`; gates `MIN_PROFILE=1`, `MIN_AS_OF=2`, `MIN_PAGINATION=3`, `MIN_AGGREGATION=4`;
`iql_supports()` es el único interruptor de versión.

### 1.8 Palabras reservadas (`lexer.rs:41-76`) — 35 entradas

`AND AS DELETE DESC FETCH FROM GROUP INSERT JOIN LIMIT MATCH MESSAGE OF OFFSET ON PROFILE RANK RELATE ROLE
SELECT SET SIGUE SYSTEM TEMPERATURE TIMEOUT TIPO TO TYPE UPDATE USER VECTOR WEIGHT WHERE WITH`.
**No** incluyen `MIN`, `RRF_K`, `CANDIDATE_K`, `BY`, `ASSISTANT`. La comparación es **case-sensitive**
(`:81`), mientras que el autocomplete (`grammar.rs:786-790`) es case-**insensitive**.

## 2. Hallazgos

> "Confirmado" = leído en el código o en la fuente de `nom 7.1.3`. "Sospechado" = no ejecutable sin `cargo`.

### F-01 · **CRÍTICO** · El `WHERE` entero se descarta en silencio si la **primera** condición no parsea
`grammar.rs:207-210` + `grammar.rs:462-475` + `executor.rs:217-218`

Traza verificada en `nom 7.1.3`: `separated_list1` devuelve `Err(Err::Error)` si falla el **primer**
elemento (pero `Ok` con lista parcial si falla uno posterior), y `opt(...)` ante `Err::Error` devuelve
`Ok((input_original, None))` — **restaura el input**.

Con `FROM Doc WHERE n =`: (1) `tag("WHERE")` ok → (2) `separated_list1` → la condición falla con
`Err::Error` → (3) `opt` lo traga y **restaura** el input → (4) `parse_query` sigue sin matchear
nada más → (5) `reject_trailing_input` (`grammar.rs:737-753`) no es estricto sin `LIMIT`/`OFFSET`
ni agregados → (6) `executor.rs:218` devuelve `Ok` → **el resto se descarta**.

**Resultado:** `FROM Doc WHERE n =` ejecuta `FROM Doc` → **escaneo completo sin filtro y sin error**.
El usuario cree que filtró y recibe la tabla entera. `PhysicalScan::open` (`scan.rs:49-60`)
materializa **todos** los nodos en `Vec<UnifiedNode>` antes de filtrar (`scan.rs:76-80`) → escala a
agotamiento de memoria (viola `memory-budget.md` R-3/R-4: estructura residente sin cap).

Asimetría: `FROM Doc WHERE n = 1 AND m =` sí aplica `n = 1` (falla el 2.º elemento) y pierde `m =`;
`FROM Doc WHERE n =` pierde **todo** el `WHERE`. Dos gramáticas de fallo para el mismo error de tipeo.

**Fix:** no usar `opt` para el `WHERE`; propagar el error (`Err::Failure`) cuando el keyword `WHERE` ya
se consumió. Alternativa estructural (cerrada por F-02): `reject_trailing_input` estricto.

### F-02 · **ALTO** · Todo el *trailing input* de un statement no estricto se descarta en silencio
`grammar.rs:737-753` + `executor.rs:218` + `server/handlers.rs:126` + `vantadb-mcp/.../tools.rs:1907`

`reject_trailing_input` solo es estricto si hay `LIMIT`/`OFFSET` o agregados; en el resto de casos el
resto del string se pierde sin error en **las tres superficies**. Lo fija un test propio:
`mod.rs:1710-1712` ("statements WITHOUT pagination keep the legacy trailing-input behavior
(still tracked as FIND-312)").

Casos reales, todos silenciosos:
- `FROM Doc WHERE a = 1, b = 2` → la coma no es separador (solo `AND`) → aplica `a = 1`, **se pierde `b = 2`**.
- `FROM Doc WHERE bio ~ "x", min >= 0.88` → el branch vectorial falla → cae a `TextMatch` → **degradación semántica vectorial→léxica sin error**.
- `FROM Doc WHERE bio ~ "x", minimum = 0.5` → `ws(tag("min"))` matchea el prefijo de `"minimum"` → cae a `TextMatch`.
- `FROM Doc PROFILE bogus` → `search_profile = None` → la query corre **sin perfil** (el propio test lo documenta: `mod.rs:773-780`; el comentario de diseño dice "el caller puede rechazar tokens sobrantes" y **el caller no lo hace**).
- `FROM Doc PROFILE KEYWORD` → los modos son lowercase → misma caída silenciosa.
- `INSERT NODE#1 TYPE X { a: 1 } LINK NODE#2` → nodo insertado, **sin arista**, sin error.

**Fix:** `reject_trailing_input` estricto para **todas** las statements (tolerando solo `;`).

### F-03 · **ALTO** · `~` tiene dos semánticas incompatibles y la doc solo describe una
`grammar.rs:70-86` vs `docs/api/IQL.md:272-278`

La forma `~ "<texto>"` sin `min` (→ `TextMatch` → `text_contains_query` sobre el campo `String`) **no
aparece en `IQL.md`**, que afirma que `~` "triggers embedding-based similarity matching". Un usuario
que escriba `WHERE text ~ "neural network"` creyendo semántica sin umbral obtiene un **match léxico de
frase sobre el campo crudo**. Y `WHERE edad ~ "25"` sobre un `Int` → **siempre falso, 0 filas, sin error**.
`openapi.yaml:181-184` **sí** distingue las dos formas; `HTTP_API.md:262` las lista como equivalentes.
**Fix:** añadir a `IQL.md` la subsección "Text Match (léxico)" y reescribir la prosa de "Vector Similarity".

### F-04 · **ALTO** · La ventana de candidatos vectoriales está hardcodeada en 5 → `LIMIT` es inoperante
`physical_plan/vector.rs:128` — `search_nearest(..., 5, ...)` con el `k=5` literal.
`FROM x WHERE t ~ "q", min = 0.1 LIMIT 1000` devuelve **≤ 5 filas**. Es `FIND-313` (severidad real
mayor: el tope **no escala con `LIMIT`**, y el planner tiene un **segundo** path vectorial (refine,
fuerza bruta) sin el tope → el mismo query da conjuntos distintos según el CBO (F-05)). Además, el
`Limit` del plan se aplica **después** del operador vectorial → el cap es invisible.
**Fix:** derivar `k` de `plan.limit` (con un piso) o exponer un knob.

### F-05 · **ALTO** · La misma expresión IQL produce **resultados distintos** según el CBO
`planner.rs:337-401` (conmutador en `:373`)

Dos paths vectoriales: `PhysicalVectorRefine` (fuerza bruta, escanea **todos** los nodos, coseno
siempre) y `PhysicalVectorSearch` (HNSW, `k=5`). El conmutador es
`joint_selectivity < HIGH_SELECTIVITY_THRESHOLD (0.1)` **y** que haya filtros relacionales. Consecuencias:
(1) el mismo `WHERE` devuelve un conjunto de filas distinto según la cardinalidad estimada (que cambia
con el tamaño del dataset); (2) **espacio de score distinto**: el refine usa siempre coseno, el HNSW usa
la métrica del índice (Euclidean → valores **negativos**) → con índice Euclidean, `min = 0.88` **no
matchea nada**. `min` no tiene espacio de score documentado.
**Fix:** unificar la métrica de scoring de los dos caminos o normalizar `min` a un espacio único.

### F-06 · **ALTO** · `min = nan` / `min = inf` se aceptan → 0 resultados silenciosos
`grammar.rs:78` + `nom-7.1.3/.../recognize_float_or_exceptions` — `ws(float)` acepta `nan`/`inf`/`infinity`
y no hay validación de rango de `min_score`. `score >= NaN` es siempre `false` → 0 filas, sin error, sin
warning. Igual con `x = nan` en `parse_literal_field_value` (`lexer.rs:170`).
**Fix:** validar `min_score` (`0.0..=1.0`, `is_finite()`) y rechazar literales no finitos.

### F-07 · **ALTO (DoS)** · Recursión sin límite en subqueries anidados → stack overflow
`grammar.rs:437-459` ↔ `:462-475` ↔ `:614-617`

Ciclo `parse_select` → `separated_list1(…, parse_where_item)` → `parse_subquery_condition_inner` →
`parse_select` → … **sin contador de profundidad**. El cap del MCP es 1 MiB
(`vantadb-mcp/src/config.rs:142`) → hasta ~30.000 niveles caben; el stack de un hilo de tokio es 2 MiB.
**Fix:** contador de profundidad (thread-local o parámetro explícito) con `Failure` al superar un tope (p. ej. 8).

### F-08 · **MEDIO** · `FROM NODE#<no-numérico>` devuelve un nodo **arbitrario de toda la base**
`lexer.rs:30-36` + `physical_plan/scan.rs:41-47` + `:73` — `ident` acepta `#` en medio, así que
`FROM NODE#abc` parsea; el `parse::<u128>()` falla → cae al escaneo completo → `scan.rs:73` (la entidad
contiene `#`) → **devuelve el primer nodo vivo de toda la base**, sin error. En una superficie
agent-facing, un modelo que escriba `FROM NODE#agent42` recibe un nodo con confianza incorrecta.
**Fix:** si la entidad tiene forma `X#<no-numérico>` → `Error::Validation` explícito.

### F-09 · **MEDIO** · `SIGUE` con profundidad fuera de `u32` → la traversía se descarta en silencio
`grammar.rs:32-34` + `:200` — `parse_number` es `u32`; `SIGUE 1..5000000000` → `map_res` falla →
`opt(parse_traversal)` → `traversal = None` → (con F-02) la query se convierte en un **escaneo sin
traversing**. Contraste: `LIMIT` con el mismo problema **sí** falla ruidosamente (`:163-169`).
Igual clase: `SIGUE 5..2` (min > max) parsea sin error y produce 0 resultados.

### F-10 · **MEDIO** · Una keyword en minúscula se consume como **alias** y la cláusula siguiente se pierde
`lexer.rs:41-82` + `grammar.rs:202` — `RESERVED_KEYWORDS` está en mayúsculas y `non_keyword_ident`
compara exactamente → `where`, `limit`, `offset`, `rank`, `by`, `min`, `assistant` no son reservadas:
- `FROM Doc where x = 1` → alias `"where"`, `where_clause = None`, resto descartado → **todos** los `Doc`.
- `FROM Doc limit 5` → alias `"limit"`, el `5` se pierde → **el `LIMIT` desaparece**.
- `FROM Doc rank by x` → alias `"rank"` → **el `RANK BY` desaparece**.

La doc afirma lo contrario: `docs/api/IQL.md:58-59` ("lowercase spelling is a parse error, not an
accepted alias"), y el test que supposedly lo fija (`mod.rs:1093-1105`) solo cubre keywords en posición
inicial, **no** en posición de alias.
**Fix:** comparar con `eq_ignore_ascii_case` y añadir las keywords que faltan.

### F-11 · **MEDIO** · `PROFILE candidate_k` se parsea y se **ignora** en el path IQL
`grammar.rs:228,262` + `planner.rs:276-277` — el AST lo expone, el planner IQL **nunca lo lee** (su
propio comentario lo dice); solo lo consume `sdk/search/fusion.rs:44`. `IQL.md:109` lo documenta como
"candidate budget" → **la doc afirma un efecto que el path IQL no tiene**.

### F-12 · **MEDIO** · El mensaje de error de parseo embebe el input crudo del usuario
`executor.rs:100-137` — el fallback es el `Display` de `nom::error::Error` ("Error {code} in {input}"):
un query de 1 MiB (permitido por `max_query_length`) produce un mensaje de ~1 MiB devuelto al cliente.
No es fuga de datos internos (es el input del propio usuario) pero es amplificación de payload.

### F-13 · **BAJO** · `min` no es palabra reservada y `AS` no existe como cláusula en `FROM`
`lexer.rs:41-76` + `grammar.rs:200-232` — `FROM Doc min` → alias `"min"`; `FROM Doc AS x` → `"AS x"`
queda como trailing → descartado.

### F-14 · **BAJO** · Cobertura de tests: falta todo el contractual de F-01/F-02/F-10
`mod.rs:50-1914` — los 1.865 líneas de tests cubren bien la superficie explícita, pero **fijan como
esperado** los silent-drops (`mod.rs:1712` `assert!(parse_statement("FROM Doc garbage").is_ok())`;
`:773-780`; `:826-827`). **No hay** ningún test para: `WHERE` con condición no parseable (F-01),
`WHERE a = 1, b = 2` (F-02), `min >= 0.88` (F-02), `min = nan` (F-06), `FROM Doc where x = 1` (F-10),
`SIGUE 1..5000000000` (F-09), `FROM NODE#abc` (F-08), ni **ningún test de límite de profundidad**
(F-07). El parser no tiene test de propiedad ni fuzz.

## 3. Comparación doc ↔ parser

### 3.1 Formas documentadas que el parser **rechaza**

| # | Doc | Forma | Por qué no parsea |
|---|---|---|---|
| D-1 | `docs/api/HTTP_API.md:264` | `UPDATE NODE#<id> VECTOR [...]` | `SET` es obligatorio (`grammar.rs:327`); la forma real es `UPDATE NODE#<id> SET VECTOR [...]` (`IQL.md:296` y `openapi.yaml:201` sí son correctos) |
| D-2 | `docs/api/HTTP_API.md:266` | `RELATE <a> -> <b> AS <label> [WEIGHT <f32>]` | La forma real es `RELATE NODE#<src> --"<label>"--> NODE#<dst> [WEIGHT <n>]` (`grammar.rs:362-383`): la de `HTTP_API.md` **no parsea en absoluto** |
| D-3 | `docs/user/QUICKSTART.md:265` | `VECTOR_SEARCH('hola mundo')` | No existe esa función en ningún lenguaje del repo; además la línea dice "SQL auto-embed" y no hay superficie SQL. **Corregido el 2026-10-21** (la forma real es `<campo> ~ "hola mundo", min = 0.88`) |

### 3.2 Formas que el parser acepta y la doc **no documenta**

| # | Comportamiento | Dónde |
|---|---|---|
| P-1 | `~ "<texto>"` sin `min` → match léxico de frase sobre el campo `String` | `grammar.rs:83-86` → `filter.rs:137-146` (ausente de `IQL.md`; presente en `openapi.yaml:181-184` y `HTTP_API.md:262`, sin distinguirlas) |
| P-2 | `~` con umbral mal formado (`min >= 0.88`, `min = abc`, `minimum =`) → degrada a léxico | `grammar.rs:70-86` + F-02 |
| P-3 | `min = nan` / `inf` | `grammar.rs:78` |
| P-4 | `PROFILE` con modo inválido o en mayúsculas → cláusula ignorada | `grammar.rs:272-278` |
| P-5 | `PROFILE candidate_k` sin efecto en el path IQL | `grammar.rs:262` + `planner.rs:276-277` |
| P-6 | `FROM NODE#<id>` — el `#` es sintaxis de `ident`, no una forma documentada de "read by id" | `lexer.rs:30-36` + `scan.rs:41-47` (lo documenta `skills/vantadb-mcp/SKILL.md:221`) |
| P-7 | `FROM <entidad>` con `#` en el nombre → devuelve **toda** la base | `scan.rs:73` |
| P-8 | Keyword en minúscula aceptada como alias | `lexer.rs:41-82` (`IQL.md:58-59` afirma lo contrario) |
| P-9 | `SIGUE 5..2` (min > max) → 0 resultados, sin error | `grammar.rs:32-34` |
| P-10 | `UPDATE NODE#<id> SET VECTOR [...]` (la forma vectorial de `UPDATE`) | `grammar.rs:329-340` (`IQL.md:296` sí la documenta) |
| P-11 | `AS OF` en subquery = error de parseo | `grammar.rs:444-449` (`IQL.md:137` sí lo documenta) |

### 3.3 Divergencias de semántica

| # | Doc | Realidad |
|---|---|---|
| S-1 | `IQL.md:278` — "`~` triggers embedding-based similarity matching" | `~` sin `min` es contención léxica de frase (F-03) |
| S-2 | `IQL.md:110` — "`LIMIT <n>` … keep at most `n` rows" | En el path vectorial el tope real es **5** (F-04); `IQL.md:240-243` lo matiza, pero la línea 110 lo afirma sin excepción |
| S-3 | `IQL.md:109` — `candidate_k` = "candidate budget" | Sin efecto en IQL (F-11) |
| S-4 | `IQL.md:109` — `PROFILE keyword\|vector\|hybrid` | `keyword` **descarta** el brazo vectorial y `vector` **descarta** los text filters (`planner.rs:278-284`): un `PROFILE` puede eliminar silenciosamente una condición del `WHERE` |
| S-5 | `IQL.md:11` y `:504` — "parsed by the Nom-based parser at `src/parser/mod.rs`" | La gramática está en `grammar.rs` + `lexer.rs`; `mod.rs` solo tiene constantes de versión y tests |
| S-6 | `IQL.md:462-474` — formato `IQL parse error at line <line>, col <col>` | El mensaje cae al `Display` de `nom` para todo lo que no sea `Verify`/`Fail` (F-12): el formato no es estable |

## 4. Qué NO se pudo verificar

1. **Umbral exacto de stack overflow** (F-07): el ciclo recursivo sin límite está confirmado leyendo; los
   niveles que caben en 2 MiB no se midieron (requiere ejecutar).
2. **Runtime del planner** (F-04, F-05): el conmutador CBO y el `k=5` literal están confirmados por
   lectura; **no** se ejecutó una query que demuestre el distinto conjunto de filas: es una derivación
   sólida del código, no una medición.
3. **Escala de score por backend/métrica** (F-05, F-06): la conversión está confirmada
   (`index/search/nearest.rs:198-202`, el refine usa coseno siempre); no se verificó con un índice
   Euclidean real que `min = 0.88` da 0 filas.
4. **`text_contains_query`**: se usa en `PhysicalTextFilter` y es *phrase/term containment* según
   `query-dsl.md` R-1/R-2, pero no se leyó la implementación: no se afirma nada más fino.
5. **`Error::DimensionMismatch` en el path IQL**: rastreado a `error.rs:146-153` y a sus usos, pero **no**
   se localizó el punto donde el mismatch de dimensión de un **query** vectorial se convierte en error:
   en el refine, `cosine_similarity(q_vec)` devuelve `None` ante dimensión distinta y **descarta el
   nodo en silencio** (`vector.rs:223-228`). Sospecha (no confirmada): el `DimensionMismatch` se
   aplica al camino de **escritura/auto-embed**, no al de **query**.
6. **Fuzzing**: no se ejecutó `cargo fuzz` ni tests; el análisis de cobertura es por lectura.
7. **SAST**: no se ejecutaron los gates de `FIND-149` (`.opencode/configs/semgrep-vanta.yml`).

## 5. Estado de los hallazgos de la campaña del 2026-10-20

| ID | Estado | Evidencia |
|---|---|---|
| **FIND-502** (esta auditoría) | **CONFIRMADO, y esta auditoría es su contenido** | `docs/dev/Backlog.md`; informe consolidado §5.3 área 1 |
| **FIND-312** (`execute_hybrid` ignora el trailing input) | **CONFIRMADO y grave** = F-02; el subcase de "trailing interno en subquery" es **F-01**, con consecuencia peor (se pierde el `WHERE` completo) | `executor.rs:218`; `grammar.rs:737-753`; `mod.rs:1710-1712` |
| **FIND-313** (ventana de candidatos hardcoded) | **CONFIRMADO** = F-04, con agravante no registrado (el tope no escala con `LIMIT` y el path de refine no lo tiene) | `physical_plan/vector.rs:128`; `planner.rs:373-400` |
| **FIND-316** (agregaciones avanzadas) | **CONFIRMADO** como deuda de alcance y **parcialmente invalidado**: el multi-field `GROUP BY` y la paginación de grupos **ya se rechazan en voz alta** (`grammar.rs:630-635`, `:670-675`); lo que queda es funcionalidad faltante | `grammar.rs:624-689` |
| **FIND-413** (el brazo vectorial tragaba fallos de embedding) | **CONFIRMADO como CORREGIDO** (4 tests lo fijan) — pero adyacente y vivo: el `min` mal formado (F-06) y el `~` degradado a léxico (F-02/F-03) producen el **mismo síntoma** ("0 resultados sin error") por una vía que FIND-413 no cubre | `physical_plan/vector.rs:117-122` |
| **S3-B10** (`VECTOR_SEARCH` no existe) | **CONFIRMADO**; el finding se marcó cerrado (FIND-445) sin arreglar `QUICKSTART.md:265` → **corregido el 2026-10-21** | `docs/user/QUICKSTART.md:265` |
| **S3-B9** (brazo IQL limitado a 5 candidatos) | **CONFIRMADO** = F-04 = FIND-313 | `physical_plan/vector.rs:128` |
| **VDB-02** (escala de score por backend) | **CONFIRMADO y AGRAVADO** para el path IQL: el mismo fenómeno llega por `min =` (F-05); `min` no tiene espacio de score documentado | `index/search/nearest.rs:198-202`; `docs/api/IQL.md:272-278` |
| **FIND-490** (un fallo del brazo vectorial tumba la query) | **CONFIRMADO**, no auditado aquí (es de `physical_plan`/`planner`) | — |
| **FIND-484/485/486/489/491** | Fuera de alcance | — |

## 6. Nota de cierre (Ponytail lite)

El 80 % de esta auditoría se reduce a **una sola regla**: `opt(...)` alrededor de una cláusula cuyo
keyword ya se consumió convierte un error de tipeo en un cambio de significado silencioso. Arreglar
`reject_trailing_input` para que sea estricto en todas las statements (F-02) cierra de una vez
F-01, la mitad de F-09, `PROFILE bogus`, `min >= 0.88`, `WHERE a = 1, b = 2` y `INSERT … LINK …`.
Lo demás (profundidad, `nan`, `candidate_k`, docs) son correcciones locales de 1-5 líneas cada una.
