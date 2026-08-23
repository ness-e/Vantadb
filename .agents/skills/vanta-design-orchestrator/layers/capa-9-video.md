# CAPA 9 — VIDEO [OPCIONAL]

> ⚠️ **Capa condicional.** Se activa SOLO si el proyecto incluye producción de video, motion graphics o composiciones animadas. Preguntar al usuario al iniciar la tarea.

---

## `hyperframes` — Composición de Video con HTML

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Framework de código abierto para convertir HTML, CSS, animaciones seekable y media en videos MP4 deterministas. Usa el CLI para preview, lint y render. |
| **¿Para qué es?** | Producir videos desde HTML con animaciones GSAP, captions sincronizados, audio-reactive visuals, transiciones de escena, y composiciones multi-secuencia. |
| **¿Para qué se usa?** | Crear videos promocionales, explainers, caption overlays, PR videos, motion graphics, y animaciones para web. Flujo: plan → HTML → animaciones → lint → preview → render → MP4. |
| **¿Cómo se usa?** | `npx hyperframes init` → escribe `index.html` con data-* attributes → `npx hyperframes preview` → `npx hyperframes render output.mp4`. Usa GSAP para timelines seekable. |
| **¿Cómo debería usarse?** | Para proyectos que necesitan video generado programáticamente. No para edición de video tradicional. HyperFrames es el entry point; `hyperframes-animation` añade conocimiento profundo de animación. |
| **¿Cuándo debería usarse?** | Cuando el proyecto requiere video generado desde código (promos, explainers, captions, motion graphics). Activar CAPA 9 solo si el usuario confirma que necesita video. |
| **Dependencias** | `npx skills add heygen-com/hyperframes -g`. CLI: `npx hyperframes`. Requiere Node 18+. Para render: Chrome/Chromium (lo usa Puppeteer internamente). |
| **Requerimientos** | Node 18+. Chrome/Chromium. GPU recomendada para renders complejos. 4GB+ RAM. |

## `hyperframes-animation` — Animación para HyperFrames

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Todo el conocimiento de animación para HyperFrames: reglas de motion atómico, blueprints de escena multi-fase, transiciones, y 7 adaptadores runtime (GSAP, Lottie, Three.js, Anime.js, CSS keyframes, WAAPI, TypeGPU). |
| **¿Para qué es?** | Implementar animaciones seekable y deterministas dentro de composiciones HyperFrames. Cubre desde easing básico hasta shaders de transición. |
| **¿Para qué se usa?** | Animar elementos HTML, sincronizar motion con audio, crear transiciones entre escenas, y usar adaptadores específicos (Lottie para vector animation, Three.js para 3D, etc.). |
| **¿Cómo se usa?** | Seleccionando un blueprint de escena (entrance → body → exit), eligiendo el adaptador adecuado, y aplicando reglas de motion atómico (duración, easing, stagger). |
| **¿Cómo debería usarse?** | Después de leer `hyperframes`. Usar GSAP como default, otros adaptadores solo si el caso lo justifica. |
| **¿Cuándo debería usarse?** | Junto con `hyperframes` cuando se necesita control fino de animación en composiciones de video. |
| **Dependencias** | Se instala con el skill `hyperframes` (`npx skills add heygen-com/hyperframes@hyperframes-animation -g`). Depende de `hyperframes` CLI. |
| **Requerimientos** | Los mismos que `hyperframes`. GSAP (opcional, vía CDN). Cada adaptador requiere su propia librería. |

## `remotion-best-practices` — Video con React (Remotion)

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Skill de mejores prácticas para Remotion — framework de creación de video con React. Cubre performance, composición, rendering, y patrones específicos de Remotion. |
| **¿Para qué es?** | Crear videos usando componentes React: animaciones programáticas, data-driven motion graphics, y composiciones dinámicas renderizadas con hooks de Remotion. |
| **¿Para qué se usa?** | Animar con hooks de Remotion (`useCurrentFrame`, `useVideoConfig`), componer escenas con `<Sequence>`, optimizar render con `<useOffthreadVideo>`, y manejar assets. |
| **¿Cómo se usa?** | Consultando reglas de performance específicas de Remotion: evitar re-renders en cada frame, usar `useMemo` para valores estáticos, `<Img>` en vez de `<img>`, y `continueRender` para assets async. |
| **¿Cómo debería usarse?** | Alternativa a HyperFrames cuando el equipo ya usa React y prefiere componentes sobre HTML plano. No mezclar con HyperFrames en el mismo proyecto — elegir uno. |
| **¿Cuándo debería usarse?** | Solo si el usuario prefiere explícitamente Remotion sobre HyperFrames, o si el proyecto ya tiene código Remotion existente. |
| **Dependencias** | Skill de skills.sh (`npx skills add remotion-dev/skills@remotion-best-practices -g`). Proyecto React con `npm install @remotion/cli`. |
| **Requerimientos** | Node 18+. React 18+. Chrome/Chromium (para render). GPU recomendada. 8GB+ RAM para renders largos. |
