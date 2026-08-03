# INV-009 — Phrase Queries + Term Positions (diseño)

- **Estado:** DISEÑO — parcialmente implementado (ver Gate 2026-08-03)
- **Dominio:** vanta-engine (text index, query execution, snippets)
- **Alcance:** NO implementación. Diseño de sintaxis IQL, phrase matching, evaluación tantivy vs custom, integración snippets.

---

## 1. Estado actual verificado (2026-08-03)

La infraestructura de **positions y phrases YA EXISTE** y está en producción:

| Componente | Referencia | Estado |
|---|---|---|
| `TextPosting { node_id, tf, positions: Vec<u32> }` | `src/text_index.rs:98-105` | ✅ serializado en postings |
| `TextRecordTerms { token_counts, token_positions: BTreeMap<String, Vec<u32>>, doc_len }` | `src/text_index.rs:133-141` | ✅ extraído en indexación |
| `TextQueryPlan { terms: BTreeSet<String>, phrases: Vec<Vec<String>> }` | `src/text_index.rs:144-150` | ✅ frases ya modeladas |
| `posting_value(node_id, tf, positions)` | `src/text_index.rs:555` | ✅ escribe positions en bytes |
| `posting_put_ops` | `src/text_index.rs:602-625` | ✅ persiste positions por token |
| `text_positions_match_phrase` (orden + adyacencia) | `src/sdk/search/phrase.rs:28-50` | ✅ lógica pura de matching, 12 tests |
| `text_positions_match_phrases` (todas las frases) | `src/sdk/search/phrase.rs:13-20` | ✅ variante ALL |
| `matched_phrases_for_record` (carga postings + matchea) | `src/sdk/search/debug.rs:156-183` | ⚠️ solo usado en `explain_hit` (1 caller) |
| `query_plan` extrae frases de comillas dobles | `src/text_index.rs` (test `query_plan_extracts_phrases_and_terms:731-738`) | ✅ tokenización de frases YA funciona |
| `generate_snippet_with_highlighting` | `src/sdk/search/snippet.rs:29-84` | ⚠️ highlight por término individual, NO por frase |
| Test `spec_declares_phrase_ready_text_index_v3` | `src/text_index.rs:820` | ✅ contrato v3 |

### El gap real (3 huecos concretos)

1. **Sintaxis IQL phrase** — el parser de VantaQL (`src/parser/mod.rs`) no tiene condición de texto. Solo `Relational` y `VectorSim` (`parse_condition`, `src/parser/mod.rs:135-156`). Las comillas dobles existen en el parser (`string_literal`) pero solo como *valores* de condición, no como frase de búsqueda en un campo de texto.
2. **Evaluación de phrases en query execution** — `matched_phrases_for_record` es solo *explicación* de un hit ya retornado (debug.rs). Ningún código de búsqueda usa `text_positions_match_phrases` como filtro/score durante la ejecución.
3. **Snippets** — `highlight_terms` (`src/sdk/search/snippet.rs:92-130`) resalta términos sueltos; no hay highlight de frase completa.

**Conclusión:** la extensión del text index ya está cubierta por el storage custom. Positions se serializan por token (`posting_value`), se decodifican (`decode_posting`, `src/text_index.rs:564`), y el matching de orden/adyacencia ya está implementado y testeado en `phrase.rs`. **No falta nada del lado de datos.**

---

## 2. Diseño de sintaxis IQL phrase (comillas dobles)

### Sintaxis propuesta

```
FROM Person p WHERE text(p.bio) ~ "machine learning" AND age > 30
```

Análisis:
- **Reuso total:** `string_literal` ya parsea comillas dobles escapadas en el parser actual (`src/parser/mod.rs`, usado en `parse_condition` y `parse_traversal`).
- **Condición nueva:** `Condition::TextMatch(field, query: String)` en `src/query.rs`, donde `query` es el string crudo entre comillas (puede contener varias frases y términos sueltos).
- **Parseo de frases dentro del string:** el trabajo pesado YA existe — `query_plan(query)` (`src/text_index.rs`) divide el string en `terms` + `phrases` usando comillas internas. El parser IQL solo necesita entregar el string crudo; no parsea frases a nivel de gramática.

### Ejemplos concretos

| Query IQL | `query_plan` resultante | Semántica |
|---|---|---|
| `text ~ "machine learning"` | terms=`{machine, learning}`, phrases=`[["machine","learning"]]` | Frase exacta adyacente |
| `text ~ "machine learning" AND text ~ "vector index"` | 2 frases | Ambas frases deben matchear (ALL) |
| `text ~ "machine learning" AND text ~ rust` | 1 frase + 1 término suelto | Frase exacta + término libre |
| `text ~ "part-time"` | tokenización con `-` → phrase de 1 token | A definir: 1 token = término, no frase (ya cubierto por términos) |

### Reglas de tokenización

1. Todo string entre comillas dobles en el campo text es un **candidato a frase**.
2. `query_plan` ya aplica el tokenizador (default o `advanced-tokenizer` con stopwords/stemming) a *cada token de la frase*. ⚠️ **Riesgo:** stemming/stopword-removal dentro de frases puede romper adyacencia (ej: "the quick brown fox" → stopwords removidos → positions ya no son consecutivas). Decisión de diseño: para frases, usar tokenización **sin** stopword removal ni stemming (solo lowercase + fold), porque la frase es literal. Esto es un ajuste a `query_plan_with_config`, no al storage.
3. Frase de 1 token tras tokenización = se trata como término normal (ya cubierto).
4. La posición base usada para adyacencia es la del tokenizer **de indexación**, no del query. Si el tokenizer de indexación difiere del de query, la adyacencia se verifica contra `positions` reales del posting — correcto por diseño.

---

## 3. Diseño de phrase matching en ejecución

### Algoritmo (ya existe, se reusa tal cual)

`text_positions_match_phrase` (`src/sdk/search/phrase.rs:28-50`):
- Carga `positions` del primer token de la frase.
- Para cada posición `start`, verifica que el token i-ésimo de la frase tenga una ocurrencia en `start + i` (adyacencia + orden).
- Costo: `O(|positions_first| × |phrase|)`, con `contains` binario en cada `Vec<u32>` ordenado.

### Dónde se conecta en la ejecución

Estado actual: `bm25_terms_for_record` (debug.rs) y el scoring BM25 operan sobre `query_plan.terms`; las phrases se ignoran en ranking.

Diseño propuesto (mínimo viable, 2 fases):

**Fase 1 — Filtro (recall correcto):**
- En la ejecución de búsqueda text (el path que llama a BM25 por término), después de calcular el candidate set por términos, filtrar cada documento candidato con `text_positions_match_phrases(term_positions, &query_plan.phrases)`.
- Necesita cargar postings de los tokens de cada frase (exactamente lo que ya hace `matched_phrases_for_record`, debug.rs:166-175 — extraer a una función reutilizable `load_phrase_term_positions(engine, namespace, key, &query_plan)`).
- Si `phrases` está vacío → skip (cero overhead).

**Fase 2 — Score (opcional, futuro):**
- Bonus de score por frase matcheada (ej: `phrase_hit_bonus` sumado al BM25). No requerido para recall; deja el hook.

### Propuesta de firma (solo diseño, no implementado)

```rust
// en src/sdk/search/ o src/text_index.rs
/// Carga positions de los tokens de todas las frases del plan.
pub fn load_phrase_positions(
    engine: &StorageEngine,
    namespace: &str,
    key: &str,
    plan: &TextQueryPlan,
) -> Result<BTreeMap<String, Vec<u32>>>;

/// Filtra un doc candidato: todas las frases deben matchear.
pub fn doc_matches_phrases(
    engine: &StorageEngine,
    namespace: &str,
    key: &str,
    plan: &TextQueryPlan,
) -> Result<bool>;
```

`doc_matches_phrases` = `load_phrase_positions` + `phrase::text_positions_match_phrases`. Reusa `decode_posting` existente.

### Correctitud

- Adyacencia estricta (slop=0) es el default correcto para frases entre comillas; si luego se quiere proximidad relajada, es extender `phrase.rs` con slop — no tocar storage.
- Update/delete de postings ya escribe/borra positions completas (`posting_delete_ops`), no hay estado parcial.

---

## 4. Evaluación tantivy vs storage custom (VantaFile)

**Veredicto: storage custom. NO agregar tantivy.**

### Verificación contra docs oficiales (docs.rs tantivy 0.26.1, 2026-07-10)

`tantivy::query::PhraseQuery` (docs.rs/tantivy/latest/tantivy/query/struct.PhraseQuery.html):
- `PhraseQuery::new(Vec<Term>)` — mínimo 2 términos, mismo field.
- **"Using a `PhraseQuery` on a field requires positions to be indexed for this field."** → tantivy exige positions; nosotros ya las tenemos.
- `set_slop(u32)` — relajación de proximidad; nuestro matching actual es slop=0 (equivalente al default de tantivy).

Es decir: tantivy *implementa exactamente lo que ya implementamos* en `phrase.rs` con 12 tests, y encima exige un schema tantivy separado, un índice duplicado, y sincronización de escrituras con VantaFile.

### Análisis costo/beneficio

| Criterio | tantivy | custom (actual) |
|---|---|---|
| Positions | ✅ require index | ✅ ya en `TextPosting` |
| Phrase matching | ✅ `PhraseQuery` (slop=0 default) | ✅ `text_positions_match_phrase` + 12 tests |
| Query parsing | QueryParser propio (gramática distinta a IQL) | ✅ `query_plan` extrae frases de comillas |
| Storage | Índice separado en disco (schema, segments, merge) | ✅ misma VantaFile/BackendPartition |
| Serialización portable | Segmentos tantivy | ✅ postcard, endianness-aware |
| Inserción incremental | requeriría re-commit por doc | ✅ `posting_put_ops` directo |
| Transacciones/WAL | integrar WAL propio con tantivy = complejidad | ✅ ya integrado |
| Dependencia | +tantivy y ~40 crates transitivas | 0 |
| Híbrido vector+text | RRF sobre dos índices separados | ✅ un solo engine |

### Conclusión ponytail: YAGNI

El storage custom ya satisface el 100% del alcance (positions + matching). tantivy **no agrega valor funcional** — duplica storage, introduce un segundo indexador con tokenización propia (incompatible con el `advanced-tokenizer` existente), rompe la serialización portable actual y agrega ~40 dependencias transitivas. El único feature de tantivy que no tenemos (slop, proximity search) es un post-process en `phrase.rs` de ~20 líneas si algún día se pide. **No tocar Cargo.toml.**

---

## 5. Diseño de integración con snippets (highlight de frase completa)

### Problema actual

`generate_snippet_with_highlighting` (`src/sdk/search/snippet.rs:29-84`):
- Calcula el window de recorte alrededor del **primer término** del query (`snippet.rs:35,47`).
- `highlight_terms` (`snippet.rs:92-130`) envuelve cada término suelto en `<strong>`.
- Resultado para `"machine learning"`: `...the <strong>machine</strong> <strong>learning</strong> model...` — visualmente pobre, no muestra que es una frase.

### Diseño propuesto (3 cambios, todo en `snippet.rs` + reuso de `phrase.rs`)

**1. Priorizar frase sobre término para el window de recorte:**
- Si `query_plan.phrases` no está vacío, localizar la **primera ocurrencia de una frase** en el payload (folded) y recortar alrededor de *ella* (start/end del span completo de la frase), en vez del primer término.
- Localización: re-tokenizar el payload con el mismo tokenizer de indexación → obtener positions → usar `text_positions_match_phrase` para hallar el span. Alternativa más simple (correcta para payloads cortos): buscar la secuencia de tokens de la frase como substring contigua en el payload folded.

**2. Highlight de frase completa:**
- Nueva función `highlight_phrases(text, &[Vec<String>])` que envuelve el **span completo** de la frase en un solo `<strong>...multi word...</strong>`.
- Después aplicar `highlight_terms` para los términos sueltos restantes (no solapados con frases).
- Resultado esperado: `...the <strong>machine learning</strong> model...`

**3. Fallback:**
- Sin frases en el query → comportamiento actual intacto (0 regresión).
- Frase no encontrada → cae al primer término (comportamiento actual).

### Ejemplo concreto

Query: `text ~ "machine learning"`
Payload: `"A machine learning system for vector search."`

- Antes: `<strong>machine</strong> <strong>learning</strong>`
- Después: `<strong>machine learning</strong>`

### Reuso de positions

El highlight no necesita cargar postings del storage (el payload está en memoria en `VantaMemoryRecord.payload`). Usar positions solo si se quiere exactitud con tokenizer; para v1, substring folding sobre el payload es suficiente y más barato (0 I/O).

---

## 6. Orden de implementación sugerido (para el backlog, fuera de alcance de este doc)

1. `Condition::TextMatch(field, query)` en `src/query.rs` + `parse_condition` en `src/parser/mod.rs` (reuso `string_literal`). *(SINTAXIS)*
2. Extraer `load_phrase_positions` de `matched_phrases_for_record` (debug.rs) a función reutilizable; aplicar filtro de frases en la ejecución de búsqueda text. *(MATCHING)*
3. Ajustar `query_plan_with_config` para frases sin stopwords/stemming (literal). *(CORRECTITUD)*
4. `highlight_phrases` + priorizar frase en window de snippet. *(SNIPPETS)*

Estimación: 2-3 días, 0 dependencias nuevas, 0 cambios de storage/serialización.

---

## Gate 2026-08-03

**Estado: PARCIALMENTE IMPLEMENTADO.**

- ✅ **Ya implementado (verificado en código, NO es diseño):** positions por documento (`TextPosting.positions`, `TextRecordTerms.token_positions`), serialización en postings (`posting_value`, `posting_put_ops`), modelo de frases en `TextQueryPlan.phrases`, extracción de frases por comillas en `query_plan`, matching de orden+adyacencia (`phrase.rs`, 12 tests), explicación de frases en debug (`matched_phrases_for_record`), contrato `spec_declares_phrase_ready_text_index_v3`.
- ⚠️ **No implementado (diseño pendiente de backlog):** condición `TextMatch` en parser IQL, enforcement de frases como filtro/score en query execution, highlight de frase completa en snippets.
- ✅ **Decisión tomada:** storage custom, sin tantivy (YAGNI, ver §4).
- **Recomendación al próximo ejecutor:** arrancar por §6 paso 1 (sintaxis), el resto tiene cero fricción con el storage existente.

---

*Generado por vanta-engine — INV-009, 2026-08-03.*
