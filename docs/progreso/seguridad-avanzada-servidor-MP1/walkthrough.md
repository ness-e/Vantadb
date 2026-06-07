# Walkthrough: MP1 — Seguridad Avanzada del Servidor

> **Estado:** ✅ Completado
> **Componente:** `vantadb-server`, `src/config.rs`

## Resumen de Cambios

Se han implementado con éxito las capas de seguridad para la API HTTP/REST (`vantadb-server`), dotando al ecosistema de características críticas para su despliegue en entornos expuestos.

### 1. Autenticación Bearer Token
- Se incorporó la configuración `VANTADB_API_KEY` (opcional).
- Se implementó `auth_middleware` en `src/middleware.rs`, asegurando que todas las rutas bajo `/api/v2/` requieran el token `Bearer <API_KEY>` si se proporciona.
- La ruta `/health` quedó explícitamente exenta para mantener compatibilidad con probes de Kubernetes y balanceadores de carga.

### 2. Rate Limiting Avanzado (tower-governor)
- Se integró `tower-governor` 0.8 junto a `tower` 0.5 para implementar Rate Limiting distribuible por IP basado en el algoritmo de *Token Bucket*.
- El límite es configurable vía la variable de entorno `VANTADB_RATE_LIMIT_RPM` (Requests Per Minute).
- La capa de middleware de limitación se inyecta dinámicamente (`if rpm > 0`) protegiendo el router de inundaciones (DDoS locales) sin degradar el performance cuando no está activa.

### 3. Cifrado de Transporte (TLS Opcional)
- A través del feature flag `tls` de Cargo, el servidor ahora tiene la capacidad de arrancar bajo un enchufe HTTPS utilizando `axum-server::bind_rustls`.
- Requiere `VANTADB_TLS_CERT_PATH` y `VANTADB_TLS_KEY_PATH`. 
- Si no se activa el feature o no se proveen las llaves, el servidor hace fallback gracefully a TCP HTTP plano.

## Validación y Certificación

1. **Compilación Estática:**
   - ✅ `cargo build -p vantadb-server` completó limpiamente resolviendo la API de configurador estricto (`GovernorConfigBuilder`) requerida por la versión 0.8.
2. **Cleanliness:**
   - ✅ Eliminado warning de variables no utilizadas en contextos condicionales usando flags `#[cfg_attr(not(feature = "tls"), allow(unused_variables))]`.
3. **Commit History:**
   - ✅ `[main dedc479] feat(server): MP1 - Seguridad avanzada (Rate limiting, Bearer Auth, TLS)` registrado.

## Archivo de Progreso

Siguiendo la **POLÍTICA CRÍTICA DE RETENCIÓN DE HISTORIAL (FMEA PREVENTIVO)**, el plan de implementación, la lista de tareas y este walkthrough serán respaldados en el directorio histórico `docs/progreso/seguridad-avanzada-servidor-MP1/` de VantaDB para preservar el contexto de esta integración antes de avanzar a la Fase 5 / MP2.
