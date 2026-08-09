# VantaDB IVF Benchmark — REVISAR-01

> **Última actualización:** 2026-08-09
> **Versión evaluada:** v0.5.0 (local build, `src/index/ivf.rs` — k-means Forgy + Lloyd, max 20 iters, conv < 1e-4)
> **Build config:** release bench profile (`cargo bench`), AVX2, 12 cores
> **Benchmark:** `benches/ivf_bench.rs` — `IvfIndex::build` + `IvfIndex::search`, recall vs brute-force cosine
> **Reproducir:** `cargo bench --bench ivf_bench -- --quick` (modo corto) · `cargo bench --bench ivf_bench` (completo)

Cierra el ciclo ERR-038/039/040/041 con un bench dedicado: construcción IVF, búsqueda,
y recall@10 contra ground-truth brute-force (coseno sobre los 10K vectores).

---

## 1. Dataset sintético

- **N:** 10,000 vectores · **D:** 128 · **Queries:** 200 · **Top-K:** 10 · **Seed:** 42
- Vectores uniformes normalizados en la esfera 128-d (sin estructura de clusters real).
  ⚠️ Datos sintéticos uniformes = caso peor para IVF (el clustering no encuentra
  separación natural). Los números de recall NO son comparables a datasets reales
  tipo GloVe/SIFT (ver COMPETITIVE_ANALYSIS.md).
- Ground truth: brute-force coseno sobre todo el dataset (10.0µs/query scan floor).

## 2. Build (k-means)

| nlist | Build Time (s) |
|-------|---------------|
| 25    | 0.414         |
| 100   | 1.472         |
| 400   | 3.769         |

Observación: build **lineal en nlist** (~3.7ms por centroide extra). El loop de Lloyd
re-asigna los 10K vectores contra todos los centroides en cada iteración.

## 3. Search: nlist × nprobe sweep

| nlist | nprobe | Recall@10 | p50 (µs) | p99 (µs) | Mean (µs) | QPS    | Cand/q |
|-------|--------|-----------|----------|----------|-----------|--------|--------|
| 25    | 1      | 0.1230    | 52.5     | 167.5    | 58.2      | 17,184 | 401    |
| 25    | 5      | 0.4165    | 218.9    | 692.8    | 267.1     | 3,744  | 2,009  |
| 25    | 10     | 0.6765    | 389.6    | 1,044.5  | 432.0     | 2,315  | 4,006  |
| 100   | 1      | 0.0695    | 30.7     | 174.7    | 37.3      | 26,802 | 101    |
| 100   | 5      | 0.2375    | 64.8     | 182.1    | 71.1      | 14,069 | 503    |
| 100   | 10     | 0.3710    | 115.5    | 307.7    | 129.0     | 7,750  | 1,002  |
| 400   | 1      | 0.0405    | 53.9     | 138.9    | 58.6      | 17,068 | 25     |
| 400   | 5      | 0.1320    | 68.3     | 195.3    | 71.4      | 14,006 | 125    |
| 400   | 10     | 0.2095    | 86.0     | 185.8    | 88.6      | 11,290 | 250    |

## 4. Interpretación (datos reproducibles para el ciclo ERR-038/039/040/041)

1. **Recall bajo en datos uniformes:** máx 0.68 (nlist=25, nprobe=10). Esperado —
   sin estructura de clusters, IVF pierde contra el scan completo. En datasets
   reales con clustering (GloVe/SIFT) el recall es sustancialmente mayor.
2. **CPIndex default** (`nlist = sqrt(n)+1 ≈ 101`, `nprobe = 10`) → recall ≈ 0.37,
   QPS ≈ 7,750 en este dataset. Es el punto de operación por defecto de
   `search_ivf` (src/index/search.rs:688).
3. **nprobe escala el scan linealmente** (Cand/q: 401 → 2,009 → 4,006 para nlist=25)
   y la latencia con él (~7× de nprobe=1 a 10).
4. **Sobre-recuerdo de nlist alta:** con nlist=400, nprobe=10 aún cubre solo 250
   candidatos (2.5% del dataset) → recall 0.21. Confirmado: para recall alto, el
   knob dominante es **nprobe**, no nlist.

## 5. Comandos

```powershell
# Modo corto (CI / smoke)
cargo bench --bench ivf_bench -- --quick

# Completo
cargo bench --bench ivf_bench

# Con salida de tablas siempre visible
cargo bench --bench ivf_bench -- --nocapture
```