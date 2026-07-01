# VantaDB — Diseño de CLI y TUI
## Especificación Completa: Estructura, Animaciones e Interactividad

> **Fecha:** 2026-06-13 | **Stack:** Rust | **Audiencia:** Developers

---

## Filosofía Base

Un CLI/TUI para developers tiene tres trabajos en orden estricto de prioridad:

1. **Comunicar correctamente** — el output dice exactamente lo que pasó
2. **Comunicar rápido** — la información aparece donde el ojo la busca
3. **Comunicar con carácter** — la herramienta tiene una personalidad visual consistente

La referencia correcta no es "hagamos algo bonito". La referencia es: ¿cómo se siente `cargo build`, `git log --oneline`, o `k9s`? Funcionales, rápidas, sin friction. **La estética al servicio de la claridad, nunca al revés.**

VantaDB tiene una ventaja de identidad poco explotada: **Vanta** viene del griego y del material de absorción de luz más negro que existe (Vantablack). El visual language correcto es dark-first, preciso, con un único acento de color que represente la memoria cognitiva — no los colores de base de datos corporativa.

---

---

# PARTE 1: CLI (Command Line Interface)

## 1.1 — Problemas Actuales

Basado en el estado conocido del CLI:

| Problema | Impacto |
|---------|---------|
| No hay `--format json` en ningún comando | No se puede usar en scripts o pipes |
| No hay barra de progreso en imports/backups | El usuario no sabe si la operación cuelga o avanza |
| Mensajes de error genéricos de clap/Rust | El developer no sabe qué hacer cuando algo falla |
| No hay shell completions | Friction en el día a día de uso |
| Help text sin ejemplos | El usuario tiene que adivinar la sintaxis |
| Sin `--quiet` para scripting | Contamina stdout con output decorativo en scripts |
| Salida sin estructurar | Imposible parsear con `grep`, `jq`, o `awk` |

---

## 1.2 — Estructura de Comandos

La estructura actual se reorganiza siguiendo el patrón de `git` (verbo + objeto) y `cargo` (grupos de comandos relacionados):

```
vantadb
│
├── DATOS
│   ├── put    <key> <texto|archivo>   # Insertar/actualizar registro
│   ├── get    <key>                   # Obtener registro por clave
│   ├── delete <key>                   # Eliminar registro
│   ├── search <query>                 # busqueda-hibrida semántica
│   └── similar <key>                  # Buscar similares a un registro existente
│
├── NAMESPACE
│   ├── ns list                        # Listar namespaces
│   ├── ns create <nombre>             # Crear namespace
│   ├── ns delete <nombre>             # Eliminar namespace (confirmación)
│   ├── ns stats  <nombre>             # Estadísticas de un namespace
│   └── ns rename <viejo> <nuevo>      # Renombrar namespace
│
├── DATOS EN BULK
│   ├── import <archivo>               # Importar desde JSONL/JSON/CSV
│   ├── export <archivo>               # Exportar a JSONL/JSON/CSV
│   ├── backup                         # Snapshot completo con verificación CRC32C
│   └── restore <archivo>              # Restaurar desde backup
│
├── MANTENIMIENTO
│   ├── stats                          # Estadísticas completas de la DB
│   ├── doctor                         # Diagnóstico de salud
│   ├── inspect <key>                  # Inspector de registro completo
│   ├── vacuum                         # Compactar WAL y limpiar TTL expirados
│   └── count                          # Contar registros con filtros
│
├── SERVIDOR
│   ├── serve --http                   # Iniciar servidor HTTP
│   └── serve --mcp                    # Iniciar servidor MCP
│
└── INTERACTIVO
    └── repl                           # TUI interactivo (ver Parte 2)
```

**Flags globales** (disponibles en todos los comandos):
```
--db <path>          Path de la base de datos (default: ./vantadb)
--namespace <ns>     Namespace activo (default: "default")
--format <fmt>       Formato de output: human|json|table|csv (default: human)
--quiet              Suprimir output decorativo (solo resultado)
--no-color           Deshabilitar colores (también respeta NO_COLOR env var)
--timing             Mostrar timing de la operación al final
--help               Ayuda detallada con ejemplos
```

---

## 1.3 — Diseño de Output

### Regla fundamental: dos modos siempre

Cada comando tiene un modo **human** (para la terminal) y un modo **json** (para pipes y scripts):

```bash
# Modo human (default)
vantadb search "privacidad de datos" --top-k 5

# Modo machine
vantadb search "privacidad de datos" --top-k 5 --format json | jq '.results[0].key'
```

---

### Output: `vantadb search`

```
$ vantadb search "política de privacidad" --namespace docs --top-k 5

  busqueda-hibrida en [docs] · 45,230 registros · 12ms

  #  Clave             Score   Extracto
  ─────────────────────────────────────────────────────────────────
  1  policy_001        0.971   "La política de privacidad establece los términos..."
  2  legal_gdpr_v2     0.954   "RGPD — Reglamento General de Protección de Datos..."
  3  compliance_050    0.891   "Cumplimiento normativo en materia de privacidad..."
  4  terms_service_04  0.847   "Los usuarios tienen derecho a solicitar la eliminación..."
  5  onboarding_legal  0.812   "Al registrarte aceptas nuestra política de privacidad..."
  ─────────────────────────────────────────────────────────────────
  5 resultados  ·  vector: HNSW  ·  texto: BM25  ·  fusión: RRF
```

**JSON equivalente:**
```json
{
  "query": "política de privacidad",
  "namespace": "docs",
  "total_records": 45230,
  "latency_ms": 12,
  "results": [
    { "key": "policy_001", "score": 0.971, "excerpt": "La política..." }
  ]
}
```

---

### Output: `vantadb stats`

```
$ vantadb stats

  VantaDB v0.1.4  ·  ./data  ·  uptime: 3d 14h 22m

  ┌─ REGISTROS ──────────────────────────────────┐
  │  Total         45,230 registros              │
  │  Con vector    45,230  (100%)                │
  │  Con texto     44,891  (99.2%)               │
  │  Con grafo      3,401  (7.5%)                │
  │  TTL activos      892                        │
  └───────────────────────────────────────────────┘

  ┌─ NAMESPACES ─────────────────────────────────┐
  │  docs          32,100   ████████████░░░░░  71% │
  │  conversation   8,430   ██████░░░░░░░░░░  18%  │
  │  preferences    4,700   ████░░░░░░░░░░░░  10%  │
  └───────────────────────────────────────────────┘

  ┌─ STORAGE ────────────────────────────────────┐
  │  WAL              128 MB  ·  1,247 entradas  │
  │  HNSW index        58 MB  ·  dim: 768        │
  │  Fjall (datos)    412 MB                     │
  │  Total            598 MB                     │
  └───────────────────────────────────────────────┘

  ┌─ MEMORIA ────────────────────────────────────┐
  │  RSS              234 MB  /  4,096 MB (5.7%) │
  │  mmap (virtual)   892 MB  (no es RAM real)   │
  └───────────────────────────────────────────────┘

  ┌─ PERFORMANCE (últimas 24h) ──────────────────┐
  │  search p50        11.2ms  ·  p99: 47.8ms    │
  │  put throughput    94 ops/s                   │
  │  WAL recovery      <100ms en tests de crash  │
  └───────────────────────────────────────────────┘
```

---

### Output: `vantadb doctor`

```
$ vantadb doctor

  Diagnóstico de VantaDB en ./data

  ✓  WAL          Íntegro · 1,247 registros · CRC32C: OK
  ✓  HNSW index   45,230 vectores · coherente con datos
  ✓  BM25 index   44,891 documentos · coherente con datos
  ✓  File lock    Sin conflictos (ningún otro proceso)
  ✓  Memoria      234 MB RSS · 5.7% del límite configurado
  ⚠  WAL size     128 MB · supera el umbral de 100 MB
     → Ejecuta 'vantadb vacuum' para compactar

  1 advertencia · 0 errores

  ¿Ejecutar vacuum automáticamente? [s/N] _
```

---

### Output: `vantadb import` (con progreso)

```
$ vantadb import ./knowledge_base.jsonl --namespace docs

  Importando knowledge_base.jsonl → [docs]

  Leyendo archivo         ████████████████████  100%   45,230 líneas
  Validando registros     ████████████████████  100%   0 errores
  Indexando vectores      ██████████░░░░░░░░░░   52%   23,450/45,230
  ETA: ~8s

  [Ctrl+C para cancelar · los datos importados hasta ahora se preservan]
```

Después de completar:
```
  ✓  Importación completa

  Registros importados   45,230
  Con vector               45,230
  Sin vector                    0
  Duplicados (ignorados)       47
  Tiempo total             18.3s  ·  2,472 registros/s
```

---

### Diseño de Mensajes de Error

Los errores deben ser **accionables**. Nunca: "Error: file not found". Siempre: qué pasó, por qué, y qué hacer.

**Error malo (como está probablemente ahora):**
```
Error: No such file or directory (os error 2)
```

**Error bueno (como debería ser):**
```
  ✗  No se puede abrir la base de datos

  Path:   ./data/vantadb.wal
  Causa:  El archivo no existe

  ¿Qué hacer?
  · Si es la primera vez: vantadb init --db ./data
  · Si migraste los archivos: usa --db <nuevo-path>
  · Si hubo un crash: intenta vantadb doctor --db ./data
```

**Error de dimensión de vector:**
```
  ✗  Dimensión de vector incorrecta

  Esperado:  768 dimensiones (configurado en este namespace)
  Recibido:  1536 dimensiones (clave: "doc_nuevo_001")

  ¿Qué hacer?
  · Asegúrate de usar el mismo modelo de embedding en todos los puts
  · Si cambiaste el modelo: crea un namespace nuevo con 'vantadb ns create'
```

**Typo en comando (sugerencia):**
```
  ✗  Comando desconocido: 'serach'

  ¿Quisiste decir: search?

  Uso: vantadb search <query> [--namespace <ns>] [--top-k <n>]
  Ver: vantadb search --help
```

---

### Shell Completions

```bash
# Bash
vantadb completions bash >> ~/.bash_completion

# Zsh
vantadb completions zsh > ~/.zsh/completions/_vantadb

# Fish
vantadb completions fish > ~/.config/fish/completions/vantadb.fish

# PowerShell
vantadb completions powershell >> $PROFILE
```

Con completions, el developer puede escribir:
```bash
vantadb se<TAB>          → vantadb search
vantadb search --<TAB>   → --namespace, --top-k, --mode, --format, ...
vantadb ns <TAB>         → list, create, delete, stats, rename
```

---

## 1.4 — Identidad Visual del CLI

### Paleta de colores

VantaDB tiene una personalidad dark-cognitive — memoria, profundidad, precisión.

```
Negro base      #0D0D0D   (fondo en modo TUI)
Violeta acento  #8B5CF6   (títulos, elementos activos, énfasis)
Cyan datos      #22D3EE   (valores, scores, métricas)
Verde éxito     #10B981   (✓ operaciones correctas)
Ámbar aviso     #F59E0B   (⚠ advertencias)
Rosa error      #F43F5E   (✗ errores)
Gris texto      #D1D5DB   (texto normal)
Gris muted      #6B7280   (metadata, secundario)
```

### Iconografía (emojis/símbolos)

Usar solo donde tienen significado semántico real:

```
✓   Éxito (operación completada)
✗   Error (operación fallida)
⚠   Advertencia (requiere atención)
→   Acción sugerida / siguiente paso
·   Separador de metadata (no estructural)
»   Prompts interactivos
```

**Nunca:** emojis decorativos, spinners excesivos, ASCII art en output de producción.

---

---

# PARTE 2: TUI (Terminal User Interface)

El TUI es accesible via `vantadb repl`. Es la herramienta para developers que quieren explorar y debuggear la DB interactivamente — no para scripts.

## 2.1 — Layout Principal

```
┌─────────────────────────────────────────────────────────────────────────┐
│ VantaDB v0.1.4  ·  ./data  ·  45,230 rec  ·  234MB  ·  12ms avg       │ ← Header
├──────────────────┬──────────────────────────────────────────────────────┤
│                  │                                                       │
│  NAMESPACES  [1] │  BÚSQUEDA                                     [2]   │
│  ──────────────  │  ┌─────────────────────────────────────────────────┐ │
│  » docs   32,100 │  │ /  política de privacidad...                    │ │ ← Search bar
│    chat    8,430 │  └─────────────────────────────────────────────────┘ │
│    prefs   4,700 │                                                       │
│                  │  RESULTADOS  (5 de 45,230)                [3]        │
│  [N] nuevo       │  ──────────────────────────────────────────────────  │
│  [D] eliminar    │  » 1  policy_001      0.971  "La política de pri..." │ ← Result list
│  [R] renombrar   │    2  legal_gdpr_v2   0.954  "RGPD — Reglamento..."  │
│                  │    3  compliance_050  0.891  "Cumplimiento norma..."  │
│                  │    4  terms_04        0.847  "Los usuarios tienen..." │
│                  │    5  onboarding_l    0.812  "Al registrarte acep..." │
│                  │                                                       │
│                  │  INSPECTOR                                    [4]    │
│                  │  ──────────────────────────────────────────────────  │
│                  │  Key:         policy_001                             │
│                  │  Namespace:   docs                                   │
│                  │  Score:       0.971                                  │
│                  │  Importance:  0.85                                   │
│                  │  Hits:        47                                     │
│                  │  Created:     2026-03-15 14:23:11                   │
│                  │  Last access: 2026-06-12 09:41:03                   │
│                  │  TTL:         ninguno                                │
│                  │  Metadata:    { "department": "legal",              │
│                  │                 "version": 2,                       │
│                  │                 "author": "juridico@empresa.com" }  │
│                  │  Edges:       → compliance_050  → terms_04          │
│                  │                                                       │
│                  │  Payload:     "La política de privacidad establece  │
│                  │               los términos bajo los cuales recopi-  │
│                  │               lamos y procesamos datos..."           │
│                  │                                                       │
├──────────────────┴──────────────────────────────────────────────────────┤
│ [/] buscar  [i] inspeccionar  [g] grafo  [d] borrar  [e] editar  [?] ? │ ← Status bar
└─────────────────────────────────────────────────────────────────────────┘
```

### Zonas del layout

| Zona | Número | Contenido | Tamaño |
|------|--------|-----------|--------|
| Header | — | DB info, métricas en tiempo real | 1 línea |
| Namespaces | [1] | Browser de namespaces | 20% ancho |
| Búsqueda | [2] | Search bar con modo híbrido | Resto |
| Resultados | [3] | Lista scrollable de resultados | 40% altura |
| Inspector | [4] | Detalle del registro activo | 40% altura |
| Status bar | — | Atajos de teclado contextuales | 1 línea |

---

## 2.2 — Pantallas del TUI

### Vista: Stats Dashboard (`vantadb repl --view stats`)

```
┌─────────────────────────────────────────────────────────────────────────┐
│ VantaDB  ·  Dashboard  ·  actualizado hace 3s                           │
├────────────────────────┬────────────────────────┬────────────────────────┤
│  REGISTROS             │  STORAGE               │  PERFORMANCE           │
│  ─────────────────     │  ─────────────────     │  ─────────────────     │
│                        │                        │                        │
│  45,230   total        │  598 MB   total        │  11ms   search p50     │
│  45,230   vectorizados │  412 MB   datos        │  48ms   search p99     │
│  44,891   con texto    │   58 MB   HNSW         │  94/s   put rate       │
│   3,401   con grafo    │  128 MB   WAL          │                        │
│     892   con TTL      │                        │  LATENCIA (24h)        │
│                        │  RAM REAL              │                        │
│  NAMESPACES            │  234 MB   RSS          │   0 ▁▁▂▃▂▁▂▃▄▃▂▁  50ms│
│  ─────────────────     │  892 MB   mmap (virt.) │                        │
│                        │                        │  WAL WRITES (24h)      │
│  docs    ████████  71% │  ████░░░░░░░░   5.7%   │                        │
│  chat    ██░░░░░░  18% │  de 4,096 MB           │   0 ▂▃▄▄▃▂▃▄▅▄▃▂  100 │
│  prefs   █░░░░░░░  10% │                        │                        │
│                        │  ✓ WAL: íntegro        │                        │
│                        │  ✓ File lock: libre    │                        │
│                        │  ⚠ WAL: 128MB > 100MB  │                        │
└────────────────────────┴────────────────────────┴────────────────────────┘
│  [R] refrescar  [V] vaciar WAL  [S] búsqueda  [Q] salir               │
└─────────────────────────────────────────────────────────────────────────┘
```

---

### Vista: Graph Explorer (`[G]` desde cualquier registro)

Cuando un registro tiene `edges`, el TUI puede mostrar el subgrafo en ASCII con box-drawing characters:

```
┌─ GRAFO: policy_001 (docs) ──────────────────────────────────────────────┐
│                                                                          │
│  Profundidad: 2 hops  ·  8 nodos                                        │
│                                                                          │
│                         ┌─────────────────┐                             │
│                         │  policy_001  ●  │  ← Nodo activo              │
│                         │  score: 1.000   │                             │
│                         └───────┬─────────┘                             │
│                    ─────────────┴───────────────                        │
│                    │                         │                          │
│          ┌─────────▼───────┐       ┌─────────▼───────┐                 │
│          │ compliance_050  │       │    terms_04      │                 │
│          │ score: 0.891    │       │ score: 0.847     │                 │
│          └────────┬────────┘       └────────┬─────────┘                │
│                   │                         │                           │
│          ┌────────▼───────┐       ┌─────────▼───────┐                  │
│          │ gdpr_audit_01  │       │  cookie_policy   │                  │
│          │ score: 0.743   │       │  score: 0.721    │                  │
│          └────────────────┘       └─────────────────┘                  │
│                                                                          │
│  ● Nodo activo   ○ Nodo relacionado                                     │
│                                                                          │
│  [↑↓←→] navegar  [Enter] ir a nodo  [+/-] profundidad  [Esc] volver   │
└──────────────────────────────────────────────────────────────────────────┘
```

---

### Vista: Import Progress (pantalla modal)

Cuando el usuario ejecuta un import desde el TUI:

```
┌─ IMPORTANDO ────────────────────────────────────────────────────────────┐
│                                                                          │
│  knowledge_base.jsonl → namespace [docs]                                │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  Leyendo archivo                                                 │   │
│  │  ████████████████████████████████████████████████████  100%     │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  Indexando vectores                                              │   │
│  │  ██████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░   52%     │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  23,450 / 45,230 registros  ·  2,472 reg/s  ·  ETA: ~9s               │
│                                                                          │
│  ──────────────────────────────────────────────────────────────────     │
│  ✓  Archivo leído      45,230 líneas                                    │
│  ✓  Validación         0 errores                                        │
│  ⟳  Vectores           en progreso...                                   │
│  ○  WAL flush          pendiente                                        │
│                                                                          │
│  [Ctrl+C] cancelar  (los datos indexados hasta ahora se preservan)     │
└──────────────────────────────────────────────────────────────────────────┘
```

---

### Vista: Doctor / Diagnóstico

```
┌─ DOCTOR ─────────────────────────────────────────────────────────────────┐
│                                                                           │
│  Diagnóstico de ./data  ·  iniciado 09:41:03                            │
│                                                                           │
│  Verificando WAL...                                                       │
│    ✓  CRC32C válido en 1,247/1,247 registros        [128ms]              │
│    ✓  Sin registros truncados                                             │
│                                                                           │
│  Verificando índice HNSW...                                               │
│    ✓  45,230 vectores coherentes con datos                [2.1s]          │
│    ✓  Sin nodos huérfanos                                                 │
│    ✓  Recall@10 estimado: 0.998                                           │
│                                                                           │
│  Verificando índice BM25...                                               │
│    ✓  44,891 documentos coherentes con datos               [890ms]        │
│                                                                           │
│  Verificando file lock...                                                  │
│    ✓  Sin otros procesos activos                                          │
│                                                                           │
│  Verificando memoria...                                                   │
│    ✓  RSS: 234 MB (5.7% del límite de 4,096 MB)                         │
│    ⚠  WAL: 128 MB (supera el umbral recomendado de 100 MB)              │
│                                                                           │
│  ──────────────────────────────────────────────────────────────────────  │
│  RESULTADO: 1 advertencia · 0 errores · tiempo: 3.1s                    │
│                                                                           │
│  ¿Ejecutar vacuum para compactar el WAL?  [S] sí   [N] no               │
│                                                                           │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 2.3 — Animaciones e Interactividad

### Principio: animación con propósito

Las animaciones en un TUI de developer tienen una sola razón de existir: **reducir la incertidumbre**. El usuario necesita saber si la herramienta está trabajando o si se colgó. Nada más.

**Animaciones justificadas:**
- Spinner durante operaciones de I/O (< 100ms no necesita spinner)
- Progress bar en operaciones que duran > 500ms
- Cursor parpadeante en campos de texto activos
- Highlight de fila seleccionada al navegar con ↑↓
- Transición de panel activo (cambio de borde color)

**Animaciones que NO deben estar:**
- Efectos de entrada al abrir el TUI
- Fade in/out de paneles
- Typing effect en mensajes
- Animaciones ambient (partículas, degradados en movimiento)

### Spinner durante búsqueda

Para queries que tardan > 100ms, mostrar spinner discreto:

```
  Buscando...  ⠋  [11ms]
  Buscando...  ⠙  [22ms]
  Buscando...  ⠹  [33ms]
  → Resultados (5)         ← Reemplaza el spinner al completar
```

El spinner usa los frames del crate `indicatif`: `⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏`

### Highlight de resultados

Cuando el usuario navega la lista de resultados con ↑↓, el registro activo se muestra con highlight de fondo violeta y el inspector [4] se actualiza instantáneamente (sin latencia adicional — los datos ya están en memoria):

```
    2  legal_gdpr_v2   0.954  "RGPD — Reglamento..."
  ▶ 3  compliance_050  0.891  "Cumplimiento norma..."   ← seleccionado
    4  terms_04        0.847  "Los usuarios tienen..."
```

### Búsqueda en tiempo real

El search bar ejecuta la búsqueda con debounce de 300ms después del último keystroke — el usuario escribe y los resultados aparecen solos:

```
  /  política de p▮              ← usuario escribiendo
     [buscando...]

  /  política de pri▮
     [buscando...]

  /  política de privacidad▮     ← 300ms de silencio → lanza la búsqueda
     → 5 resultados
```

### Confirmación de acciones destructivas

Eliminar un registro o namespace requiere confirmación explícita, con animación de "danger":

```
  ┌─ CONFIRMAR ELIMINACIÓN ──────────────────────┐
  │                                              │
  │  ✗  Esto eliminará permanentemente:          │
  │                                              │
  │     Namespace: docs                          │
  │     Registros: 32,100                        │
  │                                              │
  │  Esta acción no se puede deshacer.           │
  │                                              │
  │  Escribe el nombre del namespace para        │
  │  confirmar: ___________________________      │
  │                                              │
  │  [Enter] confirmar   [Esc] cancelar          │
  └──────────────────────────────────────────────┘
```

---

## 2.4 — Keyboard Navigation

El TUI sigue las convenciones de `vi`/`less`/`k9s` — los developers de sistemas ya tienen este muscle memory:

```
NAVEGACIÓN GLOBAL
  ?          Ayuda / overlay de atajos
  q / Ctrl+C  Salir
  Esc        Volver al panel anterior / cancelar

BÚSQUEDA
  /          Activar search bar y enfocar
  Enter      Ejecutar búsqueda manual
  Esc        Cancelar / volver a resultados

LISTA DE RESULTADOS
  j / ↓      Mover hacia abajo
  k / ↑      Mover hacia arriba
  g          Ir al primer resultado
  G          Ir al último resultado
  Ctrl+D     Bajar media pantalla
  Ctrl+U     Subir media pantalla
  Enter      Inspeccionar registro seleccionado

INSPECTOR [4]
  i          Activar inspector
  e          Editar payload del registro
  d          Eliminar registro (con confirmación)
  y          Copiar key al clipboard
  Y          Copiar payload completo al clipboard
  G          Ver grafo de relaciones

NAMESPACES [1]
  Tab        Cambiar foco a namespaces
  n          Nuevo namespace
  r          Renombrar namespace seleccionado
  D          Eliminar namespace (con confirmación)

VISTAS
  1          Vista principal (búsqueda)
  2          Dashboard de stats
  3          Historial de operaciones recientes
  4          Doctor / diagnóstico
  5          Graph explorer

FILTROS DE BÚSQUEDA (en search bar)
  :mode:hybrid    busqueda-hibrida (default)
  :mode:vector    Solo vectorial
  :mode:text      Solo BM25 (léxica)
  :top:20         Cambiar top-k
  :hop:2          Expandir con graph hops
```

---

---

# PARTE 3: HERRAMIENTAS (Rust Crates)

## 3.1 — CLI: Crates recomendados

### `clap` v4 (derive API) — Parser de argumentos
**Ya probablemente en uso. Confirmar que usa derive API, no builder API.**

```toml
[dependencies]
clap = { version = "4", features = ["derive", "env", "color", "suggestions"] }
clap_complete = "4"     # Generación de shell completions
```

```rust
// Ejemplo con features críticas habilitadas
#[derive(Parser)]
#[command(
    name = "vantadb",
    about = "Motor de persistencia cognitiva para agentes de IA",
    long_about = None,
    version,
    after_help = "Ejemplos:\n  vantadb search 'privacidad' --namespace docs\n  vantadb put mi_key 'texto' --vector [0.1,0.2,...]"
)]
struct Cli {
    #[arg(long, default_value = "./vantadb", env = "VANTADB_PATH")]
    db: PathBuf,
    
    #[arg(long, default_value = "default")]
    namespace: String,
    
    #[arg(long, default_value = "human", value_enum)]
    format: OutputFormat,
    
    #[arg(long, global = true)]
    quiet: bool,
    
    #[arg(long, global = true)]
    timing: bool,
}
```

La feature `suggestions` habilita automáticamente el "¿Quisiste decir?" para typos.

---

### `indicatif` — Progress bars y spinners

```toml
indicatif = { version = "0.17", features = ["rayon"] }
```

```rust
// Progress bar para import
let pb = ProgressBar::new(total_records as u64);
pb.set_style(ProgressStyle::with_template(
    "  Indexando vectores  {bar:52.violet/gray}  {pos}/{len}  {eta}"
)?
.progress_chars("█░"));

// Spinner para operaciones sin duración conocida
let spinner = ProgressBar::new_spinner();
spinner.set_style(ProgressStyle::with_template("  {spinner:.violet}  {msg}")?);
spinner.set_message("Conectando...");
```

---

### `console` — Colores y estilos

```toml
console = "0.15"
```

```rust
use console::{style, Emoji};

// Iconos con fallback si la terminal no soporta emoji
static SUCCESS: Emoji<'_, '_> = Emoji("✓  ", "OK ");
static ERROR: Emoji<'_, '_>   = Emoji("✗  ", "!! ");
static WARN: Emoji<'_, '_>    = Emoji("⚠  ", "?? ");

// Colores semánticos consistentes
println!("{} Operación completada", style(SUCCESS).green().bold());
println!("{} {}", style(ERROR).red().bold(), style("Error: archivo no encontrado").red());
println!("{} WAL supera 100MB", style(WARN).yellow());

// Detecta si stdout es terminal o pipe
if !console::Term::stdout().is_term() {
    // Deshabilitar colores automáticamente en pipes
}
```

---

### `comfy-table` — Tablas en terminal

```toml
comfy-table = { version = "7", features = ["tty"] }
```

```rust
use comfy_table::{Table, Cell, Color, Attribute};

let mut table = Table::new();
table
    .load_preset(comfy_table::presets::UTF8_BORDERS_ONLY)
    .set_header(vec![
        Cell::new("#").add_attribute(Attribute::Bold),
        Cell::new("Clave").add_attribute(Attribute::Bold),
        Cell::new("Score").add_attribute(Attribute::Bold),
        Cell::new("Extracto").add_attribute(Attribute::Bold),
    ]);

for (i, result) in results.iter().enumerate() {
    table.add_row(vec![
        Cell::new(i + 1).fg(Color::DarkGrey),
        Cell::new(&result.key).fg(Color::Cyan),
        Cell::new(format!("{:.3}", result.score)).fg(Color::Green),
        Cell::new(truncate(&result.excerpt, 60)).fg(Color::White),
    ]);
}
println!("{table}");
```

---

### `dialoguer` — Prompts interactivos

```toml
dialoguer = { version = "0.11", features = ["history"] }
```

```rust
use dialoguer::{Confirm, Input, Select};

// Confirmación de borrado
if Confirm::new()
    .with_prompt("¿Eliminar namespace 'docs' con 32,100 registros?")
    .default(false)
    .interact()?
{
    db.delete_namespace("docs")?;
}

// Selección de modo de búsqueda
let mode = Select::new()
    .with_prompt("Modo de búsqueda")
    .items(&["Híbrido (recomendado)", "Solo vectorial", "Solo BM25"])
    .default(0)
    .interact()?;
```

---

### `color-eyre` — Mensajes de error ricos

```toml
color-eyre = "0.6"
```

```rust
use color_eyre::{eyre::eyre, Result};

fn open_database(path: &Path) -> Result<VantaDB> {
    VantaDB::open(path).map_err(|e| {
        eyre!(e)
            .wrap_err(format!("No se puede abrir la base de datos en {:?}", path))
            .suggestion("Si es la primera vez, ejecuta: vantadb init")
            .suggestion("Si migraste los archivos, usa --db <nuevo-path>")
    })
}
```

---

### `clap_complete` — Shell completions

```rust
use clap_complete::{generate, Shell};

// Subcomando: vantadb completions <shell>
fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "vantadb", &mut io::stdout());
}
```

---

### `human-panic` — Panics legibles

```toml
human-panic = "2"
```

```rust
fn main() {
    human_panic::setup_panic!(Metadata::new(
        "VantaDB CLI",
        env!("CARGO_PKG_VERSION")
    ));
    // ...
}
```

En vez de un stack trace crudo, el usuario ve:
```
Oops! VantaDB tuvo un error inesperado.

Por favor reporta este error en:
https://github.com/ness-e/vantadb/issues

Con el contenido del archivo: /tmp/vantadb-panic-report-xxxxx.txt
```

---

## 3.2 — TUI: Crates recomendados

### `ratatui` + `crossterm` — Framework TUI

```toml
ratatui    = "0.27"
crossterm  = { version = "0.28", features = ["event-stream"] }
```

`ratatui` es el estándar actual en Rust para TUIs (fork activo de tui-rs con 9K+ stars). `crossterm` proporciona el backend cross-platform (Windows, macOS, Linux).

```rust
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Gauge},
};

// Estructura de estado de la aplicación
struct App {
    search_query: String,
    results: Vec<SearchResult>,
    selected_idx: usize,
    active_pane: Pane,
    namespaces: Vec<NamespaceInfo>,
}

// Loop principal del TUI
fn run_tui(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) {
    loop {
        terminal.draw(|f| render(f, app))?;
        
        if let Event::Key(key) = event::read()? {
            match (key.code, key.modifiers) {
                (KeyCode::Char('q'), _) => break,
                (KeyCode::Char('/'), _) => app.focus_search(),
                (KeyCode::Down | KeyCode::Char('j'), _) => app.next_result(),
                (KeyCode::Up | KeyCode::Char('k'), _) => app.prev_result(),
                (KeyCode::Char('g'), KeyModifiers::NONE) => app.go_to_graph(),
                _ => {}
            }
        }
    }
}
```

---

### `tui-textarea` — Campo de búsqueda

```toml
tui-textarea = "0.6"
```

Proporciona un text input widget completo para ratatui con soporte de cursor, historial, y shortcuts de edición tipo readline (`Ctrl+A`, `Ctrl+E`, `Ctrl+W`):

```rust
use tui_textarea::TextArea;

let mut search = TextArea::default();
search.set_placeholder_text("/ buscar en la base de datos...");
search.set_cursor_line_style(Style::default().add_modifier(Modifier::UNDERLINED));

// Render
f.render_widget(&search, search_area);
```

---

### `ratatui` gauge y sparklines (incluido en ratatui)

Para el dashboard de stats, sparklines (mini gráficas) y gauges (barras de progreso):

```rust
// Gauge de memoria RAM
let gauge = Gauge::default()
    .block(Block::default().title("RAM"))
    .gauge_style(Style::default().fg(Color::Cyan))
    .ratio(ram_usage_ratio);

// Sparkline de latencia últimas 24h
let sparkline = Sparkline::default()
    .block(Block::default().title("Latencia (24h)"))
    .data(&latency_history)
    .style(Style::default().fg(Color::Cyan));
```

---

### `tui-scrollview` — Listas scrollables

```toml
tui-scrollview = "0.4"
```

Para el inspector de registros largos y listas de resultados extensas:

```rust
use tui_scrollview::ScrollView;

let mut scroll_view = ScrollView::new(Size::new(area.width, 100));
// Render del contenido del inspector
render_inspector_content(scroll_view.buf_mut(), app);
// Display con scroll
scroll_view.render(area, buf, &mut app.scroll_state);
```

---

### `unicode-width` — Alineación correcta de caracteres

```toml
unicode-width = "0.1"
```

Para tablas y truncado de texto con caracteres unicode (español, emojis en metadata):

```rust
use unicode_width::UnicodeWidthStr;

fn truncate_to_width(s: &str, max_width: usize) -> String {
    if s.width() <= max_width {
        return s.to_string();
    }
    let mut width = 0;
    let mut result = String::new();
    for ch in s.chars() {
        let ch_width = ch.width().unwrap_or(1);
        if width + ch_width + 1 > max_width { // +1 para "…"
            result.push('…');
            break;
        }
        result.push(ch);
        width += ch_width;
    }
    result
}
```

---

## 3.3 — Resumen de crates por función

| Función | Crate | Versión | Notas |
|---------|-------|---------|-------|
| CLI parsing | `clap` | 4 | Features: derive, suggestions, env |
| Shell completions | `clap_complete` | 4 | Bash, Zsh, Fish, PowerShell |
| Progress bars | `indicatif` | 0.17 | Feature: rayon para paralelismo |
| Colores y estilos | `console` | 0.15 | Detecta tty automáticamente |
| Tablas | `comfy-table` | 7 | Feature: tty para auto-width |
| Prompts interactivos | `dialoguer` | 0.11 | Feature: history |
| Manejo de errores | `color-eyre` | 0.6 | Con suggestions |
| Panics legibles | `human-panic` | 2 | Solo para binary targets |
| TUI framework | `ratatui` | 0.27 | Backbone del TUI |
| Terminal backend | `crossterm` | 0.28 | Cross-platform |
| Text input TUI | `tui-textarea` | 0.6 | Search bar |
| Scroll containers | `tui-scrollview` | 0.4 | Inspector y listas largas |
| Ancho unicode | `unicode-width` | 0.1 | Truncado correcto de strings |
| Tamaños en bytes | `bytesize` | 1 | "598 MB" en vez de "627145728" |
| Formato de fechas | `chrono` | 0.4 | "2026-06-13 09:41:03" |
| JSON output | `serde_json` | 1 | Con `--format json` |

---

---

# PARTE 4: PRIORIDAD DE IMPLEMENTACIÓN

## Fase 3 (pre-lanzamiento) — Lo que bloquea la credibilidad

```
CLI-P1 | Flags globales --format json y --quiet en TODOS los comandos
CLI-P2 | Progress bar en import (indicatif)
CLI-P3 | Mensajes de error accionables con color-eyre
CLI-P4 | Sugerencias de typos en comandos (clap feature: suggestions)
CLI-P5 | Shell completions (clap_complete)
```

## Fase 4 (lanzamiento) — Lo que mejora la DX

```
CLI-P6  | vantadb doctor (output diseñado como en mockup)
CLI-P7  | vantadb stats (con tablas comfy-table)
CLI-P8  | vantadb inspect (inspector de registro completo)
CLI-P9  | vantadb backup con progress bar y verificación CRC32C
CLI-P10 | Confirmaciones interactivas para borrados (dialoguer)
TUI-P1  | vantadb repl — versión básica: búsqueda + resultados + inspector
TUI-P2  | Real-time search con debounce 300ms (tui-textarea)
TUI-P3  | Dashboard de stats (ratatui gauges + sparklines)
```

## Fase 5 (post-lanzamiento) — Lo que diferencia

```
TUI-P4 | Graph explorer con ASCII (box-drawing characters)
TUI-P5 | Import/Export con progress modal en TUI
TUI-P6 | Mouse support en el TUI
TUI-P7 | Themes configurables (sin color por default para terminales básicas)
```

---

## La regla de oro del diseño para VantaDB

> **Un developer que abre `vantadb repl` por primera vez tiene que entender la herramienta en 10 segundos, sin leer documentación.**

Si el layout del TUI requiere explicación, el layout está mal. Si un mensaje de error requiere buscar en la documentación, el mensaje está mal. Si una progress bar no muestra ETA, no sirve de nada.

El CLI/TUI no es el producto. Es la ventana por la que el developer ve el producto. La ventana tiene que ser clara, no decorativa.

---

*Especificación completada: 2026-06-13 | Basado en análisis de herramientas equivalentes: k9s, lazygit, cargo, btm, broot*
