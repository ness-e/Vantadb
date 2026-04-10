# Gobernanza: The Shadow Archive (Memoria Glaciar)

## 1. Objetivo
Evitar la fragmentación física al borrar información y evitar el borrado profundo que limite a los LLMs a la hora de auditar su propio razonamiento defectuoso. "Nunca olvidamos, solo reprimimos".

## 2. RocksDB Column Families (CF)

Debemos modificar `src/storage.rs` para soportar una nueva Column Family auxiliar.

- **Actualmente:** Solo existe la CF `default` (Donde reside todo).
- **Nuevo Requerimiento:** CF `shadow_kernel`.

### Configuración en StorageEngine:
```rust
let mut opts = Options::default();
opts.create_missing_column_families(true);

let cfs = vec!["default", "shadow_kernel"];
let db = DB::open_cf(&opts, path, cfs)?;
```

## 3. El Shadow Worker (Flujo de Archivo)

Cuando una `Neuron` debe ser borrada permanentemente del motor consciente (ejemplo: El TTL expiró fuertemente, o se lanzó un comando DROP lógico).

1. El sistema NO hace un simple `db.delete()`.
2. Hacemos un `db.get(id)` para traer la `Neuron`.
3. Serializamos la estructura usando Bincode.
4. Escribimos la estructura serializada en la Column Family `shadow_kernel` bajo el mismo `id`.
5. Ejecutamos un `db.delete(id)` pero solo en la CF `default`.

**Impacto:** El nodo desaparece completamente de las queries SQL-like (`Cortex Plan`), pero es recuperable si activamos el flag "Modo Subconsciente" analítico.
