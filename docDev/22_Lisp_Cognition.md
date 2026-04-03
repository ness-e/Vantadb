# Fase 22: Cognitive IQL & S-Expressions (Homoiconicidad)

## Meta
Dotar a ConnectomeDB de una capa teórica extremadamente avanzada donde el código es igual a los datos. El motor es capaz de razonar funcionalmente, almacenando nodos que no solo representan "hechos pasivos", sino "reglas de negocio dinámicas" (S-Expressions).

## Mecanismo de Implementación

### 1. Parsing (`src/parser/lisp.rs`)
Se utiliza un parser secundario basado en `nom` que identifica estructuras balanceadas de paréntesis.
- **Átomos**: Identificadores de funciones (`INSERT`, `MATCH`).
- **Keywords**: Metadatos rápidos (`:label`, `:trust`).
- **Variables**: Identificadores dinámicos que comienzan con `?`.
- **Mapas**: Representación de payloads complejos `{ :key "val" }`.

### 2. Sandbox de Ejecución (`src/eval/mod.rs`)
Para prevenir ataques de denegación de servicio (DoS) mediante recursión infinita o bucles lógicos, se implementa el `LispSandbox`.
- **Cognitive Fuel**: Cada paso de evaluación consume 1 unidad de "combustible". El límite por defecto es `1000`. Si se agota, la ejecución aborta con `Sandbox Abort: Out of Cognitive Fuel`.
- **Inmutabilidad**: El evaluador opera sobre `std::borrow::Cow<'_, LispExpr>` para minimizar copias innecesasias durante el descenso recursivo.

### 3. Integración con el Executor
El `Executor` en `src/executor.rs` realiza una detección temprana del string de entrada:
```rust
if trimmed.starts_with('(') {
    // Redirigir al evaluador LISP
} else {
    // Parser IQL estándar
}
```

### 4. Homoiconicidad Transaccional
Los nodos pueden contener S-Expressions como valores de campo. El "Abogado del Diablo" (`DevilsAdvocate`) tiene la capacidad de evaluar estas expresiones antes de permitir una mutación, asegurando que las reglas lógicas no entren en contradicción con el conocimiento ya establecido en el grafo.
