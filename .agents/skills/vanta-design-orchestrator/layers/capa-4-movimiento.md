# CAPA 4 — INTERACCIONES Y MOVIMIENTO

---

## 9. `emil-design-eng` — Filosofía de Microinteracciones

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Base de conocimiento que codifica la filosofía de diseño de Emil Kowalski sobre detalles invisibles, springs de animación y los micro-detalles que hacen que el software se sienta extraordinario. |
| **¿Para qué es?** | Diseñar transiciones y hovers dinámicos que se sientan físicos, fluidos y de calidad premium. Definir la "personalidad" del movimiento de la interfaz. |
| **¿Para qué se usa?** | Definir constantes de easing inspiradas en física (spring), duración de animaciones (150-300ms), efectos hover en botones y tarjetas, border-radius coherente, sombras con intención, y la regla de que "lo bueno es invisible — los usuarios no notan las buenas animaciones, solo notan las malas". |
| **¿Cómo se usa?** | Aplicando las directrices: respuesta física inmediata (<100ms), easing de salida con deceleración, sin rebotes excesivos, sin `ease-in` en UI. Hover effects con `transform: scale(1.02)` + shadow sutil, no `scale(1.1)` que se siente agresivo. |
| **¿Cómo debería usarse?** | Como complemento de `interaction-design`. Emil define la filosofía; `interaction-design` define los patrones técnicos (state machines, loading states). |
| **¿Cuándo debería usarse?** | **Fase 3** — Al diseñar cualquier elemento interactivo: menús, botones, popups, control dinámico de Three.js. |
| **Dependencias** | Skill de proyecto (`.agent/skills/emil-design-eng/`). No requiere instalación — es filosofía de micro-interacciones. |
| **Requerimientos** | Ninguno. Funciona sobre cualquier stack. Complementa a `motion` y `interaction-design`. |

## 10. `interaction-design` — Patrones de Interacción Completos

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Suite de 13 sub-skills que cubren: principios de animación (easing, duración, stagger), leyes cognitivas (Doherty <400ms, Fitts target sizing, Hick decision reduction, Miller chunking), manejo de errores UX, patrones de feedback, diseño de formularios, patrones de gestos, estados de carga (skeleton, optimistic UI, progressive), especificación de micro-interacciones (trigger/rules/feedback/loops), diseño de navegación (tab bar, sidebar, breadcrumbs), onboarding (progressive, wizard, sample data), search UX (autocomplete, zero-results, faceted) y state machines (FSM para UI). |
| **¿Para qué es?** | Diseñar interacciones completas fundamentadas en ciencia cognitiva y patrones probados. |
| **¿Para qué se usa?** | Modelar flujos como state machines (idle→loading→success/error). Aplicar Doherty Threshold: feedback visual <100ms, loading indicator >400ms, progress >3s. Diseñar formularios con validación inline on-blur. Diseñar search con autocomplete <300ms. Target sizing con Fitts: 44×44pt mínimo en touch. Chunking con Miller: agrupar en bloques de 4±1. |
| **¿Cómo se usa?** | Invocando la sub-skill según la necesidad: `form-design` para formularios, `loading-states` para carga, `state-machine` para FSM, `error-handling-ux` para errores, `navigation-patterns` para nav, `onboarding-design` para first-run, `search-ux` para búsqueda. |
| **¿Cómo debería usarse?** | Los patrones de interacción se fundamentan con las leyes cognitivas (`doherty-threshold`, `fitts-law`, `hicks-law`, `millers-law`). Cada decisión de interacción debe citar qué ley respalda el diseño. |
| **¿Cuándo debería usarse?** | **Fase 3** — Al implementar comportamientos interactivos, flujos de datos y feedback de sistema. |
| **Workflows disponibles** | `/interaction-design:design-interaction`, `/interaction-design:error-flow`, `/interaction-design:map-states` |
| **Dependencias** | Skill de skills.sh (se instaló con `npx skills add wshobson/agents@interaction-design -g`). No requiere paquetes npm adicionales — es conocimiento de patrones de interacción con 13 sub-skills. |
| **Requerimientos** | Ninguno. Funciona sobre cualquier stack. Opcional: herramientas de prototipado (Figma, ProtoPie) para validar flujos. |

## `motion` — Librería de Animaciones (motion.dev)

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Librería de animaciones para JavaScript, React y Vue de motion.dev (v12). Reemplazo moderno de Framer Motion con API simplificada. Soporta `motion.div`, layout animations, scroll-linked effects, y gesture-driven motion. |
| **¿Para qué es?** | Implementar todas las animaciones del proyecto con una sola librería declarativa: entradas, salidas, transiciones de layout, hover, tap, drag y scroll. Es la librería de animación PREDETERMINADA del proyecto. |
| **¿Para qué se usa?** | Animar componentes con `<motion.div>`, escalar con `whileHover`, transiciones de layout con `layoutId`, scroll animations con `useScroll`, y gestos con `whileTap` / `whileDrag`. Sin CSS keyframes — todo declarativo. |
| **¿Cómo se usa?** | `import { motion } from "motion"` → `<motion.div initial={{opacity:0}} animate={{opacity:1}} transition={{duration:0.3}}>`. Para scroll: `const { scrollYProgress } = useScroll()`. |
| **¿Cómo debería usarse?** | Como librería única de animación. No mezclar con CSS keyframes ni otras librerías para el mismo tipo de animación. `prefers-reduced-motion` lo maneja motion.dev automáticamente. |
| **¿Cuándo debería usarse?** | **Fase 3** — En toda animación de UI: transiciones, hover, scroll, layout, micro-interacciones. Reemplaza CSS keyframes y Framer Motion legacy. |
| **Dependencias** | `npm install motion` (motion.dev v12+). Es la librería de animación por defecto del proyecto. Reemplaza a Framer Motion. |
| **Requerimientos** | Proyecto con npm/pnpm/yarn. React 16.8+ o Vue 3+. Node 18+. Sin dependencias nativas. |

## `animejs` — Animaciones de Timeline y SVG

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Librería versátil de animación JavaScript que trabaja con DOM, CSS, SVG y objetos JS. Especializada en timelines con coreografía precisa, stagger grid, morphing SVG, y easing spring físico. ~9KB gzipped. |
| **¿Para qué es?** | Animaciones complejas con secuencias multi-paso (timelines), animaciones escalonadas sobre grids (stagger desde centro/filas), morphing de rutas SVG, y keyframes con temporización porcentual. |
| **¿Para qué se usa?** | Coreografiar secuencias con `anime.timeline()` y posicionamiento relativo (`-=500`). Animar desde centro de grid con `stagger()`. Morphing SVG con atributos `d`. Animar objetos JS para datos reactivos. |
| **¿Cómo se usa?** | `import anime from 'animejs'` → `anime({ targets: '.el', translateX: 250, duration: 800, easing: 'spring(1, 80, 10, 0)' })`. Timelines: `anime.timeline().add({...}).add({...}, '-=500')`. Stagger: `anime({ targets, delay: anime.stagger(100, {from: 'center'}) })`. |
| **¿Cómo debería usarse?** | Usar `anime` para animaciones que requieren control temporal fino (timelines, SVG morphing, stagger grid). Usar `motion` (motion.dev) para animaciones declarativas de UI (hover, layout, scroll). No mezclar ambas en el mismo componente — elegir según el caso. |
| **¿Cuándo debería usarse?** | **Fase 3** — Cuando se necesita: timelines multi-secuencia con posicionamiento relativo, animaciones SVG (morphing, draw), stagger sobre colecciones/grids, o easing spring avanzado no disponible en CSS. |
| **Dependencias** | Skill de skills.sh (se instaló con `npx skills add freshtechbro/claudedesignskills@animejs -g`). Proyecto: `npm install animejs`. Incluye scripts Python para generación de animaciones (`scripts/animation_generator.py`, `scripts/timeline_builder.py`). |
| **Requerimientos** | Node 18+. Proyecto con npm/pnpm/yarn. Sin dependencias nativas. Anime.js v4 (última). Compatible con todos los navegadores modernos. Para scripts Python: Python 3.8+. |

## `design-motion-principles` — Auditoría de Movimiento

| Campo | Detalle |
| :-- | :-- |
| **¿Qué es?** | Skill de auditoría de motion basado en las técnicas de Emil Kowalski, Jakub Krehel y Jhey Tompkins. Dos modos: construir componentes con movimiento intencional, o auditar animaciones existentes y detectar patrones "slop". |
| **¿Para qué es?** | Revisar y mejorar la calidad del movimiento en la interfaz. Detectar easing genéricos, duraciones incorrectas, hover effects agresivos, y falta de intencionalidad en transiciones. |
| **¿Para qué se usa?** | Auditar motion existente (genera reporte HTML con demos en loop). Definir personalidad de movimiento del proyecto (constantes de spring, duración base, easing tokens). |
| **¿Cómo se usa?** | En modo audit: corre reglas de calidad de motion y emite reporte. En modo build: consulta perspectivas por diseñador (Emil → micro-interacciones, Jakub → layout, Jhey → scroll/parallax). |
| **¿Cómo debería usarse?** | Como complemento de `motion` y `emil-design-eng`. Primero construir con `motion`, luego auditar con `design-motion-principles`, luego refinar con `impeccable`. Sobre `prefers-reduced-motion`: motion.dev lo maneja automáticamente. |
| **¿Cuándo debería usarse?** | **Fase 3-4** — Después de implementar animaciones, antes de la auditoría final. Opcional si el proyecto usa solo motion básico sin scroll ni gestos complejos. |
| **Dependencias** | Skill de skills.sh (se instaló con `npx skills add wshobson/agents@design-motion-principles -g`). No requiere paquetes npm — es conocimiento de patrones y genera reporte HTML autónomo. |
| **Requerimientos** | Ninguno. El reporte de auditoría se genera como HTML estático. |
