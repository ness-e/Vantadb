# DESKTOP-01 — Tauri como plataforma desktop para VantaDB

- **Tipo:** Investigación / Arquitectura (sin código)
- **Fecha:** 2026-08-04
- **Fuente:** `docs/Backlog.md` línea 166
- **Decisión que informa:** ¿Es Tauri la plataforma desktop correcta para una app AI privada con memoria local sobre el core `vantadb`?
- **Estado:** ✅ Recomendación emitida (ver §9)

---

## 1. Contexto VantaDB

`vantadb` es la crate core Rust de VantaDB, un motor vectorial y grafo con persistencia local (Fjall por defecto, RocksDB fallback) y WAL. La API de integración directa es `VantaEmbedded::open_with_config(VantaConfig)`:

- **`vantadb-python`** la expone vía PyO3 (feature `python_sdk`).
- **`vantadb-wasm`** la expone vía WASM con persistencia en `opfs` / `idb` / `worker`.
- **`vantadb-ts`** es un wrapper TS sobre el build WASM.

Para una app desktop, la pregunta central es: ¿usar **Tauri** (Rust nativo, `vantadb` como dependency directa del backend, sin bridge WASM) o **Electron** (requiere pasar por el TS SDK vía WASM)?

Esta investigación responde esa pregunta. **No toca código Rust/web** — es solo documentación y recomendación de arquitectura.

---

## 2. Estado actual de Tauri v2 (agosto 2026)

- **Tauri v2 es estable desde octubre 2024** y en agosto de 2026 está en **la línea 2.11** — el último release es `tauri v2.11.5` (01 jul 2026) en el repo oficial `tauri-apps/tauri`. Source: [GitHub releases — tauri-apps/tauri](https://github.com/tauri-apps/tauri/releases).
- **Stack Rust:** el backend de la app es Rust compilado a binario nativo; no se empaqueta un runtime (Node.js). Tauri usa **el webview nativo del sistema** en vez de empaquetar Chromium. Source: [What is Tauri? — v2.tauri.app/start](https://v2.tauri.app/start/).
- **Plataformas:** Linux, macOS, Windows, Android e iOS desde un solo codebase. Source: [Tauri homepage](https://tauri.app/).
- **Tamaño mínimo:** "a minimal Tauri app can be less than 600KB in size" (usa el webview que el sistema ya trae). Source: [What is Tauri? — Smaller App Size](https://v2.tauri.app/start/).
- **Seguridad:** permitidos por capacidades (capability-based security model) — el frontend solo accede a APIs nativas que le autorizas explícitamente. Source: [What is Tauri? — Secure Foundation](https://v2.tauri.app/start/) y [Tauri v2 vs Electron 2026 — buildmvpfast](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026).

---

## 3. Integración de una crate Rust nativa como backend (command API / events)

Tauri **es ideal** para integrar `vantadb` directamente: expone las funciones Rust del backend hacia el frontend (web) vía **commands** (`invoke`), con un subsistema de **events** y **channels** para comunicación dinámica y streaming. Source: [Calling Rust from the Frontend — v2.tauri.app/develop/calling-rust](https://v2.tauri.app/develop/calling-rust/).

### Cómo se llama Rust desde el frontend (patrón verificado en docs oficiales)

1. **`#[tauri::command]`** anota una función Rust en `src-tauri/src/lib.rs`:
   ```rust
   #[tauri::command]
   fn my_custom_command() { println!("I was invoked from JavaScript!"); }
   ```
2. Se registra en el builder con `.invoke_handler(tauri::generate_handler![my_custom_command])`.
3. El frontend la llama con `invoke` (paquete `@tauri-apps/api/core`):
   ```js
   import { invoke } from '@tauri-apps/api/core';
   invoke('my_custom_command', { /* args */ }).then(...);
   ```
   
Las commands aceptan argumentos y devuelven valores **siempre que implementen `serde::Serialize`/`Deserialize`**, manejan errores (retorno `Result<T, E>`), y pueden ser **async** (se ejecutan fuera del main thread vía `async_runtime::spawn` — evita freezes de UI con trabajo pesado). Source: [Calling Rust from the Frontend — Commands](https://v2.tauri.app/develop/calling-rust/).

- **Event system:** `emit`/`listen` entre frontend y Rust, con payloads JSON. No es type-safe ni devuelve valores; para alto throughput se usa **Channels**. Source: [Calling Rust / Calling the Frontend from Rust](https://v2.tauri.app/develop/calling-rust/) y [v2.tauri.app/develop/calling-frontend](https://v2.tauri.app/develop/calling-frontend/).
- **Managed State:** `tauri::Builder::manage()` + `tauri::State<T>` permite compartir estado entre commands (ej: una instancia `VantaEmbedded` en memoria). Source: [Calling Rust from the Frontend — Accessing Managed State](https://v2.tauri.app/develop/calling-rust/).
- **Acceso a FS nativa:** el backend Rust lee/​escribe el sistema de archivos real, con rutas tipo `app_handle.path().app_dir()` — eliminando la capa OPFS del WASM. Source: [Calling Rust from the Frontend — AppHandle](https://v2.tauri.app/develop/calling-rust/).
- **Responses binarias:** `tauri::ipc::Response` permite devolver bytes (array buffers) sin overhead de JSON — útil para embeddings blobs. Source: [Calling Rust — Returning Array Buffers](https://v2.tauri.app/develop/calling-rust/).

### Patrón de integrar una DB / crate nativa como dependency

Tauri soporta plugins oficiales y de comunidad (SQL, store, fs, notifications, deep-link, shell, updater, etc.) — ver lista en [Tauri Plugins — v2.tauri.app/plugin](https://v2.tauri.app/start/). Pero **para VantaDB no se necesita plugin alguno**: basta con añadir `vantadb` al `Cargo.toml` de la app Tauri (`src-tauri/`), crear el estado `VantaEmbedded` en el setup, y exponer commands delgados (ingest/search) que delegan en esa instancia. Es el mismo patrón que `vantadb-python` y `vantadb-wasm` ya usan con `VantaEmbedded::open_with_config`, solo que sin capa FFI/WASM de por medio. Contexto del crate verificado en el task file (§25-27).

---

## 4. Casos de uso: desktop AI app privada con memoria local

VantaDB es local-first: embeddings + grafo + búsqueda híbrida corren en la máquina del usuario con persistencia en disco. Eso encaja con la tendencia 2026 de apps AI desktop que priorizan **privacidad y fuera-de-línea**:

- **Tauri es la dirección recomendada para apps desktop nuevas en 2026** por tamaño, RAM y postura de seguridad. Source: [Tauri v2 vs Electron 2026 — buildmvpfast](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026).
- **Superficie de ataque menor:** el frontend web solo alcanza las funciones nativas autorizadas por capacidad; Rust es el core. Para datos sensibles (memoria personal, embeddings) es un argumento real, no de marketing. Source: [buildmvpfast §Where Tauri clearly pulls ahead](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026).
- **Ejemplo real de app local/fuera-de-línea sobre Tauri:** la app GIS open-source **GeoLibre** corre browser + desktop + móvil desde un único codebase con **React + TypeScript + Tauri (Rust) + DuckDB**. Confirma que el patrón "frontend web + backend Rust + DB embebida nativa" es producción en Tauri v2. Source: [buildmvpfast — cita @giswqs](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026).

Para una "AI app privada con memoria local", el backend Rust de Tauri ejecuta directamente `vantadb` (persistencia por Fjall/RocksDB en el filesystem del usuario), los embeddings se generan/almacenan en la máquina, y la UI web (React/Svelte/Vite, configurado por [Tauri Frontend Config](https://v2.tauri.app/start/frontend/)) llama a commands async para ingest/search. **Sin necesidad de servidor HTTP ni de OPFS.**

---

## 5. Comparativa Tauri v2 vs Electron (agosto 2026)

Versiones validadas en docs oficiales (agosto 2026):
- **Tauri:** v2.11.5 estable. Source: [GitHub releases](https://github.com/tauri-apps/tauri/releases).
- **Electron:** v43.2.0 estable (Chromium 150, Node.js 24.18.0); prerelease 44.0.0-alpha. Source: [Electron Releases — releases.electronjs.org](https://releases.electronjs.org/).

| Aspecto | Tauri v2 | Electron | Fuente |
|---------|----------|----------|--------|
| **Bundle size (instalador)** | **2–10 MB** (mín. <600KB para app vacía) | **80–200 MB** | [buildmvpfast](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026); [Tauri docs](https://v2.tauri.app/start/) |
| **Bundle size (benchmark app demo)** | **8.6 MiB** | **244 MiB** | [gethopp.app benchmark](https://www.gethopp.app/blog/tauri-vs-electron) |
| **RAM en idle** | **~50 MB** (benchmark 172 MB con 6 ventanas) | **~120 MB+** (benchmark 409 MB con 6 ventanas) | [buildmvpfast](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026); [gethopp](https://www.gethopp.app/blog/tauri-vs-electron) |
| **RAM startup** | menor (sin arranque Chromium) | 200–500 MB al inicio | [oflight comparison](https://www.oflight.co.jp/en/columns/tauri-v2-vs-electron-comparison) |
| **Backend** | **Rust** (binario nativo) | Node.js/JavaScript | [gethopp](https://www.gethopp.app/blog/tauri-vs-electron) |
| **Motor de render** | **WebView nativo del SO** (WebView2/WKWebView/WebKitGTK) | **Chromium empaquetado** | [gethopp](https://www.gethopp.app/blog/tauri-vs-electron); [buildmvpfast](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026) |
| **Consistencia de render entre SO** | ⚠️ Varía (WebKit en macOS/Linux tiene quirks) | ✅ idéntica en todos (mismo Chromium) | [buildmvpfast §Where Electron still wins](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026) |
| **Soporte móvil (iOS/Android)** | ✅ Sí (mismo codebase) | ❌ No (solo desktop) | [buildmvpfast](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026) |
| **Build time (primer build, N=1)** | 🐢 ~1m21s (compilación Rust) | 🏎️ ~16s | [gethopp benchmark](https://www.gethopp.app/blog/tauri-vs-electron) |
| **Sistema de plugins** | Oficial + comunidad (SQL, store, fs, notification, deep-link, updater, shell…) | npm masivo (20 años de ecosistema) | [Tauri Plugins](https://v2.tauri.app/start/); [buildmvpfast](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026) |
| **Auto-update / signing** | ✅ Updater oficial, ecosistema más joven | ✅ Maduro/battle-tested (differential updates, staged rollouts) | [buildmvpfast §Where Electron still wins](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026) |
| **Madurez en producción** | Roster más joven (GeoLibre, apps indie; IDE multi-window ~5MB a la semana) | Máxima (VS Code, Slack, Discord, Notion) | [buildmvpfast §Who ships on each](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026) |
| **DX del stack** | Requiere algo de Rust (la UI suele ser 90% del trabajo) | JavaScript/NPM conocido | [buildmvpfast](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026) |

**Advertencia sobre benchmarks (gap honesto):** los números de bundle size y RAM varían según fuente y app concreta. El benchmark de [gethopp](https://www.gethopp.app/blog/tauri-vs-electron) es **N=1** (una sola ejecución) y el propio autor pide tomarlo "with a grain of salt"; [forasoft](https://www.forasoft.com/blog/article/electron-desktop-app-development-guide-for-business) también cuestiona los ratios "96% smaller / 5x less RAM" que se citan a ciegas. Los **rangos** (Tauri 2–10MB vs Electron 80–200MB; RAM ~2–3× menor) son consistentes entre fuentes, pero los números exactos son aproximados y dependientes de la app. Para VantaDB habría que correr un benchmark propio (nativo fallback no contemplado — ver §8 gap G2).

---

## 6. Vía de integración recomendada (SI Tauri, sin WASM/OPFS)

Arquitectura objetivo para una desktop app de VantaDB:

```
┌────────────────────────────────────────────────┐
│ Frontend web (React/Svelte/Vite) — src/         │
│   llamadas invoke('vanta_ingest'|'vanta_search')│
└───────────────────────┬────────────────────────┘
                        │ @tauri-apps/api/core (invoke, events)
┌───────────────────────▼────────────────────────┐
│ Backend Rust — src-tauri/                       │
│   tauri::Builder::manage(VantaEmbedded)         │
│   #[tauri::command] async fn vanta_ingest(...)  │
│   #[tauri::command] async fn vanta_search(...)  │
│        │                                        │
│        ▼                                        │
│   vantadb (dependency directa del Cargo.toml)   │
│   VantaEmbedded::open_with_config(VantaConfig)  │
│   persistencia Fjall/RocksDB en FS nativa       │
└────────────────────────────────────────────────┘
```

1. **Scaffold** de la app con `npm create tauri-app@latest` (elige frontend Vite + tu framework). Source: [Tauri Create a Project](https://v2.tauri.app/start/create-project/).
2. **Añadir `vantadb`** como dependency en `src-tauri/Cargo.toml` (con las features de persistencia necesarias: `fjall` o `rocksdb`). **Sin bridge WASM.**
3. En el **setup** del builder, crear `VantaEmbedded::open_with_config(VantaConfig)` contra el directorio de datos de la app (ej. `app_handle.path().app_dir()`/`app_data_dir()`), y guardarlo con `.manage(...)`.
4. Exponer **commands async** delgados: `vanta_ingest` (documentos → embeddings → nodos), `vanta_search` (query híbrida → resultados), `vanta_delete`, etc. — cada uno accede al estado `VantaEmbedded`.
5. Para ingest larga, usar **channels** para progreso en streaming, o commands async (no bloquean el main thread). Source: [Calling Rust — async/channels](https://v2.tauri.app/develop/calling-rust/).

**Beneficios frente a Electron + WASM:**
- Elimina la capa **OPFS** del WASM: el backend Rust escribe directamente en el filesystem del usuario (durabilidad real por Fjall/WAL, no emulación browser).
- Cero overhead de serialización WASM ↔ JS en el hot path de búsqueda; tipos nativos Rust→serde→JSON directo.
- Acceso directo a FS, tray, notificaciones, deep-link, updater vía plugins oficiales Tauri. Source: [Tauri Plugins](https://v2.tauri.app/start/).
- Bundle chico (10–20MB con `vantadb` incluido, vs 80–200MB Electron).
- Posibilidad futura de target móvil (iOS/Android) con el mismo frontend.

**Riesgo clave a gestionar (ver §7):** variabilidad del webview entre Windows (WebView2/Chromium) y macOS/Linux (WebKit). La UI de la app debe mantenerse en CSS/HTML convencional y probarse en CI sobre cada webview. Source: [buildmvpfast §Where Electron still wins](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026).

---

## 7. Riesgos y mitigaciones

| Riesgo | Impacto | Mitigación |
|--------|---------|------------|
| **Inconsistencia de render entre webviews** (WebKit macOS/Linux vs WebView2/Chromium Windows) | Medio — bugs de UI solo en algunas plataformas | Mantener la UI simple (CSS estándar), feature-detection, testear en CI sobre cada webview. Source: [buildmvpfast](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026) |
| **Build primero-lento** (compilar `tauri` + `vantadb` vía Rust) | Bajo–Medio — fricción DX en CI/dev | Builds incrementales rápidos tras el primero; perfiles release/dev optimizados. Source: [gethopp build-time](https://www.gethopp.app/blog/tauri-vs-electron) |
| **Cadena de dependencias Rust grandes** (GTK/WebKit en Linux; FTPs para `rocksdb` si se usa) | Medio — impacto en tiempo build y binario | Feature-select en `vantadb` para no arrastrar todo; `fjall` (pure Rust) como default para desktop. Contexto crate §2. |
| **Menor madurez de production que Electron** en ecosistema de release/signing | Bajo para VantaDB (app single-dev, MVP) | Tauri v2 ya tiene updater oficial; Electron solo se justifica si se requiere el pipeline signing más maduro. Source: [buildmvpfast](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026) |
| **Comandos async con tipos prestados** (limitación `&str`/`State` en async) | Bajo — fricción de implementación | Usar `String`/`Result` en firmas async, o commands síncronos cortos. Source: [Calling Rust — async borrowed args](https://v2.tauri.app/develop/calling-rust/) |
| **RAM/performance de la app depende del webview del SO** | Bajo — con Solo UI suele sobrar | Benchmark propio con la app real antes de decidir UI pesada. |

---

## 8. Effort estimate MVP (desglose)

Estimación de ingeniería basada en: setup Tauri documentado ([scaffold](https://v2.tauri.app/start/)), existencia de `VantaEmbedded::open_with_config` ya usada por Python/WASM (factible en el task file §25-27), y command API ya verificada (§3). **Nota de método:** esto es una **estimación**, no una cifra de benchmark — asume 1 dev Rust+TS con el repo VantaDB ya construido.

| Paso | Tarea | Effort | Depende de |
|------|-------|--------|------------|
| 1 | Scaffold Tauri v2 + frontend Vite (React/Svelte) | 🟢 0.5–1 día | — |
| 2 | `vantadb` como dep en `src-tauri/` + compile-check cross (Windows/macOS/Linux CI) | 🟢 0.5–1 día | 1 |
| 3 | Setup de `VantaEmbedded` en el builder + manage() | 🟢 0.5 día | 2 |
| 4 | Commands backend: `vanta_ingest` + `vanta_search` (+ metadata/delete) con manejo de error serde | 🟡 1–2 días | 3 |
| 5 | Transporte de modelos: embeddings en base64/arrays + `tauri::ipc::Response` para blobs | 🟡 1 día | 4 |
| 6 | UI mínima: input de ingest, box de búsqueda, lista de resultados, directorio de datos | 🟡 2–3 días | 4 |
| 7 | Persistencia en FS nativa (`app_data_dir()`), validación de re-abierto tras reinicio | 🟢 1 día | 4 |
| 8 | Empaquetado/instalador (Windows/macOS/Linux) + updater opcional | 🟡 1–2 días | 6 |
| 9 | Pruebas en los 3 webviews (CI) + ajuste de CSS cross-platform | 🟡 1–2 días | 6 |
| | **TOTAL MVP** | **≈ 8–13 días hábiles** (2–3 semanas) | |

**Composición aprox.:** scaffold 10%, integración crate/commands 30%, UI web 30%, persistencia 10%, empaquetado/CI 20%.

---

## 9. Recomendación final

**✅ SÍ — Tauri v2 es la plataforma desktop correcta para VantaDB**, mediante la **vía de integración nativa Rust**: `vantadb` como dependency directa en `src-tauri/`, `VantaEmbedded` en managed state, y commands async delgados (`vanta_ingest` / `vanta_search`) para el frontend web. **Sin bridge WASM ni OPFS.**

**Justificación (1 línea):** para una app AI privada y local-first sobre un core Rust, Tauri elimina la capa de emulación (WASM/OPFS) con durabilidad real en FS nativa, bundle 10–25× menor, RAM ~2–3× menor y postura de seguridad por capacidades — a cambio de un costo de DX (Rust) que VantaDB ya asume de base.

**Cuándo NO usar Tauri (y elegir Electron):** solo si la UI exige render pixel-idéntico en todos los SO (macOS/Linux WebKit es el riesgo), o si se dependiera fuertemente de paquetes npm/Node (no es el caso de VantaDB, cuyo motor es Rust). Source: [buildmvpfast §decision framework](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026).

---

## Gaps documentados

- **G1 — Benchmark propio no realizado.** Los números de bundle/RAM son de fuentes de terceros (gethopp N=1, buildmvpfast rangos). Se necesita un benchmark propio con `vantadb` + UI real para confirmar el impacto antes de validar la decisión final de UI pesada.
- **G2 — Costo de añadir `rocksdb`** (si se elige como backend) no se cuantificó aquí; `fjall` (pure Rust) es el default razonable para desktop.
- **G3 — Modelo de embeddings local** (Ollama/propio) vs remoto no se evaluó; afecta el tamaño del binario y el flujo offline. Informa al diseño de la UI de la app, no a la elección de framework.

---

## Referencias (URLs)

- Tauri docs — What is Tauri: https://v2.tauri.app/start/
- Tauri docs — Calling Rust from the Frontend: https://v2.tauri.app/develop/calling-rust/
- Tauri docs — Calling the Frontend from Rust: https://v2.tauri.app/develop/calling-frontend/
- Tauri docs — Frontend Configuration: https://v2.tauri.app/start/frontend/
- Tauri — GitHub releases (v2.11.5): https://github.com/tauri-apps/tauri/releases
- Tauri homepage: https://tauri.app/
- Electron Releases (v43.2.0): https://releases.electronjs.org/
- gethopp — Tauri vs Electron benchmark (N=1): https://www.gethopp.app/blog/tauri-vs-electron
- buildmvpfast — Tauri v2 vs Electron 2026: https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026
- oflight — Tauri v2 vs Electron comparison: https://www.oflight.co.jp/en/columns/tauri-v2-vs-electron-comparison
- forasoft — crítica a benchmarks citados a ciegas: https://www.forasoft.com/blog/article/electron-desktop-app-development-guide-for-business