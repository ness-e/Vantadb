# Tool Registry — Ecosistema de Herramientas Gratuitas y Open Source

> El orquestador no solo ejecuta diseño — también sabe qué herramientas existen, cómo instalarlas, y cuándo usarlas.
> Este registro cataloga **40+ herramientas gratuitas/open-source** organizadas por capacidad, con instrucciones de integración y ejemplos de uso via Function Calling.

---

## 1. Browser Automation & Screenshot

### 1.1 Playwright

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería Python/Node.js (Open Source) |
| **Repo** | `microsoft/playwright` |
| **Instalación** | `pip install playwright && playwright install chromium` o `npx playwright install` |
| **Costo** | 100% gratis. Sin API key. Auto-alojado |
| **Qué permite** | Navegador headless controlado por código. Renderiza SPAs, extrae CSS computed, toma screenshots, ejecuta JS en página |
| **Uso en el orquestador** | Inspeccionar sitios de referencia, extraer sistemas de diseño de competidores, validar responsive en múltiples viewports |
| **Alternativa ligera** | Puppeteer Core (solo Chromium, sin dependencias extra) |

**Ejemplo de uso por el agente:**
```python
# Dentro de una tool call del agente:
page = await browser.new_page()
await page.goto("https://ejemplo.com")
# Extraer CSS computed de un elemento
color = await page.eval_on_selector(".hero-title", "el => getComputedStyle(el).color")
# Screenshot de sección específica
await page.locator(".hero-section").screenshot(path="hero-ref.png")
```

### 1.2 Browserless (Community Edition)

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Docker service (Open Source) |
| **Repo** | `browserless/browserless` |
| **Instalación** | `docker run -p 3000:3000 ghcr.io/browserless/browserless` |
| **Costo** | 100% gratis. Auto-alojado en Docker |
| **Qué permite** | Infraestructura headless browser con WebSocket, cola de requests, panel de gestión. Recibe peticiones de agentes remotos |
| **Cuándo usarlo** | Cuando necesitas un servicio de navegador persistente al que multiples agentes hacen requests simultáneas |

### 1.3 Puppeteer Core

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería Node.js (Open Source) |
| **Repo** | `puppeteer/puppeteer` |
| **Instalación** | `npm install puppeteer-core` (sin Chromium incluido — usar Chromium del sistema) |
| **Costo** | 100% gratis |
| **Qué permite** | Control programático de Chromium. Más ligero que Playwright si solo necesitas Chromium. Ideal para screenshots rápidos de componentes |

### Trigger words
"playwright", "puppeteer", "browserless", "headless browser", "automatización navegador", "screenshot de página", "inspeccionar sitio", "extraer CSS de página", "navegador controlado", "web scraping con navegador", "SPA rendering"

---

## 2. Web Search & Discovery

### 2.1 DuckDuckGo Search

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería Python (Open Source) |
| **Instalación** | `pip install duckduckgo_search` |
| **Costo** | 100% gratis. Sin API key. Sin límites oficiales |
| **Qué permite** | Búsqueda de texto e imágenes. Scraping de resultados sin bloqueo |
| **Uso en el orquestador** | Buscar referencias de diseño, tendencias, componentes, documentación de librerías |

### 2.2 SearXNG

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Docker service (Open Source) |
| **Repo** | `searxng/searxng` |
| **Instalación** | `docker pull searxng/searxng` |
| **Costo** | 100% gratis. Auto-alojado. Sin límites |
| **Qué permite** | Meta-búsqueda: agrega resultados de 70+ motores (Google, Bing, GitHub, Stack Overflow, Reddit). JSON output optimizado para IA |
| **Uso en el orquestador** | Búsqueda federada de referencias de diseño, componentes, código de ejemplo. Sin anuncios ni bloqueo |

### 2.3 Tavily API

| Campo | Detalle |
|:------|:--------|
| **Tipo** | API (Free Tier) |
| **Web** | `tavily.com` |
| **Costo** | 1,000 búsquedas/mes gratis. Luego pay-per-use |
| **Qué permite** | Búsqueda optimizada para agentes de IA. Resultados resumidos y limpios, sin ruido |
| **Cuándo usarlo** | Cuando el agente necesita respuestas rápidas y estructuradas sin procesar HTML |

### Trigger words
"duckduckgo search", "searxng", "tavily", "búsqueda web", "web search", "buscar referencias", "buscar componentes", "buscar documentación", "meta search", "búsqueda federada"

---

## 3. APIs de Plataformas de Diseño y Código

### 3.1 Figma REST API

| Campo | Detalle |
|:------|:--------|
| **Tipo** | API REST (acceso gratuito) |
| **Docs** | `figma.com/developers/api` |
| **Costo** | Gratis para desarrollo. Autenticación con token personal |
| **Qué permite** | Extraer estructura JSON completa de archivos Figma: nodos, coordenadas, colores, tipografías, estilos. Sin procesar imágenes |
| **Uso en el orquestador** | Leer sistemas de diseño de Figma, exportar tokens de diseño, inspeccionar componentes públicos |

### 3.2 GitHub REST / GraphQL API

| Campo | Detalle |
|:------|:--------|
| **Tipo** | API (gratis para repos público) |
| **Docs** | `docs.github.com/en/rest` |
| **Costo** | 5,000 peticiones/hora gratis con token personal |
| **Qué permite** | Buscar código, leer archivos raw, explorar repos de componentes, analizar librerías, buscar ejemplos de implementación |
| **Endpoint clave** | `GET /search/code?q=repo:user/repo+keyword` `raw.githubusercontent.com/user/repo/branch/path` |

### 3.3 Google Fonts API

| Campo | Detalle |
|:------|:--------|
| **Tipo** | API REST (gratis) |
| **Docs** | `developers.google.com/fonts/docs/css2` |
| **Costo** | 100% gratis |
| **Qué permite** | Listar fuentes, obtener metadatos (weights, styles, axes variables), descargar WOFF2 |
| **Endpoint** | `GET https://fonts.googleapis.com/css2?family=Inter:wght@400;700` |

### 3.4 Fontsource API

| Campo | Detalle |
|:------|:--------|
| **Tipo** | CDN + API (Open Source) |
| **Web** | `fontsource.org` |
| **Costo** | 100% gratis. Self-hosted fonts |
| **Qué permite** | Descargar y auto-alojar fuentes open-source en lugar de depender de Google Fonts. Ideal para GDPR/privacy |
| **Instalación** | `npm install @fontsource/inter` |

### Trigger words
"figma api", "github api", "google fonts", "fontsource", "extraer tokens figma", "buscar código github", "descargar fuente", "API de diseño", "leer archivo figma", "buscar repositorio"

---

## 4. Agent Frameworks & Function Calling

### 4.1 LangChain / LangGraph

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Framework Python/TypeScript (Open Source) |
| **Repo** | `langchain-ai/langchain` |
| **Instalación** | `pip install langchain langchain-community` |
| **Costo** | 100% gratis. Open Source MIT |
| **Qué permite** | Cientos de herramientas pre-construidas: conectores GitHub, buscadores web, parsers HTML, herramientas de código. Orquestación de chains con estado |
| **Uso en el orquestador** | Encadenar múltiples herramientas: buscar referencia → extraer diseño → generar código → auditar |

### 4.2 CrewAI

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Framework Python (Open Source) |
| **Repo** | `crewAIInc/crewAI` |
| **Instalación** | `pip install crewai` |
| **Costo** | 100% gratis. Open Source MIT |
| **Qué permite** | Coordinación multi-agente. Configurar roles: "Agente Diseñador" (usa Figma API) + "Agente Desarrollador" (usa Playwright) + "Agente Auditor" (usa axe-core) trabajando en paralelo |
| **Uso en el orquestador** | Simular un estudio de diseño completo con agentes especializados colaborando |

### 4.3 AutoGen

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Framework Python (Open Source) |
| **Repo** | `microsoft/autogen` |
| **Instalación** | `pip install pyautogen` |
| **Costo** | 100% gratis. Open Source MIT |
| **Qué permite** | Agentes conversacionales que pueden llamar herramientas y pasarse mensajes entre sí |
| **Diferenciador** | Mejor para debugging: cada agente puede explicar su razonamiento paso a paso |

### Trigger words
"langchain", "langgraph", "crewai", "autogen", "agente de IA", "multi-agente", "function calling", "chain de herramientas", "orquestación de agentes", "framework de agentes"

---

## 5. HTML Content Extraction & Parsing

### 5.1 Trafilatura

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería Python (Open Source) |
| **Instalación** | `pip install trafilatura` |
| **Costo** | 100% gratis |
| **Qué permite** | Extraer texto principal de páginas web eliminando menús, footers, anuncios. Output en Markdown o JSON. Superior a BeautifulSoup para IA |
| **Uso en el orquestador** | Leer artículos de diseño, documentación, y referencias. Ahorra tokens porque solo entrega el contenido relevante |

### 5.2 Mozilla Readability

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería JS (Open Source) |
| **Repo** | `mozilla/readability` |
| **Instalación** | `npm install @mozilla/readability` |
| **Costo** | 100% gratis |
| **Qué permite** | El mismo algoritmo de "Modo Lectura" de Firefox. Convierte HTML en árbol semántico limpio |
| **Uso en el orquestador** | Reducir tokens drásticamente antes de enviar contenido de página al LLM |

### 5.3 BeautifulSoup4

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería Python (Open Source) |
| **Instalación** | `pip install beautifulsoup4` |
| **Costo** | 100% gratis |
| **Qué permite** | Parser de HTML con navegación por árbol. Útil para extracción específica de datos estructurados |

### Trigger words
"trafilatura", "readability", "beautifulsoup", "extraer contenido", "content extraction", "parsear HTML", "modo lectura", "limpiar HTML", "scraping", "parser HTML"

---

## 6. Image & OG Generation

### 6.1 Satori

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería JS (Open Source) |
| **Repo** | `vercel/satori` |
| **Instalación** | `npm install satori` |
| **Costo** | 100% gratis. Open Source MPL-2.0 |
| **Qué permite** | Convertir HTML+CSS a SVG. Con resvg-js, convertir ese SVG a PNG. Ideal para generar OG images, social cards, y certificados programáticamente |
| **Uso en el orquestador** | Generar OG images para proyectos, crear social cards dinámicas, producir previews de diseño |
| **Limitación** | Solo flexbox layout. No grid, no position:absolute complejo |

### 6.2 Sharp

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería Node.js (Open Source) |
| **Repo** | `lovell/sharp` |
| **Instalación** | `npm install sharp` |
| **Costo** | 100% gratis |
| **Qué permite** | Procesamiento de imágenes de alto rendimiento: redimensionar, recortar, rotar, convertir formatos, compositing, aplicar filtros |
| **Uso en el orquestador** | Post-procesar imágenes generadas: optimizar tamaño, convertir a WebP/AVIF, crear thumbnails, compositing de mockups |

### 6.3 Pillow (PIL)

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería Python (Open Source) |
| **Instalación** | `pip install Pillow` |
| **Costo** | 100% gratis |
| **Qué permite** | Procesamiento de imágenes en Python: redimensionar, filtros, dibujar texto/formas, compositing |
| **Uso en el orquestador** | Alternativa a Sharp cuando el pipeline es Python. Crear placeholders, watermarks, thumbnails |

### Trigger words
"satori", "sharp", "pillow", "og image", "generar imagen", "social card", "optimizar imagen", "convertir imagen", "WebP", "AVIF", "image processing", "HTML a SVG", "SVG a PNG"

---

## 7. Font Manipulation

### 7.1 Glypher

| Campo | Detalle |
|:------|:--------|
| **Tipo** | CLI + Librería TypeScript (Open Source) |
| **Repo** | `TupiC/glypher` |
| **Instalación** | `npx glypher` (sin dependencias externas). Sucesor de glyphhanger |
| **Costo** | 100% gratis |
| **Qué permite** | Subsetting de fuentes (incluir solo los glifos usados), slicing de ejes variable fonts, conversión a WOFF/WOFF2, crawling de sitios para extraer glifos necesarios |
| **Uso en el orquestador** | Optimizar fuentes para web: reducir peso de 200KB a 5-15KB subsetting. Crawlear la página para detectar qué caracteres se usan realmente |

### 7.2 Subfont

| Campo | Detalle |
|:------|:--------|
| **Tipo** | CLI Node.js (Open Source) |
| **Instalación** | `npm install -g subfont` |
| **Costo** | 100% gratis |
| **Qué permite** | Analizar estáticamente una página, detectar qué caracteres usa cada fuente, generar subsets óptimos, inyectar preload hints |
| **Diferenciador** | Automático: corre sobre el build y optimiza todo sin intervención manual |

### 7.3 FontTools

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería Python (Open Source) |
| **Instalación** | `pip install fonttools` |
| **Costo** | 100% gratis |
| **Qué permite** | Manipulación programática de fuentes: merging, renaming, subsetting, conversión, extracción de metadatos |
| **Uso en el orquestador** | Análisis de fuentes existentes, preparación para web, merging de weights en variable fonts |

### Trigger words
"glypher", "subfont", "fonttools", "glyphhanger", "subsetting de fuentes", "optimizar fuentes", "font subsetting", "WOFF2", "variable fonts", "cargar fuentes", "preload fonts"

---

## 8. SVG & Icon Tools

### 8.1 SVGO

| Campo | Detalle |
|:------|:--------|
| **Tipo** | CLI + Librería Node.js (Open Source) |
| **Repo** | `svg/svgo` |
| **Instalación** | `npm install -g svgo` |
| **Costo** | 100% gratis |
| **Qué permite** | Optimizar SVG: eliminar metadatos innecesarios, limpiar paths, reducir precisión decimal, remover atributos redundantes |
| **Uso en el orquestador** | Optimizar iconos y gráficos SVG para producción. Reduce tamaño típico en 30-50% |

### 8.2 Lucide

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería de iconos (Open Source) |
| **Repo** | `lucide-icons/lucide` |
| **Instalación** | `npm install lucide-react` o CDN |
| **Costo** | 100% gratis. MIT license |
| **Qué permite** | 1,500+ iconos SVG consistentes (mismo stroke width, corner radius, design language). Paquetes para React, Vue, Svelte, Figma |
| **Uso en el orquestador** | Sistema de iconos para UI sin necesidad de diseñar cada icono. Consistencia garantizada |

### 8.3 Phosphor Icons

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería de iconos (Open Source) |
| **Web** | `phosphoricons.com` |
| **Instalación** | `npm install phosphor-react` |
| **Costo** | 100% gratis. MIT license |
| **Qué permite** | 6 variantes de peso (thin, light, regular, bold, fill, duotone) para cada icono. Mayor flexibilidad visual que Lucide |
| **Cuándo usarlo** | Cuando el diseño necesita variación de peso en iconos (ej: bold para activo, regular para inactivo) |

### Trigger words
"svgo", "lucide", "phosphor", "iconos", "optimizar SVG", "SVG icons", "icon library", "sistema de iconos", "lucide react", "phosphor icons"

---

## 9. CSS & Performance Optimization

### 9.1 Stylelint

| Campo | Detalle |
|:------|:--------|
| **Tipo** | CLI + Librería Node.js (Open Source) |
| **Instalación** | `npm install -g stylelint stylelint-config-standard` |
| **Costo** | 100% gratis |
| **Qué permite** | Linting de CSS: detectar propiedades deprecadas, errores de sintaxis, inconsistencias de naming, problemas de orden |
| **Uso en el orquestador** | Auditar CSS generado por la IA. Asegurar consistencia con las reglas del proyecto |

### 9.2 PurgeCSS

| Campo | Detalle |
|:------|:--------|
| **Tipo** | CLI + Librería Node.js (Open Source) |
| **Repo** | `FullHuman/purgecss` |
| **Instalación** | `npm install purgecss` |
| **Costo** | 100% gratis |
| **Qué permite** | Eliminar CSS no utilizado en producción. Análisis estático del HTML/JSX para detectar qué clases se usan realmente |
| **Uso en el orquestador** | Reducir CSS bundle hasta 90% en proyectos con frameworks CSS (Tailwind, Bootstrap) |

### 9.3 Critters

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Plugin Webpack/Vite (Open Source) |
| **Repo** | `GoogleChromeLabs/critters` |
| **Instalación** | `npm install critters` |
| **Costo** | 100% gratis |
| **Qué permite** | Inline CSS crítico y lazy-load del resto. Mejora First Contentful Paint (FCP) |
| **Uso en el orquestador** | Optimizar Lighthouse Performance Score. Inline del CSS above-the-fold automático |

### 9.4 Lighthouse CI

| Campo | Detalle |
|:------|:--------|
| **Tipo** | CLI Node.js (Open Source) |
| **Repo** | `GoogleChrome/lighthouse-ci` |
| **Instalación** | `npm install -g @lhci/cli` |
| **Costo** | 100% gratis |
| **Qué permite** | Auditoría automatizada de performance, accesibilidad, SEO, best practices. Con gate de calidad (fail si score < umbral) |
| **Uso en el orquestador** | Gate de calidad CI/CD. No dejar pasar diseño que no cumpla mínimo 90 en Performance, Accessibility, SEO |

### Trigger words
"stylelint", "purgecss", "critters", "lighthouse", "optimizar CSS", "lint CSS", "CSS performance", "critical CSS", "Lighthouse CI", "performance audit", "CSS bundle", "eliminar CSS no usado"

---

## 10. Accessibility Testing

### 10.1 axe-core

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería JS + CLI (Open Source) |
| **Repo** | `dequelabs/axe-core` |
| **Instalación** | `npm install axe-core` o extensión navegador |
| **Costo** | 100% gratis. Open Source MPL-2.0 |
| **Qué permite** | Auditoría automática de accesibilidad WCAG 2.2. Detecta violaciones y proporciona sugerencias de reparación |
| **Uso en el orquestador** | Gate de accesibilidad obligatorio antes de lanzar cualquier interfaz |

### 10.2 Pa11y

| Campo | Detalle |
|:------|:--------|
| **Tipo** | CLI Node.js (Open Source) |
| **Repo** | `pa11y/pa11y` |
| **Instalación** | `npm install -g pa11y` |
| **Costo** | 100% gratis |
| **Qué permite** | Auditoría de accesibilidad desde línea de comandos. Integrable en CI/CD. Soporta múltiples estándares (WCAG 2.0, 2.1, 2.2, Section 508) |
| **Uso en el orquestador** | Automatizar auditoría de accesibilidad en cada build. Generar reporte |

### Trigger words
"axe-core", "pa11y", "accesibilidad", "WCAG", "a11y audit", "auditoría accesibilidad", "Section 508", "accesible"

---

## 11. Component & Design Tools

### 11.1 Storybook

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Framework (Open Source) |
| **Web** | `storybook.js.org` |
| **Instalación** | `npx storybook@latest init` |
| **Costo** | 100% gratis |
| **Qué permite** | Catálogo interactivo de componentes UI con variantes, estados, y documentación. Visual Regression Testing con Chromatic |
| **Uso en el orquestador** | Documentar componentes diseñados. Verificar visual regression. Compartir con el equipo |

### 11.2 shadcn/ui

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Colección de componentes (Open Source) |
| **Web** | `ui.shadcn.com` |
| **Instalación** | `npx shadcn@latest init` |
| **Costo** | 100% gratis |
| **Qué permite** | Componentes React accesibles y customizables. No es una dependencia — son archivos que copias a tu proyecto y modificas |
| **Uso en el orquestador** | Base de componentes para proyectos React. Customizar con tokens del design system |

### 11.3 Radix UI

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería de primitives (Open Source) |
| **Web** | `radix-ui.com` |
| **Instalación** | `npm install @radix-ui/react-dialog` |
| **Costo** | 100% gratis. MIT license |
| **Qué permite** | Primitivas UI headless (Dialog, Popover, Dropdown Menu, Tooltip, Tabs, Accordion, etc.) con accesibilidad WCAG incorporada |
| **Uso en el orquestador** | Base de interacciones complejas sin reinventar accesibilidad. Combinar con shadcn/ui o custom styling |

### 11.4 Argos Visual Testing

| Campo | Detalle |
|:------|:--------|
| **Tipo** | CI/CD Visual Regression (Open Source) |
| **Repo** | `argos-ci/argos` |
| **Costo** | Gratis para proyectos open-source |
| **Qué permite** | Comparación visual entre builds. Detectar cambios no intencionales en UI |

### Trigger words
"storybook", "shadcn/ui", "radix ui", "argos", "visual regression", "componentes", "primitives UI", "catálogo de componentes", "testing visual"

---

## 12. 3D & Animation Tools

### 12.1 Three.js Examples Explorer

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Repositorio de ejemplos (Open Source) |
| **Web** | `threejs.org/examples` |
| **Costo** | 100% gratis |
| **Qué permite** | 700+ ejemplos interactivos de Three.js: partículas, shaders, física, post-processing, VR, AR |
| **Uso en el orquestador** | Referencia rápida para implementar efectos 3D. Buscar ejemplo similar a lo que necesita el diseño |

### 12.2 Lottie Files (dotLottie)

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Formato de animación (Open Source) |
| **Web** | `lottiefiles.com` |
| **Costo** | 100% gratis para animaciones open-source |
| **Qué permite** | Animaciones vectoriales ligeras (JSON) reproducibles en web, iOS, Android. DotLottie = formato contenedor optimizado |
| **Uso en el orquestador** | Animaciones interactivas sin depender de librerías JS pesadas. Iconos animados, loading states, micro-interacciones |

### 12.3 Rive

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Editor + Runtime (Free Tier) |
| **Web** | `rive.app` |
| **Costo** | Gratis para proyectos públicos (1 archivo). Runtime open-source |
| **Qué permite** | Animaciones state-machine driven. Un solo archivo con múltiples estados (idle, hover, active, disabled) y transiciones |
| **Uso en el orquestador** | Animaciones de UI con lógica condicional. Mejor que Lottie para interacciones complejas |

### Trigger words
"three.js", "lottie", "rive", "dotLottie", "animación 3D", "animación vectorial", "state machine animación", "lottie files", "animación interactiva", "threejs examples", "WebGL"

---

## 13. Color Science

### 13.1 chroma.js

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería JS (Open Source) |
| **Instalación** | `npm install chroma-js` |
| **Costo** | 100% gratis |
| **Qué permite** | Manipulación de color: conversión entre espacios (RGB, HSL, OKLCH, LAB, LCH), interpolación, escalas de color, contraste WCAG, paletas |
| **Uso en el orquestador** | Generar paletas programáticamente, verificar contraste WCAG, interpolar entre colores, convertir a cualquier espacio |

### 13.2 culori

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería JS (Open Source) |
| **Instalación** | `npm install culori` |
| **Costo** | 100% gratis |
| **Qué permite** | Similar a chroma.js pero más liviano (10KB vs 80KB). Soporta espacios modernos: OKLCH, JCh, ICtCp |
| **Cuándo usarlo** | Cuando el bundle size importa. OKLCH nativo sin polyfills |

### 13.3 color-name-list

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Dataset (Open Source) |
| **Repo** | `meodai/color-names` |
| **Costo** | 100% gratis |
| **Qué permite** | 5,000+ nombres de color con valores HEX. Ideal para generar naming de colores en design systems |
| **Uso en el orquestador** | Asignar nombres descriptivos a colores en el design system |

### Trigger words
"chroma.js", "culori", "color name list", "manipulación de color", "color science", "OKLCH", "paleta programática", "interpolación de color", "contraste WCAG", "color conversion"

---

## 14. Utilidades Generales

### 14.1 tinycolor2

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Librería JS (Open Source) |
| **Instalación** | `npm install tinycolor2` |
| **Costo** | 100% gratis |
| **Qué permite** | Parsing y manipulación ligera de color. Ideal para operaciones rápidas: `lighten`, `darken`, `isLight`, `toHex`, `toRgb` |
| **Uso en el orquestador** | Ajustes rápidos de color en runtime. No recomendado para ciencia del color compleja |

### 14.2 Prettier + ESLint

| Campo | Detalle |
|:------|:--------|
| **Tipo** | Formatters/Linters (Open Source) |
| **Instalación** | `npm install -D prettier eslint` |
| **Costo** | 100% gratis |
| **Qué permite** | Formateo consistente de código (Prettier) + calidad estática (ESLint). Configuración compartida por proyecto |
| **Uso en el orquestador** | Asegurar que todo código generado por IA pase lint antes de commit. Consistencia en todo el codebase |

### Trigger words
"tinycolor", "prettier", "eslint", "formatear código", "lint", "calidad de código", "código limpio"

---

## Integración con el Orquestador

**Trigger words:** "herramientas gratuitas", "open source tools", "tool registry", "Playwright", "Puppeteer", "SearXNG", "Tavily", "Figma API", "GitHub API", "Satori", "Sharp", "Glypher", "SVGO", "Stylelint", "PurgeCSS", "Lighthouse", "axe-core", "Storybook", "shadcn/ui", "Radix UI", "Three.js", "Lottie", "chroma.js", "culori", "crowler", "scraper", "optimizar imágenes", "optimizar fuentes", "auditar accesibilidad", "extraer diseño de página", "inspeccionar sitio web", "buscar en github"

**Flujo de integración:**

```
Para cada proyecto, el orquestador puede:
  1. Buscar referencias visuales → Playwright (screenshot) + DuckDuckGo Search
  2. Extraer sistemas de diseño → Figma API + GitHub API
  3. Analizar competencia → Playwright (extraer CSS) + Trafilatura (contenido)
  4. Optimizar assets → Glypher (fuentes) + Sharp (imágenes) + SVGO (SVG)
  5. Auditar calidad → axe-core (a11y) + Lighthouse (perf) + Stylelint (CSS)
  6. Generar OG/social cards → Satori (SVG) + Sharp (PNG)
  7. Documentar componentes → Storybook + shadcn/ui
  8. Validar accesibilidad → Pa11y + axe-core
```

**Herramientas por contexto de uso:**

| Contexto | Herramientas recomendadas | Alternativa auto-alojada |
|:---------|:-------------------------|:------------------------|
| Inspeccionar sitio web | Playwright (screenshot + CSS) | Puppeteer Core + Browserless |
| Buscar referencias | DuckDuckGo Search | SearXNG (Docker) |
| Extraer contenido | Trafilatura (Markdown) | Mozilla Readability (JS) |
| Generar imagen OG | Satori → Sharp | Pillow (Python) |
| Subsetting fuentes | Glypher (CLI) | FontTools (Python) |
| Optimizar SVG | SVGO (CLI) | — |
| Auditar accesibilidad | axe-core (JS) | Pa11y (CLI) |
| Linting CSS | Stylelint | — |
| Iconos UI | Lucide (React) | Phosphor (React) |
| Color science | chroma.js (JS) | culori (JS, +liviano) |
| Búsqueda código | GitHub API | — |
| Componentes base | shadcn/ui + Radix UI | — |
