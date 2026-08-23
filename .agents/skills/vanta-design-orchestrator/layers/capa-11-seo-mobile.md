# CAPA 11 — SEO + MOBILE [OPCIONAL]

> ⚠️ **Capa condicional.** Se activa SOLO si el proyecto necesita optimización SEO, diseño mobile-first, o estrategia de visibilidad en buscadores/LLMs. Preguntar al usuario al iniciar la tarea.

Los skills individuales están referenciados en CAPA 6 (`roier-seo`, `ai-seo`, `seo`). Esta capa proporciona el flujo de trabajo combinado.

### Flujo SEO + Mobile

```mermaid
graph LR
    SEO[Auditoría SEO Técnica] -->|seo-audit| AI[AI SEO]
    AI -->|ai-seo| DEPLOY[Producción]
    SEO -->|seo| DEPLOY

    style SEO fill:#1a1a2e,stroke:#16213e,color:#e0e0e0
    style AI fill:#2a1a1a,stroke:#4a2a2a,color:#fff
    style DEPLOY fill:#1e241e,stroke:#2f4a2f,color:#fff
```

- **Fase 4 (Auditoría):** Correr `seo-audit` para auditoría técnica → `ai-seo` para optimización AI visibility → `seo` para correcciones on-page.
- **Criterio de salida:** Scores Lighthouse >90 en SEO y Performance. Diseño mobile validado en 3 dispositivos (4.7", 6.1", 6.7"). Contenido optimizado para AI search (OKF/llms.txt).
