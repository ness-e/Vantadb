# Web Frontend (Next.js 16 + shadcn/ui + framer-motion)

> Movido desde `.opencode/AGENTS.md` — referencia on-demand. Consultar cuando toques `web/`. Si editas, actualiza también el puntero en AGENTS.md.

Stack: **Next.js 16 + React 19 + shadcn/ui (New York) + Tailwind CSS v4 + framer-motion + animejs**

## Estructura

```
web/
  src/
    app/           ← Next.js App Router pages (all "use client")
    components/
      ui/          ← shadcn/ui components (Radix-based)
      vanta/       ← VantaDB-specific components (hero, features, etc.)
    hooks/         ← count-up, focus-trap, reveal, toast, etc.
    lib/           ← dictionaries.ts (i18n ~880 ES/EN keys), utils.ts (cn)
  public/
    assets/        ← images
```

## Stack decisions

| Decisión | Por qué |
|----------|---------|
| **Next.js 16** | App Router, standalone output, `ignoreBuildErrors: true` en next.config.ts |
| **React 19** | Client components everywhere (no RSC) |
| **shadcn/ui** | New York style, neutral base, lucide icons |
| **framer-motion** | Page transitions via AnimatePresence, scroll reveal |
| **Tailwind v4** | Theme via `@theme inline {}` in globals.css; `tailwind.config.ts` inert |
| **animejs** | Interactive graph animation (mark-classic.tsx) |
| **zustand** | State management (installed, usage TBD) |

## Design System (globals.css + shadcn)

Colores: cream `#FBF9F5`, ink `#000000`, neon `#FF5500`, paper `#F2EDE2`, smoke `#1A1A1A`.
Bordes: `border-4 border-black` con rigid shadow `shadow-[6px_6px_0_0_#000]`.
Efectos: press/press-lg/glow-neon/glitch-hover/scanlines/halftone/speed-lines/grid-tech.

## i18n (custom)

`LanguageProvider` context + `useLanguage()` hook → `{ t, lang, setLang }`. Dictionary ~880 ES/EN keys. `next-intl` installed but unused.

## Animación

- Page transitions: `<PageTransition viewKey={pathname}>` wrapping layout children (framer-motion)
- Scroll reveal: `<Reveal direction="up" delay={n}>` wrapping sections
- Count-up: `<CountUpStat value={N} suffix="vec/s" />`
- Animejs: SVG interactive graph in mark-classic.tsx

## Contenido

- Stack real: **Rust 1.94**+ | **Python 3.11**+ | Fjall + RocksDB + InMemory backends
- Integraciones reales: CrewAI + DSPy + Haystack + Mem0 + OpenAI + Ollama + LiteLLM
- Versión: **0.2.0** (no 0.1.5)
- Embedding providers: OpenAI, Ollama, LiteLLM
