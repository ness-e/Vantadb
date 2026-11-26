---
title: "Stealth Customer Discovery & The Mom Test Playbook"
kind: concept
status: active
description: Protocolo encubierto de validación de hipótesis de dolor de memoria para agentes AI en comunidades técnicas basado en The Mom Test
tags: [vantadb, gtm, customer-discovery, mom-test, validation, reddit, sales]
---

# Stealth Customer Discovery & The Mom Test Playbook: VantaDB

Este documento formaliza el protocolo encubierto de validación de mercado y descubrimiento de clientes (*Stealth Customer Discovery*) para **VantaDB**, aplicando rigurosamente los principios de **"The Mom Test"** (Rob Fitzpatrick).

> **Objetivo Primario:** Confirmar si el dolor de la amnesia de los agentes de IA, la saturación de archivos de contexto (`CLAUDE.md` / `AGENTS.md`), el impuesto de tokens de MCP y la fricción de Docker representan una necesidad urgente y monetizable, **sin mencionar VantaDB, sin vender nada y sin pedir validación emocional artificial.**

---

## 1. Principios Operativos de The Mom Test Aplicados

1. **Habla de sus vidas y flujos reales, no de tu idea:** Prohibido decir *"Estoy construyendo una base de datos de memoria para agentes llamada VantaDB, ¿qué opinas?"*. La respuesta siempre será condescendiente y falsa.
2. **Pregunta por hechos pasados y comportamientos específicos:** *"¿Cómo resolviste la persistencia de contexto la semana pasada?"* o *"¿Cuánto tiempo te tomó configurar tu último servidor MCP?"*, en lugar de *"¿Te gustaría tener memoria persistente?"*.
3. **Mide el dolor en tiempo y dinero ya invertido:** Si el usuario no ha gastado dinero, instalado extensiones, escrito scripts en Python o sufrido caídas de sesión, el dolor no existe o es secundario.
4. **Detecta el "Complimento Asesino":** Si alguien dice *"¡Qué gran idea, avísame cuando esté lista!"*, eso es un rechazo amable. Solo cuenta si piden el código de inmediato, comparten un repositorio roto donde sufrieron el fallo o aceptan una llamada de diagnóstico técnico.

---

## 2. Matriz de las 5 Hipótesis Críticas (H1 – H5)

| ID | Hipótesis Falsable | Señal de Dolor Real (Validación) | Señal de Falsa Alarma (Invalidación) | Métrica / Umbral de Éxito |
| :--- | :--- | :--- | :--- | :--- |
| **H1** | **Inflación de `CLAUDE.md` / `AGENTS.md`:** Los desarrolladores que usan Claude Code o Cursor acumulan archivos de reglas >300-500 líneas, quemando 5k-20k tokens por sesión y degradando el razonamiento del modelo ("amnesia y confusión"). | Desarrolladores admiten podar manualmente archivos cada semana, dividir reglas en múltiples archivos o quejarse de límites de contexto agotados en pocos turnos. | "Con 50 líneas de `CLAUDE.md` me basta", o "No noto degradación ni costo relevante en tokens". | ≥ 60% de comentarios en Post 1 confirman fatiga de mantenimiento o degradación de contexto. |
| **H2** | **Impuesto de Contexto en MCP (Tool Tax):** Los servidores MCP tradicionales inyectan 40-80 esquemas de herramientas JSON-RPC, consumiendo entre el 20% y el 40% del contexto antes de que el usuario escriba su primer prompt. | Usuarios desconectan servidores MCP en Cursor por superar el límite de 40 herramientas o comentan flags en `.mcp.json` para no agotar la ventana. | "El contexto de Claude 3.5 / 3.7 es de 200k, 10k tokens de herramientas no me molestan". | ≥ 50% de los usuarios de Cursor/Claude Code declaran auditar o apagar MCPs por gasto de contexto. |
| **H3** | **Locks y `DatabaseBusy` en Concurrencia Local:** Al abrir dos instancias de Cursor, Claude Code en terminal y un agente en background contra SQLite/DuckDB local, el sistema colapsa por locks de archivo. | Usuarios reportan errores de bloqueo, reinicios forzados de IDE o abandono de soluciones locales por fallos de concurrencia. | "Solo corro un agente a la vez en una sola ventana de terminal". | ≥ 30% de desarrolladores multi-agente declaran haber sufrido locks de base de datos en local. |
| **H4** | **Fricción Operativa de Docker:** Desarrolladores individuales y equipos pequeños rechazan frameworks como Mem0, Letta (MemGPT) o Zep porque exigen Docker Compose, Postgres+pgvector y consumen >2-4GB de RAM en su laptop. | Expresan frustración por tener que levantar contenedores para un simple asistente de código local y buscan un binario embebido estilo SQLite. | "Tengo Docker siempre corriendo con 64GB de RAM, prefiero levantar Postgres con un comando". | ≥ 65% de desarrolladores prefieren un binario único embebido sobre un stack Dockerizado. |
| **H5** | **Disposición a Pagar (Consultoría/Soporte $750–$1,000/mes):** Líderes técnicos de equipos medianos (5–25 ingenieros) están dispuestos a pagar consultoría mensual o soporte prioritario para unificar la memoria de su equipo y evitar re-trabajo entre PRs. | Líderes técnicos solicitan diagnóstico en DM, admiten que sus devs gastan horas re-explicando decisiones de arquitectura y aceptan una llamada de 20 min. | Ningún contacto por DM pide ayuda formal; solo buscan software gratuito sin soporte ni SLA. | ≥ 2 conversaciones en DM convertidas a propuestas de diagnóstico o PoC empresarial en 30 días. |

---

## 3. Las 4 Plantillas de Posts Encubiertos para Reddit

> **Norma Estricta:** Estos posts se publican desde una cuenta personal activa con karma técnico genuino. No enlaces a GitHub, no nombres a VantaDB, no incluyas llamadas a la acción comerciales.

### Post 1: r/ClaudeAI & r/Cursor — Dolor H1 (Context Bloat & Amnesia)

* **Título:** How are you guys keeping Claude Code / Cursor from re-learning your entire codebase architecture every session without bloating CLAUDE.md past 500 lines?
* **Subreddits sugeridos:** `r/ClaudeAI`, `r/Cursor`, `r/OpenCode`
* **Cuerpo del Post:**

```markdown
I've been heavily using Claude Code and Cursor for multi-day refactors, and I've hit a wall with project context retention.

Right now, my `CLAUDE.md` (and `.cursorrules`) has grown into an unruly 650-line monster. It contains our architectural boundaries, key decisions, API conventions, and database schema caveats.

The problem:
1. Every new terminal session or chat burns ~8k-15k tokens just loading instructions before I even write a prompt.
2. Worse, after ~10-15 conversation turns, the model starts suffering from "lost-in-the-middle" and ignores rules explicitly defined in line 400.
3. If I strip it down to 50 lines, it immediately hallucinates old deprecated patterns we abandoned two weeks ago.

How are teams actually handling long-term context retention across sessions? 
- Are you using sub-rules that get loaded conditionally?
- Have you hooked up a local memory MCP server (and if so, does it actually recall cleanly)?
- Or do you just accept re-explaining context every morning?

Curious to hear what patterns have worked in practice vs what looks good on paper.
```

---

### Post 2: r/Cursor & r/mcp — Dolor H2 (MCP Tool Tax & Token Limits)

* **Título:** Has anyone actually measured the token overhead of MCP server schemas? My context window is 20% full before typing prompt #1.
* **Subreddits sugeridos:** `r/Cursor`, `r/mcp`, `r/ClaudeAI`
* **Cuerpo del Post:**

```markdown
Ran a quick inspection on what gets sent over the wire when initializing my dev workspace with 3 MCP servers enabled (GitHub, a DB inspector, and a memory server).

The tool definitions and JSON schemas alone took up almost 22,000 tokens of system context. 

In Cursor, I've also run into the ~40 tool limit where adding one more server causes tools to randomly disappear or fail silent execution. On top of that, each tool call payload returns verbose JSON that blows past context budgets within 4-5 multi-step tool iterations.

For those running MCP in daily coding workflows:
- Are you pruning tool manifests manually?
- Do you run different MCP configs per project, or disable them dynamically?
- Is there any standard or proxy tool that exposes a minimal cognitive surface (e.g. 5-10 core actions) while delegating the rest under the hood?

Would love to know if people have benchmarked this or if there's a lean pattern I'm missing.
```

---

### Post 3: r/LocalLLaMA & r/AI_Agents — Dolor H4 & H1 (Docker Fatigue)

* **Título:** Why does every agent memory framework require Docker Compose, PostgreSQL, and 3GB of RAM? Am I crazy for wanting a zero-dependency embedded binary?
* **Subreddits sugeridos:** `r/LocalLLaMA`, `r/AI_Agents`, `r/LangChain`
* **Cuerpo del Post:**

```markdown
I've spent the last weekend evaluating memory backends for local agent workflows (Mem0, Letta/MemGPT, Zep, etc.).

All of them have interesting cognitive architectures, but the operational footprint feels insane for local-first developer tooling:
- You need Docker running constantly in the background.
- A dedicated PostgreSQL instance with pgvector, plus Redis for session queuing.
- Python runtime overhead eating 2GB to 4GB of RAM just sitting idle waiting for a hook.
- When working offline or on a laptop on battery, it's completely impractical.

In the relational world, we have SQLite: a single binary or embedded library, file-backed, zero daemon, instant startup.

Why haven't we seen a true "SQLite for Agent Memory" that runs embedded (pure Rust/C++ binary), stores vectors + BM25 text on disk with a simple WAL, runs local embeddings (like ONNX or small local models), and exposes stdio MCP without needing a single container?

Is there a technical bottleneck I'm overlooking with embedded vector graphs and persistence, or is the AI tooling ecosystem just addicted to microservices?
```

---

### Post 4: r/ClaudeAI & r/Cursor — Dolor H3 & H5 (Multi-Instance Concurrency)

* **Título:** Multi-agent concurrency on local disk: how do you prevent file locks and 'DatabaseBusy' crashes when running multiple IDE windows?
* **Subreddits sugeridos:** `r/ClaudeAI`, `r/Cursor`, `r/rust`
* **Cuerpo del Post:**

```markdown
Question for developers running agentic setups across multiple workspaces:

I frequently have 2 Cursor windows open for different microservices, plus Claude Code running in a background terminal doing test generation. Both need access to shared architectural memory and past bug resolutions.

Whenever I tried using local SQLite or embedded vector stores for this, things blew up immediately:
- Database locks (`database is locked` / `busy`) as soon as both sessions attempt to commit memories or update embeddings simultaneously.
- Corrupted WAL indices when an agent crashes mid-transaction.
- Stale memory reads where Window B doesn't see what Window A just stored without restarting the server.

The only "working" workaround I found was standing up an external cloud API or a heavy centralized server, which defeats the point of local privacy and offline work.

How are people solving concurrent multi-instance agent writes on local disk? Is anyone using a local reader-writer multiplexer or WAL proxy that actually works reliably?
```

---

## 4. Calendario de Publicación y Distribución

Para evitar sospechas y recolectar datos limpios sin sesgo de spam:

| Día | Post | Subreddit Principal | Subreddit Secundario | Objetivo |
| :--- | :--- | :--- | :--- | :--- |
| **Día 1** | **Post 1** (Context Bloat) | `r/ClaudeAI` | `r/Cursor` | Validar dolor H1 en usuarios de IDEs comerciales. |
| **Día 4** | **Post 3** (Docker Fatigue) | `r/LocalLLaMA` | `r/AI_Agents` | Validar H4 en comunidad hacker y defensora de lo local. |
| **Día 7** | **Post 2** (MCP Tool Tax) | `r/Cursor` | `r/mcp` | Validar H2 y el apetito por perfiles compactos (`brain` profile). |
| **Día 10** | **Post 4** (Multi-Instance) | `r/ClaudeAI` | `r/rust` | Validar H3 y detectar líderes técnicos con flujos complejos (H5). |

---

## 5. Protocolo de Preguntas de Seguimiento (The Mom Test)

Cuando los usuarios respondan en el hilo, **jamás expliques la solución**. Usa estas respuestas tipo para profundizar en el dolor real:

### Banco de Respuestas en Hilo Público

1. **Cuando alguien dice "Yo uso un script propio en Python":**
   * *Pregunta:* "¿Cuánto tiempo te tomó dejarlo funcionando de forma estable? ¿Qué pasa cuando dos procesos intentan escribir al mismo tiempo?"
2. **Cuando alguien dice "Yo uso Mem0 / Letta":**
   * *Pregunta:* "¿Qué tal el consumo de recursos en tu máquina del día a día? ¿Has tenido problemas con extracción de datos basura o latencia al buscar en proyectos grandes?"
3. **Cuando alguien dice "Solo uso `CLAUDE.md` corto y le paso docs a mano":**
   * *Pregunta:* "¿Con qué frecuencia el agente olvida decisiones que tomaste hace dos días? ¿Cuánto tiempo calculas que pasas re-escribiendo ese contexto cada semana?"
4. **Cuando alguien se queja amargamente de tokens o límites de Cursor:**
   * *Pregunta:* "¿Has calculado cuántos dólares o porcentaje de tu cuota mensual se van en re-alimentar contexto repetitivo? ¿Estarías dispuesto a eliminar los archivos de reglas si la recuperación fuera automática?"

---

## 6. Embudo de Conversión por DM (Venta de Consultoría $750–$1,000/mes)

Cuando un usuario demuestre un dolor extremo (ej. líder de equipo, CTO de startup o desarrollador senior con múltiples repositorios), inicia el contacto por Mensaje Directo (DM).

### Paso 1: Mensaje de Diagnóstico Empático (Día 0)

> **Asunto / Primer mensaje:**
> *"Hola [Nombre]. Vi tu comentario en r/[Subreddit] sobre cómo sufres con [la pérdida de contexto de Claude Code / los locks de base de datos en múltiples ventanas]. Nosotros estuvimos lidiando exactamente con ese mismo infierno técnico en varios repositorios grandes.*
> 
> *¿Pudiste encontrar alguna forma limpia de resolverlo o sigues teniendo que reiniciar el entorno a mano?"*

### Paso 2: Calificación de Necesidad y Esfuerzo (Día 1–2)

Si el usuario responde explicando su problema a fondo:
> *"Totalmente de acuerdo, ese problema de re-explicar arquitectura entre PRs le cuesta a cualquier equipo de 5 ingenieros al menos 10-15 horas semanales de re-trabajo y desalineación de código.*
> 
> *Actualmente estoy asesorando a un par de equipos técnicos para montar una arquitectura de memoria persistente local y segura (cero fugas a la nube, sin Docker pesado, con sincronización de equipo encriptada). Si te sirve, podemos hacer una llamada técnica de 20 minutos por Discord/Google Meet esta semana, revisamos tu setup actual y te muestro exactamente cómo eliminamos ese cuello de botella sin costo ni compromiso."*

### Paso 3: Propuesta Comercial en Llamada ($750 – $1,000/mes)

Durante la llamada de 20 minutos:
1. **Minutos 0–10:** Escuchar su flujo actual, diagramar dónde pierden tiempo y calcular el costo en horas/dólares del equipo.
2. **Minutos 10–15:** Demostrar cómo una arquitectura local-first con proxy concurrente y sincronización E2EE resuelve el problema de raíz en su IDE.
3. **Minutos 15–20:** Presentar la oferta:
   * **Tier de Soporte / Integración Dedicada:** **$750/mes** (equipos pequeños, 1-5 devs) o **$1,000/mes** (equipos en crecimiento, 6-15 devs).
   * **Entregables:**
     - Configuración e integración a medida del sistema de memoria en los IDEs del equipo (Claude Code, Cursor, Windsurf).
     - Canal privado en Discord/Slack con SLA de respuesta <24h.
     - Pipeline personalizado de embeddings locales (EmbeddingGemma 2 / E5 ONNX) según su stack.
     - Garantía de cero caídas por locks de base de datos y auditoría de retención de contexto.
   * **Riel de Pago:** Facturación directa vía Ko-fi, Patreon o transferencia USDT (Binance Pay) con factura comercial simplificada.

---

## 7. Semáforo de Interpretación y Ajustes en el Producto

Tras recopilar entre **30 y 50 respuestas acumuladas** en los 4 posts:

| Color | Señales Observadas | Diagnóstico | Acción Inmediata en VantaDB |
| :---: | :--- | :--- | :--- |
| 🟢 **VERDE** | • >50% se quejan del peso de `CLAUDE.md` o del impuesto de tokens en MCP.<br>• Múltiples solicitudes de *"¿Hay algún repo que solucione esto?"*<br>• Al menos 3 DMs de líderes técnicos o devs seniors interesados en probar una solución. | **Validación Plena de Problema-Solución.** El dolor es agudo, urgente y el mercado busca activamente una alternativa embebida. | 1. Proceder al lanzamiento público formal (Semana 8).<br>2. Poner en el hero del `README`: *"Stop stuffing 500 lines into CLAUDE.md. Persistent memory for AI IDEs in a single 0-Docker Rust binary."*<br>3. Publicar el perfil `brain` de MCP como el default para Cursor. |
| 🟡 **AMARILLO** | • La gente reconoce el problema pero usa soluciones parche ("yo solo hago git commit del resumen en un archivo txt").<br>• El dolor existe pero la disposición a cambiar de herramienta es tibia.<br>• Pocas menciones a Docker como problema mayor. | **Problema Real pero Solución No Urgente.** El usuario está acostumbrado al dolor y tolera la ineficiencia. | 1. Modificar el copy del `README`: centrarse en la **velocidad de setup en 30 segundos** y el **ahorro medible de dólares en tokens**.<br>2. Reforzar el script `vhs/demo.tape` para mostrar la comparación visual lado a lado: con VantaDB vs sin VantaDB.<br>3. Ajustar el soporte comercial a una PoC inicial más acotada. |
| 🔴 **ROJO** | • Comentarios mayoritarios del tipo: *"Con 100k de contexto no me importa quemar tokens"* o *"Docker no me molesta en absoluto"*. | **Falsa Hipótesis o Mercado Inmaduro.** El dolor de la memoria no es el cuello de botella que sienten los desarrolladores en este momento. | 1. Frenar la inversión en la capa Enterprise / Team Sync.<br>2. Reposicionar VantaDB puramente como motor de búsqueda vectorial híbrido de ultra-alta velocidad (BM25 + HNSW embebido en Rust para Python/Node), destacando los benchmarks de SIFT-1M y recall en <2ms. |
