# Identidad de Marca — VantaDB Web

> **Estilo:** Manga / Linocut / Neo-Brutalist
> **Fecha:** 2026-07-27
> **Fuente:** Extraído del código actual en `web/`

---

## 1. Resumen de Identidad

VantaDB no es una base de datos más. Es un motor embebido escrito en Rust para memoria duradera y recuperación híbrida de vectores. La marca web refleja esa personalidad: **precisa, anti-cloud, analógica**.

La identidad visual se inspira en tres corrientes:

- **Manga** — alto contraste, línea negra marcada, iconografía de gato, viñetas, energía cruda
- **Linocut** — textura de papel (`cream`), trazo grueso (`ink`), acabado artesanal
- **Neo-Brutalist** — jerarquía tipográfica extrema, colores planos, bordes duros, sin gradientes

La paleta es deliberadamente reducida: papel crema, tinta negra, un solo acento neón naranja que funciona como marcador fluorescente sobre el fondo crudo. Esta economía (3 colores funcionales + 2 tonos de apoyo) fuerza decisiones de diseño claras.

---

## 2. Paleta de Colores

| Token | Hex | Variable CSS | Rol |
|-------|-----|-------------|-----|
| **Cream** | `#FBF9F5` | `--color-cream` | Fondo principal, tarjetas, superficie — el "papel" del lienzo físico |
| **Ink** | `#000000` | `--color-ink` | Texto body, headings, bordes, íconos, separadores — la "tinta" |
| **Neon** | `#FF5500` | `--color-neon` | Acento único, CTAs, hover states, selección, badges, highlights, "DB" en el logo |
| **Paper** | `#F2EDE2` | `--color-paper` | Secciones alternas, fondos de tarjetas secundarias — ligeramente más oscuro que cream |
| **Smoke** | `#1A1A1A` | `--color-smoke` | Fondos oscuros (código, pre), hover de elementos ink, overlays |

### Reglas de paleta

- **Cream es el fondo por defecto.** Toda página nueva debe usar `bg-cream` como base y `bg-paper` para secciones alternas.
- **Neon es el único acento.** No introducir un segundo color de acento sin aprobación explícita. Si se necesita variación, usar opacidad de neon (ej. `bg-neon/10`).
- **Texto siempre ink sobre cream.** Sobre smoke, el texto debe ser cream o neon.
- **Bordes:** color ink por defecto, grosor 1-2px. No sombras suaves — si hay sombra, es dura y negra.

---

## 3. Tipografía

| Font | Variable CSS | Tailwind Class | Uso |
|------|-------------|---------------|-----|
| **Geist** | `--font-geist-sans` | `font-sans` | Texto body, UI general, navegación, párrafos |
| **Geist Mono** | `--font-geist-mono` | `font-mono` | Código inline, bloques de código, datos numéricos, métricas |
| **Anton** | `--font-anton` | `font-display` | Headings, hero title, logo wordmark, títulos de sección — **siempre uppercase** |
| **Space Mono** | `--font-space-mono` | `font-tech` | Badges, tags, metadata, labels técnicos, fechas, captions — **siempre uppercase + tracking-wider** |

### Reglas tipográficas

- **Anton solo en uppercase.** Es una display face sin minúsculas funcionales. Usar para el hero heading (`text-5xl` o mayor), títulos de landing y el wordmark "Vanta".
- **Space Mono para metadatos.** Badges como "NEW", "BETA", "v0.1.7", tags de features, labels de sección, timestamps. Siempre `font-tech uppercase tracking-wider text-xs`.
- **Geist para todo lo demás.** Es la voz neutral de la interfaz. El body text debe mantenerse en `text-sm` o `text-base`.
- **Geist Mono para precisión técnica.** Números de benchmark ("1.2ms", "100% Recall@10"), código inline, comandos CLI.
- **Jerarquía heading:** Anton display → Space Mono metadata → Geist body.

---

## 4. Logo y Mascota

### VantaLogoMark (SVG inline)

El logo mark es un SVG de 64×64 viewBox:

- Círculo exterior negro (`r=28`), centrado en (32, 32)
- Esfera interior naranja neon (`r=14`), centrada en (32, 32)
- Dos ojos: rectángulos negros de 2.5×10 px, ubicados simétricamente

**Lectura visual:** Cara de gato minimalista, estilo manga, con ojos de neón. La esfera interior es un ojo central tipo cíclope felino.

**Uso en navbar:**
```
<VantaLogoMark size={36} /> + "Vanta" + "DB" (neon)
```

**Uso en footer:**
```
<VantaLogoMark size={48} />
```

### Vanta Cat — Mascota

- Archivos: `/assets/mascota_gato.png`, `/assets/avatar_gato.png`
- Descripción visual: "The shadow cat with fire eyes"
- Aparece en: hero section, navbar (easter egg), Team page
- **Rol:** Mascot & Morale Officer
- La mascota refuerza la identidad manga y da calidez a un producto técnico
- No forzar su inclusión en páginas donde no encaje — es un acento, no un requisito

---

## 5. Taglines y Copy

### Taglines principales

| Contexto | Texto |
|----------|-------|
| Hero | "Embedded Rust engine for durable local memory and hybrid vector retrieval." |
| Hero subhead | "Persistent memory, crash-safe WAL recovery, and native hybrid search — without external services, containers, or network dependencies." |
| Footer | "forged in rust · printed on cream stock" |
| Meta / SEO | "VantaDB is a local-first, embedded Rust database engine for AI agents and local RAG." |

### Mission

> Make local-first hybrid retrieval the default for AI agents, RAG pipelines, and edge applications. No cloud tax on your own data.

### Company Principles

1. **Local-first, always** — "The fastest network hop is no network hop."
2. **Embedded, not attached** — "A database that runs in your process is simpler than one you have to deploy."
3. **Crash-safe by construction** — "Durability is not a feature you add later."
4. **Open source, no telemetry** — "Apache 2.0, no phone-home, no analytics."

---

## 6. Tono de Voz — 5 Dimensiones

### 1. Preciso técnicamente

Cada claim incluye evidencia numérica. No se hacen afirmaciones vagas.

> **Bien:** "1.2ms per dimension, 100% Recall@10"  
> **Mal:** "muy rápido y preciso"

### 2. Anti-establishment

El producto se posiciona contra la complejidad innecesaria del cloud. Vocabulario: "zero daemons", "no Kubernetes", "no cloud tax".

### 3. Honesto

La sección `Product Boundary` del hero explicita qué NO está construido aún. La marca no promete lo que no puede cumplir.

> "No cloud sync, no replication, no RBAC yet."

### 4. Comunitario

Apache 2.0 es un feature, no una nota legal. Invitación explícita a forkear, auditar y contribuir.

> "Apache 2.0, no strings attached. Fork it, audit it, ship it."

### 5. Manga / analógico

El léxico usa metáforas de imprenta y papel: "forged in rust", "printed on cream stock", "ink", "paper". Esto conecta el mundo digital con el artesanal.

### Nota de idioma

El sitio está configurado con `<html lang="es">`. La UI principal está en español. Esto debe mantenerse: el copy técnico puede tener términos en inglés (RAG, WAL, embeddings) pero la navegación, botones y labels deben estar en español.

---

## 7. Target Audience (del contenido)

| Segmento | Señales en el código |
|----------|---------------------|
| **AI agent developers** | Ejemplos de integración con OpenAI, LangChain, LlamaIndex en la SDK |
| **RAG pipeline builders** | Benchmarks de recuperación híbrida, sparse-dense search |
| **Edge / IoT engineers** | "Embedded", "no containers", benchmarks de memoria y CPU |
| **Indie developers** | "No cloud tax", "Apache 2.0", single binary, sin dependencias externas |
| **Privacy-conscious teams** | "No telemetry", "no phone-home", local-first |

---

## 8. Reglas de Uso

1. **Nuevas páginas** deben seguir esta paleta — no agregar colores sin justificación.
2. **Neon es exclusivo** para interacciones y acentos. No usar neon para texto body ni fondos grandes.
3. **Anton es solo para display** — no usar Anton en body text, botones pequeños ni captions.
4. **Space Mono para metadatos** — badges, tags, versiones, fechas. No usar para párrafos.
5. **VantaLogoMark** siempre incluir el SVG inline (no rasterizado). Mantener relación "Vanta" + "DB" (neon).
6. **Footer siempre termina** con "forged in rust · printed on cream stock".
7. **Idioma español por defecto.** Si se agrega i18n, español es la fuente de verdad.
8. **Vanta Cat es opcional** en nuevas páginas. Siempre en hero y team. Decisión por página.
9. **No sombras suaves** — sombras duras negras o sin sombra.
10. **Bordes ink** — usar `border-ink` o `border-2 border-ink`. Sin bordes redondeados excesivos.