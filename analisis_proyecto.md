# Reporte de Revisión y Evaluación del Proyecto — VantaDB

Este documento presenta una evaluación de arquitectura, seguridad, concurrencia, calidad de código y estado del roadmap de **VantaDB** a fecha del **2 de julio de 2026**.

---

## 1. Arquitectura y Estructura del Proyecto

VantaDB es un motor de base de datos vectorial y persistencia embebida de alto rendimiento. El proyecto se organiza como un workspace de Cargo con la siguiente estructura de componentes:

```mermaid
graph TD
    subgraph Core Engine [vantadb (Rust Core)]
        Storage[StorageBackend]
        HNSW[HNSW Vector Index]
        BM25[BM25 Lexical Index]
        RRF[RRF Query Planner]
        WAL[WAL + Recovery]
    end

    subgraph Adapters & Bindings
        Py[vantadb-python / PyO3]
        Wasm[vantadb-wasm]
        TS[vantadb-ts]
    end

    subgraph Deployment Wrappers
        Server[vantadb-server / Axum]
        MCP[vantadb-mcp / Protocolo de Contexto de Modelos]
    end

    Py --> Core Engine
    Wasm --> Core Engine
    TS --> Wasm
    Server --> Core Engine
    MCP --> Core Engine
```

### Tabla de Componentes y Estado

| Componente | Descripción Técnica | Estado |
| :--- | :--- | :--- |
| **`src/` (Core)** | Motor base, WAL con CRC32C, planificador RRF, integraciones con Fjall y RocksDB. | 🟢 Estable |
| **`vantadb-python`** | Extensión nativa en Rust con PyO3. Soporta asyncio, stubs de tipado y conversión mediante buffer protocol. | 🟢 Estable |
| **`vantadb-server`** | API REST/gRPC basada en Axum, TLS mediante Rustls y límites de tasa con Tower. | 🟢 Completo |
| **`vantadb-mcp`** | Interfaz MCP para comunicación nativa con agentes de inteligencia artificial. | 🟢 Completo |
| **`vantadb-wasm`** | Compilación WASM del motor, limitado temporalmente a persistencia en memoria (`InMemory`). | 🟡 Limitado |
| **`vantadb-ts`** | Wrapper de TypeScript que consume la compilación WASM o el servidor. | 🟢 Completo |
| **`web/`** | Interfaz del sitio web en React 19 + Tailwind v4 + GSAP/Motion para documentación e interacciones. | 🟢 Completo |

---

## 2. Diagnóstico del Core Engine y Persistencia

### Abstracción de Persistencia
El motor implementa el trait `StorageBackend`, permitiendo desacoplar la base de datos del motor LSM subyacente.
- **Fjall** (por defecto): Excelente rendimiento embebido, escrito en Rust nativo.
- **RocksDB**: Proporciona compatibilidad heredada y alta tolerancia bajo cargas extremas.
- **In-Memory**: Usado para pruebas y en entornos WASM.

### Análisis de la serialización y Riesgos de Formato en Disco
> [!WARNING]
> **Riesgo Crítico de Schema Evolution:**
> Actualmente, las estructuras serializadas en disco (como metadatos, WAL e índices) utilizan `bincode v2.0`. Dado que `bincode` produce salidas compactas pero acopladas a la representación binaria exacta en memoria de las structs de Rust, **cualquier refactorización o cambio de versión en las structs romperá la compatibilidad con bases de datos antiguas**.
>
> VantaDB carece de un subsistema de migraciones físicas de almacenamiento (Physical Storage Migrations). Un salto de versión de v0.1.5 a v0.2.0 sin migración de datos corromperá el almacenamiento de los usuarios.

---

## 3. Vulnerabilidades de Seguridad y Dependencias Inseguras

El backlog del proyecto destaca dos dependencias prioritarias por motivos de seguridad:

1. **`bincode` (RUSTSEC-2025-0141):**
   - **Estado actual:** Se migró de 1.3 a 2.0-rc para solventar vulnerabilidades directas de pánicos. Sin embargo, al ser considerado *unmaintained*, representa una deuda técnica y un riesgo de seguridad a mediano plazo.
   - **Acción propuesta:** Evaluar la migración a `postcard` (seguro, compacto y diseñado para entornos integrados) o `rkyv` (acceso zero-copy, rendimiento extremo).
2. **`rustls-pemfile` (RUSTSEC-2025-0134):**
   - **Uso:** Importado condicionalmente en `src/cli_server.rs` para configurar certificados TLS en el servidor HTTP Axum.
   - **Acción propuesta:** Actualizar las llamadas de parsing a `rustls-pki-types` o encapsular un parser manual simplificado para eliminar la dependencia obsoleta.

---

## 4. Concurrencia y Cuellos de Botella de Rendimiento

El motor implementa protección mediante exclusión mutua en puntos calientes de datos:
- **`insert_lock` en HNSW:** El proceso de inserción requiere bloquear capas de HNSW mediante `RwLock`. En cargas de inserción concurrente masiva, esto crea contención y degrada la latencia.
- **Sugerencia de Optimización:** Implementar una estructura de bloques distribuidos (`sharded-slab`) o un esquema lock-free parcial para los enlaces de HNSW (como se plantea en `TSK-122`), mitigando la contención de escritura en el grafo.

---

## 5. Diagnóstico de los Bindings de Clientes

### Python SDK (`vantadb-python` / PyO3)
- **Ventajas:** Excelente uso del buffer protocol de Python 3.11+ para evitar copias de memoria en la transferencia de arrays NumPy de alta dimensionalidad (`extract_vector`).
- **Desventajas:** La conversión dinámica de payloads JSON o metadatos escalares mediante `py_any_to_value` introduce sobrecarga de paso de mensajes, manteniendo la latencia p50 en ~62ms.
- **Acción:** Optimizar la serialización FFI delegando conversiones masivas directamente a deserializadores de alta velocidad en Rust (como `serde_json::value::Value` o parseadores nativos directos).

### WebAssembly (`vantadb-wasm`)
- **Limitación principal:** Limitado exclusivamente a `BackendKind::InMemory`. Esto imposibilita su uso como base de datos persistente en navegadores.
- **Solución propuesta:** Integrar compatibilidad con Origin Private File System (OPFS) de HTML5 para proporcionar persistencia nativa en disco en el navegador a través de `vantadb-wasm`.

---

## 6. Suite de Pruebas e Infraestructura de CI/CD

El proyecto cuenta con una cobertura admirable: **265 tests funcionales**. La auditoría realizada en junio resolvió la mayoría de las incidencias críticas (como fallos en el runner de Windows y tests desorganizados en CI).

Sin embargo, persisten problemas operativos en la infraestructura de pruebas:

- **`test-threads = 2` global en nextest:**
  Establecido en `.config/nextest.toml` debido a un problema ambiental en Windows con el archivo de paginación (`os error 1455`). Forzar este límite globalmente perjudica los tiempos de ejecución del pipeline en entornos Linux/macOS, donde el paralelismo nativo podría ser sustancialmente mayor.
  * **Solución:** Crear configuraciones de perfiles de nextest específicos del sistema operativo o eliminar la limitación global, delegándola al script de CI de Windows de manera exclusiva.

---

## 7. Recomendaciones y Siguientes Pasos

Para preparar a VantaDB para el lanzamiento formal (v0.2.0) y hacerlo apto para producción, se recomiendan las siguientes prioridades técnicas:

```
┌─────────────────────────────────────────────────────────────┐
│ 1. RESOLVER DEPENDENCIAS SEGURAS (rustls-pemfile, bincode)   │
└──────────────────────────────┬──────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. DISEÑAR ESTRATEGIA DE EVOLUCIÓN DE FORMATOS EN DISCO     │
└──────────────────────────────┬──────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. IMPLEMENTAR PERSISTENCIA OPFS EN WASM                    │
└─────────────────────────────────────────────────────────────┘
```

1. **Resolución de Dependencias Vulnerables:** Migrar `rustls-pemfile` a `rustls-pki-types`.
2. **Definición de Formato de Intercambio Físico:** Implementar un sistema de versión de cabeceras en el almacenamiento que permita actualizar bases de datos creadas con versiones anteriores de `bincode` o reemplazar el motor de serialización por `postcard`.
3. **Persistencia WASM (OPFS):** Investigar e implementar persistencia con OPFS para `vantadb-wasm`, completando un hito único para su uso en aplicaciones web cliente.
4. **Optimización FFI en Python:** Abordar `DX-02` para bajar la latencia p50 de ~62ms a <20ms mediante FFI rápido y deserialización optimizada.
5. **Ajuste de Nextest:** Remover `test-threads = 2` global en nextest para optimizar el paralelismo en entornos Unix.
