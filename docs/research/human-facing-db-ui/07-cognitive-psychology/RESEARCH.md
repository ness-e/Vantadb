# Psicología cognitiva aplicada a la representación de VantaDB

Evaluación del concepto `06-synthesis/SYNTHESIS.md` contra cómo funciona el cerebro humano,
cómo el humano trabaja en la oficina/sociedad y cómo el humano programa y visualiza datos.
Documento de corrección: identifica 5 debilidades cognitivas y propone los fixes.

> Fuente del análisis: principios establecidos de percepción visual (Ware), percepción de
> gráficos (Cleveland & McGill), carga cognitiva (Sweller), mantenimiento del mantra de
> Shneiderman, recuperación de errores (Norman), atención dividida (Mayer), y forrajeo de
> información (Pirolli & Card). URLs y referencias al final.

---

## 1. Verdict

El concepto de la síntesis es **sólido y correcto en lo esencial**: las decisiones fuertes
(registros como núcleo, master-detail, commit explícito, explicabilidad, progressive
disclosure, grafos con navegación, local-first) son cognitivamente bien fundamentadas.
No es **100 % óptimo**: tiene 5 debilidades reales desde la perspectiva del cerebro humano.
Este documento las corrige.

---

## 2. Lo que ya es óptimo (y por qué)

| Decisión de la síntesis | Principio cognitivo que la respalda |
|-------------------------|--------------------------------------|
| Master-detail (grid → inspector) | **Details-on-demand** del mantra de Shneiderman: el ojo explora primero, el detalle llega después |
| Progressive disclosure (vector colapsado, JSON como lujo) | **Carga extrínseca** (Sweller): no exponer información irrelevante a la tarea |
| Commit explícito con diff | **Feedback + recuperación de errores** (Norman): el usuario ve el estado y puede revertir |
| Color por tipo `VantaValue` + badges TTL | **Atributos preatentivos** (Ware/Treisman): forma, color y posición se procesan en <200 ms |
| Explicabilidad "por qué recuperó esto" | **Razonamiento causal humano**: el cerebro entiende causas, no scores crudos |
| Registros como núcleo (no índices) | **Modelo mental del usuario**: respeta cómo el usuario ya piensa la memoria |
| Local-first sin login | Menos fricción, más **confianza y propiedad** (contexto social de uso) |
| Mapa vectorial opcional con filtros | Evita el anti-patrón documentado: proyecciones sin contexto confunden (dato en 01) |

---

## 3. Las 5 debilidades y sus fixes

### Debilidad 1 — Falta el Overview (primer paso del mantra de Shneiderman)

**Problema.** El home es directamente la tabla MEMORIAS. Shneiderman (1996) ordena la
interacción como *overview → zoom/filter → details-on-demand*: el cerebro hace **scoping
antes que probing** — necesita saber primero *cuánta* memoria hay, de qué tipos, qué está
por expirar, qué creció. Sin ese índice global, un namespace grande se vive como entrar a
un archivo sin índice.

**Fix.** Vista HOME/RESUMEN como pantalla de entrada:
- Conteo por namespace + tendencia (crecimiento de memoria en el tiempo).
- Distribución de tipos de metadata (mini-histograma).
- Próximos a expirar (TTL) y expirados recientes.
- Actividad reciente (últimas escrituras/borrados del audit log).
- 6-8 cards/sparklines como máximo: **overview, no dashboard-bloat** (Few: cada número
  debe poder defenderse; lo demás es decoración).

### Debilidad 2 — 5 tabs fragmentan el modelo mental (split-attention)

**Problema.** Cinco pestañas (Memorias, Retrieval, Grafo, Espacio, Operaciones) son cinco
metáforas separadas. El cerebro es malo **conmutando entre tareas** e **integrando
información distribuida** (efecto split-attention de Mayer: aprender/entender cuesta menos
cuando lo relacionado está co-ubicado). El propio reporte 04 recomendaba "3 vistas
complementarias, no una" — no 5 destinos. Además, cada conmutación cobra un impuesto
(cost of switching) que se multiplica con el uso frecuente.

**Fix.** Workspace unificado tipo IDE (VS Code mental model, natural para el desarrollador):
- **Centro**: la tabla de registros es la superficie permanente.
- **Inspector derecho contextual**: al seleccionar un registro, muestra capas del mismo
  objeto (General / Metadata / Vector / Historial+Diff / Conexiones).
- **Modos contextuales**: Grafo, Espacio y Retrieval se comportan como *lentes sobre el
  contexto actual* (el namespace o registro seleccionado), accesibles sin perder la posición.
- **Operaciones** (métricas, import/export, audit) como vista secundaria, no como pestaña
  hermana de igual rango.
- Resultado: menos navegación, más contexto, un solo modelo mental.

### Debilidad 3 — Falta la dimensión temporal y el diff entre versiones

**Problema.** El cerebro humano razona en **historias y cambios** ("qué cambió", "qué pasó
ayer"). El audit log existe pero como lista plana. Falta (a) un **timeline unificado** de la
vida de la memoria y (b) el **diff entre versiones** (n vs n-1). El desarrollador entiende un
diff al instante (Git mental model): es la forma más natural de "editar sin romper".

**Fix.**
- En el inspector, tab **Historial** pasa a ser **Historial + Diff**: cada versión con su
  cambio resaltado (payload/metadata/vector) estilo git.
- Vista **Timeline** (por namespace): línea de tiempo de escrituras, actualizaciones,
  expiraciones y borrados (el audit log JSONL ya da los eventos; falta agruparlos en el
  tiempo).
- Complemento de la consolidación asistida: marcar duplicados/superados con su diff visible.

### Debilidad 4 — Recuperación de errores incompleta: undo, papelera, command palette

**Problema.** Commit explícito sin undo genera **ansiedad ante operaciones destructivas**
(Norman: el sistema debe permitir recuperarse de errores). Y el usuario objetivo — un
desarrollador de IA — piensa en **comandos y teclado**, no en menús (forrajeo de
información, Pirolli & Card: el humano minimiza el costo de buscar una acción).

**Fix.**
- **Undo global** (Ctrl+Z) sobre las operaciones de la sesión (put/delete/edit/batch).
- **Papelera / soft-delete**: "eliminar" mueve a un estado recuperable (VantaDB ya modela
  tombstones; la UI puede exponer un restore), en vez de destrucción inmediata.
- **Command palette (Ctrl+K)** con acciones tipadas (abrir namespace, buscar key, ejecutar
  IQL, exportar, borrar) — el patrón más querido de VS Code/Linear; cada operación se
  reduce a segundos y el descubrimiento es incremental.
- Teclado-first: atajos para las operaciones frecuentes (editar, guardar, buscar, delete).

### Debilidad 5 — Codificación visual de scores (Cleveland & McGill)

**Problema.** Mostrar el desglose BM25/HNSW/RRF como números crudos es ineficiente. Cleveland
& McGill (1984) y Mackinlay (1986) ordenan la precisión de los canales visuales:
**posición > longitud > ángulo > área > color > forma**. El cerebro compara por longitud y
posición antes que por dígitos o colores.

**Fix.**
- Desglose de score como **barras horizontales apiladas** (longitud = score) con etiqueta
  numérica opcional; color solo como canal secundario de agrupación.
- TTL: **barra/ring de countdown** (longitud/arco = tiempo restante), no solo badge.
- Comparación entre resultados: usar el mismo eje de longitud para que la diferencia se
  lea preatentivamente.

---

## 4. Correcciones menores (aplicar sin re-planear)

1. **Accesibilidad (color + redundancia).** Los chips por tipo `VantaValue` y los badges
   TTL no deben depender solo del color (~8 % de hombres con daltonismo). Encoding
   redundante: color + ícono/forma + texto (Okabe-Ito friendly palette). El reporte 05 ya
   lo menciona; la síntesis no lo explicita.
2. **Humano en la oficina/sociedad.** Falta anotación (notas/comentarios por registro) y
   exportar "vista legible" (reporte markdown/HTML del estado de la memoria), además de
   JSON/CSV. Deep links ya existen (bien).
3. **Metáfora única y coherente.** "Observatorio" (pasivo) + "atelier" (activo) + "Studio"
   mezclan modelos. Para desarrolladores la metáfora **IDE/workspace** es más natural:
   mantener "Studio" y usarla de forma consistente.
4. **Densidad de columna configurable.** El límite real de memoria de trabajo es ~4 ítems
   (Cowan), pero en grids se aplica a la *relevancia*, no al conteo de filas (percepción ≠
   memoria). Columnas configurables + densidad ajustable + modo comparación.
5. **On-boarding cognitivo.** Empty states con ejemplos y templates de namespace; el
   cerebro novato necesita guía y estructura inicial.

---

## 5. Prioridad de implementación de los fixes

| Prioridad | Fix | Fase sugerida |
|-----------|-----|---------------|
| P0 | Overview/HOME (Fix 1) | Fase 0 |
| P0 | Workspace unificado en vez de 5 tabs (Fix 2) | Fase 0 (reestructura `App.tsx`) |
| P0 | Diff entre versiones + timeline (Fix 3) | Fase 1 |
| P0 | Undo + papelera + command palette (Fix 4) | Fase 0–1 |
| P1 | Barras apiladas para scores + ring TTL (Fix 5) | Fase 1 |
| P1 | A11y redundant encoding + notas + reporte legible (menores) | Fase 1–2 |

---

## 6. Referencias

- Shneiderman, B. (1996). *The eyes have it: A task by data type taxonomy for information
  visualizations.* IEEE Symposium on Visual Languages. — mantra overview/zoom/details.
- Cleveland, W. & McGill, R. (1984). *Graphical Perception: Theory, Experimentation, and
  Application to the Development of Graphical Methods.* JASA 79(387). — jerarquía de
  precisión de canales visuales.
- Mackinlay, J. (1986). *Automating the Design of Graphical Presentations of Relational
  Information.* ACM TOG. — canal visual óptimo por tipo de dato.
- Ware, C. (2020). *Information Visualization: Perception for Design* (4.ª ed.). Morgan
  Kaufmann. — atributos preatentivos (forma, color, posición, movimiento).
- Healey, C. — *Perception in Visualization* (NCSU): resumen de teoría psicofísica y
  procesamiento preatentivo. https://www.csc2.ncsu.edu/faculty/healey/PP/
- *Chapter 2: How the Eye Sees — Pre-Attentive Processing and Visual Encoding.*
  https://datafield.dev/data-visualization-python/part-01/chapter-02/
- *Preattentive attributes of visual perception and their application to data
  visualizations* (uxdesign.cc). https://uxdesign.cc/preattentive-attributes-of-visual-perception-and-their-application-to-data-visualizations-7b0fb50e1375
- Sweller, J. (1988). *Cognitive load during problem solving: Effects on learning.* Cognitive
  Science 12. — carga intrínseca/extrínseca/germana.
- Mayer, R. (2009). *Multimedia Learning* (2.ª ed.). Cambridge. — efecto split-attention.
- Norman, D. (2013). *The Design of Everyday Things* (rev. ed.). Basic Books. — feedback,
  affordances, recuperación de errores.
- Pirolli, P. & Card, S. (1999). *Information Foraging.* Psychological Review. — el humano
  minimiza el costo de buscar información/acciones.
- Cowan, N. (2001). *The magical number 4 in short-term memory.* BBS 24. — límite real de
  memoria de trabajo (~4 ítems).
- Miller, G. (1956). *The magical number seven, plus or minus two.* — a menudo mal aplicado
  a grids; percepción ≠ memoria.
- Okabe, M. & Ito, K. — *Color Universal Design* palettes (accesibles para daltonismo).
- Few, S. — *Information Dashboard Design* (O'Reilly). — "cada número debe poder
  defenderse".
- Tufte, E. (1983). *The Visual Display of Quantitative Information.* Graphics Press. —
  data-ink ratio.
- Springer (2021). *Cognitive Processing of Information Visualization* (encyclopedia entry).
  https://link.springer.com/rwe/10.1007/978-3-319-08234-9_95-1
- *Navigation That Thinks: The Cognitive Economics of Visual Encoding.* https://timgraf.com/ux-design/navigation-that-thinks-the-cognitive-economics-of-visual-encoding-how-pre-attentive-processing-and-gestalt-principles-shape-decision-latency-in-data-visualization-design/
- *The Psychology behind Data Visualization Techniques* (Towards Data Science).
  https://towardsdatascience.com/the-psychology-behind-data-visualization-techniques-68ef12865720/