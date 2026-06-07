# Fase MP3: Compatibilidad y CLI / MCP

Esta fase es la consolidación final de nuestras integraciones de Seguridad y Observabilidad, asegurando que las nuevas características (OTLP, JSON logs) operen perfectamente con el Model Context Protocol (MCP) y preparando el sistema para su uso real por agentes de IA y clientes externos.

## Proposed Changes

---

### vantadb-server

#### [MODIFY] `vantadb-server/Cargo.toml`
1. **Añadir dependencia `clap`**: Se añadirá `clap = { version = "4", features = ["derive"] }` para soportar banderas estructuradas.

#### [MODIFY] `vantadb-server/src/main.rs`
1. **Refactorización de `init_telemetry`**: 
   - Actualmente, el sistema de logging emite todo hacia la salida estándar (`stdout`) por defecto.
   - **Solución:** Se modificará la firma de `init_telemetry` para aceptar un booleano `is_mcp: bool`. Si es `true`, el inicializador inyectará `.with_writer(std::io::stderr)` en el suscriptor de logs (`fmt_layer`).

2. **Parsing Estructurado con Clap**: 
   - Se reemplazará el código manual `env::args().collect()` por una estructura `ServerCli` derivada con Clap.
   - Banderas soportadas iniciales: `--mcp` (inicia el protocolo de contexto de modelo por stdio) y, opcionalmente, configuración de puertos que sobreescriban el `.env`.


---

### Preparación del Entorno (Documentación)

#### [MODIFY] `.env.example`
- Se actualizará el archivo de variables de entorno de plantilla para documentar explícitamente las nuevas opciones incorporadas en estas fases:
  - `VANTADB_API_KEY`: Para la autenticación Bearer de la API HTTP.
  - `VANTADB_RATE_LIMIT_RPM`: Para el ajuste del rate-limiter (default 100).
  - `VANTADB_LOG_JSON`: Para cambiar la salida a formato JSON.
  - `OTEL_EXPORTER_OTLP_ENDPOINT`: Para el recolector de OpenTelemetry.
  - Opciones de `VANTADB_TLS_CERT_PATH` y `VANTADB_TLS_KEY_PATH` para tráfico HTTPS.

## Verification Plan

### Manual Verification
1. Compilaremos `vantadb-server` usando `--features opentelemetry`.
2. Lanzaremos el comando `cargo run -p vantadb-server --features opentelemetry -- --mcp`.
3. Verificaremos que el banner inicial y cualquier log ("Recovering LSM-tree", etc.) se impriman exclusivamente en `stderr` (en la terminal se ve normal, pero internamente el fd es 2).
4. El servidor deberá quedar congelado esperando input JSON-RPC por consola (`stdin`), demostrando que `stdout` está limpio.
