# Business Model Design — Estrategia de Negocio para Diseño

> Antes de diseñar, entender cómo la empresa genera valor. Esta skill alinea cada decisión visual con la viabilidad económica del proyecto.

---

## 1. Business Model Canvas (BMC)

Los 9 bloques fundamentales que definen cómo opera y captura valor un negocio. El diseño debe responder a cada bloque:

| Bloque | Pregunta de diseño | Implicación visual/UX |
|:-------|:-------------------|:----------------------|
| **Propuesta de Valor** | ¿Qué problema resolvemos? | Hero section debe comunicar el valor en <3s. USP claro |
| **Segmentos de Cliente** | ¿Para quién? | Diseño orientado al perfil demográfico/psicográfico específico |
| **Canales** | ¿Cómo llegamos? | Consistencia cross-platform (web, mobile, físico) |
| **Relación con Clientes** | ¿Cómo interactuamos? | UX de soporte, self-service, onboarding |
| **Flujo de Ingresos** | ¿Cómo pagan? | Arquitectura de pricing page, upsell flows, paywalls |
| **Recursos Clave** | ¿Qué necesitamos? | Design system como recurso escalable |
| **Actividades Clave** | ¿Qué hacemos? | Feature prioritization en UI |
| **Socios Clave** | ¿Con quién trabajamos? | Co-branding, integraciones third-party visibles |
| **Estructura de Costos** | ¿Cuánto cuesta? | Eficiencia de diseño: componentes reutilizables, tokens |

### Uso en el flujo

```
BMC completo → identifica qué bloque es crítico para el negocio →
  el diseño prioriza ese bloque (ej: si "Canales" es crítico →
    diseño responsive impecable en cada touchpoint)
```

---

## 2. Value Proposition Canvas

Conecta el **perfil del cliente** (Customer Profile) con el **mapa de valor** (Value Map):

### Customer Profile
- **Customer Jobs:** ¿Qué tareas funcionales, sociales y emocionales intenta resolver el usuario?
- **Pains:** ¿Qué frustraciones, obstáculos y riesgos enfrenta?
- **Gains:** ¿Qué resultados o beneficios desea?

### Value Map
- **Products & Services:** Lista concreta de lo que ofrece el producto
- **Pain Relievers:** Cómo cada feature elimina un pain específico
- **Gain Creators:** Cómo cada feature genera un gain específico

### Fit
El diseño logra **fit** cuando el Value Map cubre los elementos más importantes del Customer Profile.

**Aplicación directa:** Cada sección de la landing page debe mapear a un elemento del VPC. Si no hay mapping, la sección sobra.

---

## 3. Jobs To Be Done (JTBD)

Marco de Clayton Christensen: los usuarios "contratan" productos para hacer un trabajo.

### Dimensiones del Job
- **Funcional:** ¿Qué tarea concreta? (ej: "encontrar un vuelo barato")
- **Emocional:** ¿Cómo quieren sentirse? (ej: "sentirse inteligente por ahorrar dinero")
- **Social:** ¿Cómo quieren ser percibidos? (ej: "ser visto como un viajero experimentado")

### Estructura de un JTBD
```
When ____ [situación], I want to ____ [motivación], so I can ____ [resultado esperado].
```

### Jobs Map
```
1. Define → 2. Locate → 3. Prepare → 4. Confirm → 5. Execute → 6. Monitor → 7. Modify → 8. Conclude
```

**Aplicación:** El diseño debe eliminar fricción en cada paso del Jobs Map. Cada pantalla responde a un step específico.

---

## 4. Investigación y Diagnóstico Estratégico

### Auditoría Interna
- **Análisis de embudo actual:** ¿Dónde pierde usuarios?
- **Inventario de diseño existente:** ¿Hay design system? ¿Tokens? ¿Deuda visual?
- **Score de consistencia:** ¿La marca se ve igual en todos los canales?

### Auditoría Externa (Competencia)
- Mapear **territorio, promesa de valor y personalidad** de cada competidor
- Identificar **vacíos de mercado** (oportunidades que nadie está aprovechando)
- Evaluar **madurez de diseño** de la competencia

### Output esperado
```
Diagnóstico → Oportunidades → Recomendaciones estratégicas → Design brief
```

---

## 5. Ejemplo End-to-End: SaaS de Gestión de Proyectos

### Contexto
Una startup quiere lanzar un SaaS de gestión de proyectos para equipos remotos. Necesitan definir el modelo de negocio antes de diseñar cualquier pantalla.

### Paso 1: BMC completo

| Bloque | Decisión |
|:-------|:---------|
| **Propuesta de Valor** | "Gestión de proyectos async-first para equipos remotos" |
| **Segmentos** | SMEs de 10-50 personas, equipos distribuidos |
| **Canales** | Web app + mobile app + Slack/Teams integration |
| **Relación** | Self-serve onboarding + chat support |
| **Ingresos** | SaaS mensual: $10/user/mes (starter), $25/user/mes (pro) |
| **Recursos** | Design system como activo principal de consistencia |
| **Actividades** | Desarrollo continuo de features + contenido educativo |
| **Socios** | Slack, Notion, Google Workspace |
| **Costos** | Hosting (AWS), equipo (5 devs + 1 designer) |

### Paso 2: Value Proposition Canvas

**Customer Profile (PM)**
- **Jobs:** Coordinar tareas entre husos horarios, visibilidad de avance
- **Pains:** Reuniones innecesarias, información dispersa en Slack/email
- **Gains:** Un solo lugar para todo, actualización asíncrona

**Value Map**
- **Products:** Timeline async, kanban, docs integrados
- **Pain Relievers:** Comentarios async en lugar de reuniones, búsqueda unificada
- **Gain Creators:** Dashboard de productividad del equipo, integración calendario

### Paso 3: JTBD → Diseño

**JTBD principal:** "When I'm managing a remote team across 3 timezones, I want to see what everyone is working on without scheduling a meeting, so I can unblock them faster and stop wasting time in status meetings."

**Jobs Map → Features:**

| Step | Job | Feature de diseño |
|:-----|:----|:------------------|
| Define | Crear proyecto | Wizard de 3 pasos, templates pre-hechos |
| Locate | Encontrar tareas | Búsqueda global + filtros por persona/fecha |
| Prepare | Asignar trabajo | Drag & drop desde backlog al sprint |
| Confirm | Aceptar tarea | Notificación push + botón "confirm" |
| Execute | Trabajar | Editor rich text + comentarios async |
| Monitor | Ver avance | Dashboard visual (burndown, timeline) |
| Modify | Reasignar | Drag & drop + notificación automática |
| Conclude | Cerrar proyecto | Retrospective template + archive |

### Paso 4: Output al equipo de diseño

```markdown
## Design Brief desde BMC

**Propuesta de Valor:** Gestión async-first → 
  Hero debe mostrar "sin reuniones" en <3s

**Segmento:** SMEs remotas →
  UI optimizada para equipos pequeños (no enterprise bloat)

**Ingresos:** $10/$25 per user →
  Pricing page clara, comparativa planes, upgrade flow

**Canales:** Web + Mobile + Slack →
  Design system responsive, mobile-first, Slack deep links

**Diferenciador:** Async sobre sync →
  Timeline visual como feature hero, no el kanban

**Prioridad visual:** Dashboard > Timeline > Kanban > Docs
```

### Outputs del ejercicio
- BMC completo con 9 bloques validados
- Value Proposition Canvas con fit logrado
- JTBD principal documentado + Jobs Map completo
- Design brief listo para la fase de diseño visual
- Decisiones de pricing que impactan arquitectura de UI
- Mapa de canales que define responsive strategy

---

## Integración con el Orquestador

Esta skill se activa cuando el usuario menciona:
- "business model", "modelo de negocio", "BMC", "canvas"
- "value proposition", "propuesta de valor"
- "jobs to be done", "JTBD"
- "modelo de ingresos", "cómo ganan dinero"
- "auditoría de competencia", "competitive analysis"
- "viabilidad", "factibilidad", "deseabilidad"

**Flujo:** `strategy/business-model-design.md` → `strategy/brand-platform.md` → `ux-strategy` → Fase 1-4

**Ver también:** `strategy/decision-hierarchy.md` (jerarquía de decisiones), `strategy/validation-sustainability.md` (triple balance)
