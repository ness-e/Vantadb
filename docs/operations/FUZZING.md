# 🔍 Guía de Fuzzing para VantaDB

## 📊 Estrategia de Validación

VantaDB utiliza un enfoque dual de fuzzing para maximizar cobertura y compatibilidad:

| Método | Plataforma | Propósito | Ejecución |
|--------|-----------|-----------|----------|
| **`proptest`** | ✅ Windows, Linux, macOS | Validación cross-platform en desarrollo | `cargo test fuzz_proptest` |
| **`cargo-fuzz`** | 🐧 Solo Linux/macOS | Fuzzing intensivo con sanitizers (CI) | `cd fuzz && cargo +nightly fuzz run <target>` |

---

## 🪟 Validación en Windows (Proptest)

Ejecuta tests basados en propiedades que generan datos aleatorios para validar robustez:

```powershell
# Ejecutar tests de fuzzing
cargo test fuzz_proptest

# Ejecutar con más casos (por defecto 256)
PROPTEST_CASES=1000 cargo test fuzz_proptest
```

**Qué valida:**
- ✅ Deserialización segura de `WalRecord` y `UnifiedNode` ante bytes corruptos
- ✅ Roundtrip correcto con payloads y vectores aleatorios
- ✅ Cero pánicos en código crítico de parsing

**Criterios de aceptación:**
- Todos los tests de `fuzz_proptest` deben pasar
- No debe haber pánicos ni crashes en la deserialización

---

## 🐧 Fuzzing Avanzado (Linux/macOS + Nightly)

Requiere `cargo-fuzz` y `rustc nightly`. Úsalo en CI o WSL2:

### Prerequisitos
```bash
rustup install nightly
cargo install cargo-fuzz
```

### Ejecución
```bash
cd fuzz/

# Fuzzing de deserialización (WAL + Nodes)
cargo +nightly fuzz run fuzz_node_deserialize -- -max_total_time=300

# Fuzzing de parser LISP/queries
cargo +nightly fuzz run fuzz_parser -- -max_total_time=300
```

### Reproducir Crashes
Si `cargo-fuzz` encuentra un crash, guarda el caso en `fuzz/artifacts/`:
```bash
cargo +nightly fuzz run fuzz_node_deserialize fuzz/artifacts/fuzz_node_deserialize/crash-<hash>
```

### Integración en CI (GitHub Actions)
```yaml
# .github/workflows/fuzz.yml (ejemplo para Linux)
name: Fuzzing
on: [push, pull_request]

jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: rustup install nightly
      - run: cargo install cargo-fuzz
      - run: cd fuzz && cargo +nightly fuzz run fuzz_node_deserialize -- -max_total_time=60
```

---

## 🎯 Criterios de Aceptación en CI

- [ ] `proptest` pasa en todas las plataformas (Windows/Linux/macOS)
- [ ] `cargo-fuzz` corre 300s sin crashes en Linux (no-pánico)
- [ ] 0 memory leaks detectados por LSan/ASan (Linux only)
- [ ] Cualquier crash encontrado se documenta en `docs/audits/fuzz-crashes.md`

---

## ⚠️ Notas Técnicas

### Por qué `cargo-fuzz` no funciona en Windows nativo
- `cargo-fuzz` depende de `libFuzzer`, que requiere sanitizers (ASan/LSan)
- Los sanitizers de LLVM solo están soportados oficialmente en Linux/macOS
- En Windows, usa `proptest` para validación de lógica y reserva `cargo-fuzz` para CI Linux

### Configuración del Workspace
El directorio `fuzz/` está excluido del workspace en `Cargo.toml`:
```toml
[workspace]
exclude = ["fuzz"]  # Evita conflicto con cargo-fuzz en Windows
```

Esto permite:
- Ejecutar `cargo test` en Windows sin errores de workspace
- Ejecutar `cargo +nightly fuzz run` en Linux sin interferencias

---

## 📈 Métricas de Calidad

| Métrica | Objetivo | Herramienta |
|---------|----------|-------------|
| **Cobertura de fuzzing** | >1M inputs sin crash | `cargo-fuzz` (Linux) |
| **Tiempo sin regresiones** | 300s continuos | `cargo-fuzz -- -max_total_time=300` |
| **Tests de propiedad** | 256+ casos por test | `proptest` (default) |
| **Pánicos en deserialización** | 0 | Ambos métodos |

---

> **Nota para desarrolladores**: Si añades nuevos tipos serializables críticos (ej: nuevos records de WAL, estructuras de índice), considera añadir un test de `proptest` correspondiente en este archivo para mantener la cobertura de fuzzing.
