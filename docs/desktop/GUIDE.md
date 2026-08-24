# Guía de usuario — Modos de conexión de VantaDB Desktop

VantaDB Desktop (Vanta Studio) se conecta al motor de tres maneras. Elegí el
modo según tu caso de uso — la interfaz es idéntica en los tres.

---

## 1. Nativo embebido (default) — máximo rendimiento

El motor corre **dentro de la app**, sin servidor ni red. Es el modo por
defecto cuando abrís la aplicación de escritorio.

**Cuándo usarlo:** uso personal en una sola máquina; querés la mejor latencia y
cero configuración.

**Cómo conectar:**
1. Abrí VantaDB Desktop.
2. En la pantalla de conexión, elegí **Nativo**.
3. Indicá la carpeta donde vivirán tus datos (ej. `C:\Users\tu\Datos\VantaDB`).
   Si no existe, se crea automáticamente.
4. Conectar. Listo — ya podés ingestar y buscar.

**Buenas prácticas:**
- Un solo proceso puede abrir cada carpeta de datos. Si otro proceso tiene la
  base abierta, verás un error de lock (`Lock`) — cerrá la otra instancia o
  elegí otra carpeta.
- Se genera un log de auditoría automático en `<carpeta>/audit.jsonl`.
- Los datos se escriben a disco con cada operación; al cerrar la app, todo se
  guarda (flush) antes de salir.

---

## 2. Server HTTP — remoto y multi-usuario

La app habla por REST (`/api/v2/*`) con un `vantadb-server` corriendo en otra
máquina o proceso.

**Cuándo usarlo:** datos centralizados compartidos entre varias personas o
dispositivos; el motor vive en un servidor que administrás vos.

**Cómo conectar:**
1. En el servidor, iniciá `vantadb-server` (ver documentación del server para
   puertos y bind). Por defecto escucha solo en loopback (`127.0.0.1`).
2. En la app, elegí **Servidor** e ingresá la URL base
   (ej. `http://127.0.0.1:8080`).
3. Si el server tiene autenticación activada (`api_key` / `require_auth`),
   cargá las credenciales Bearer que te pida la pantalla.
4. Conectar. La salud del servidor se valida al conectar (`/health`).

**Buenas prácticas:**
- Sin auth configurada en el server, los endpoints de loopback funcionan sin
  token (decisión local-first, ADR-026). Nunca expongas el server fuera de
  localhost sin activar auth.
- Si una operación devuelve `401 Unauthorized`, revisá las credenciales Bearer.
- Si devuelve timeout, verificá que el server esté vivo y alcanzable.

---

## 3. WASM-OPFS — standalone, offline, demo

Consola 100% en el navegador: el motor corre compilado a WebAssembly dentro de
la página y persiste en OPFS (sistema de archivos del navegador).

**Cuándo usarlo:** probar VantaDB sin instalar nada; demo portátil; uso
offline en un solo navegador.

**Cómo usarlo:**
1. Serví la build standalone: `npm run build:wasm` genera `dist-wasm/`, y se
   sirve con cualquier servidor estático en `127.0.0.1`.
2. Abrí la página en Chrome/Edge (necesario secure context para OPFS).
3. Todo lo que crees persiste en el navegador: al recargar, tus registros
   siguen ahí.

**Buenas prácticas:**
- La persistencia vive **dentro del navegador**: limpiar datos del sitio borra
  la base. Exportá (`export`) lo que quieras conservar.
- En modo incógnito OPFS no está disponible; la app cae automáticamente a
  IndexedDB.
- No hay conexión remota posible en este modo — es local al navegador.

---

## Problemas comunes

| Síntoma | Qué hacer |
|---------|-----------|
| Error `Lock` al conectar nativo | Otra app/proceso tiene esa carpeta abierta. Cerrala o usá otra ruta. |
| "no active connection" | No hay conexión activa: conectate primero desde la pantalla de conexiones. |
| `401` en modo server | Credenciales Bearer incorrectas o faltantes. |
| Registros desaparecen en WASM | Limpiaste los datos del sitio, o estabas en modo incógnito (IndexedDB efímero). |

Más detalles técnicos: [README](README.md) · [ARCHITECTURE](ARCHITECTURE.md).
