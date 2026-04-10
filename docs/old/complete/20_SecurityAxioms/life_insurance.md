# Axiomas de Seguridad: Life Insurance (Seguro de Vida)

## 1. Objetivo
Respaldar todo el grafo antes de eventos potencialmente destructivos que ejecutan los LLMs. Actúa como el botón de "Guardado de Seguridad" en juegos para prevenir fallos críticos.

## 2. El Snapshot RocksDB

Al estar basado el storage engine en `rocksdb`, podemos usar la funcionalidad nativa de RocksDB Checkpoints para crear enlaces duros en el FileSystem que toman 0 segundos físicamente y cuestan 0 bytes hasta que diverjan.

### Flujo de Vida Útil
```rust
// Implementación sugerida en src/storage.rs
use rocksdb::checkpoint::Checkpoint;

pub fn create_life_insurance(&self, timestamp_name: &str) -> Result<(), ConnectomeError> {
    let cp = Checkpoint::new(&self.db)?;
    let snapshot_dir = format!("./connectome_snapshots/{}", timestamp_name);
    cp.create_checkpoint(&snapshot_dir)?;
    Ok(())
}
```

## 3. Disparadores del Seguro (Triggers)
¿Cuándo debe el motor instanciar el seguro obligatorio?
- Inmediatamente antes de un Barrido del Garbage Collector mayor (cuando hay > 10,000 nodos en cola).
- Antes de activar una macro `DROP DATABASE` o una purga de un Sub-Grafo completo.
- Comando explícito de CLI: `connectome-cli backup create "Pre-Experiment-Z"`.

## 4. Recuperación
Un simple script/lógica para levantar el StorageEngine apuntando a la carpeta de `connectome_snapshots`, permitiendo al usuario revivir su grafo semántico si destruyó todo accidentalmente jugando con las mutaciones asíncronas.
