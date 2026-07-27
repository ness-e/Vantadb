# Quality Audit — VantaDB Web Frontend

Source: `web/AUDIT.md` (auto-generated, 24 items). Full report at `web/AUDIT.md`.

## Critical Issues

| # | Issue | File | Impact |
|---|---|---|---|
| 1 | `AGENTS.md` describes old SPA (3 views, no routing) | `AGENTS.md` | Misleading for new devs |
| 2 | `<html lang="es">` hardcoded | `layout.tsx:85` | SEO (Google sees all-Spanish), a11y (wrong accent in EN) |
| 3 | Dark mode broken — no dark CSS vars | `globals.css` | Theme toggle renders broken UI |
| 4 | 404 uses `[...slug]` instead of `notFound()` | `[...slug]/page.tsx` | Routes render without SiteShell |

## Dead Code

| Component | Lines | Status |
|---|---|---|
| `navbar.tsx` | 546 | Replaced by `site-navbar.tsx` |
| `hero-mark-interactive.tsx` | 312 | Replaced by `mark-classic.tsx` |
| `metrics-bar.tsx` | 131 | Removed from HomeView |
| `ecosystem.tsx` | 168 | Removed from HomeView |
| Hero block `{false && (...)}` | 30 | Never renders (dead mascot code) |
| **Total dead lines** | **1,157** | |

## Unused Dependencies (17)

| Package | Size | 
|---|---|
| `@dnd-kit/*` (3 packages) | ~130KB |
| `@mdxeditor/editor` | ~500KB |
| `@tanstack/react-query` | ~100KB |
| `@tanstack/react-table` | ~120KB |
| `@hookform/resolvers` | ~15KB |
| `date-fns` | ~300KB |
| `next-auth` | ~200KB |
| `next-intl` | ~100KB |
| `react-markdown` | ~50KB |
| `react-syntax-highlighter` | ~800KB |
| `sharp` | ~30MB (native) |
| `uuid` | ~10KB |
| `z-ai-web-dev-sdk` | ~50KB |
| `zustand` | ~10KB |
| `zod` | ~50KB |
| `@reactuses/core` | ~50KB |
| **Total waste** | **~150MB+ node_modules** |

## i18n Issues

| # | Issue | Files |
|---|---|---|
| 5 | `tt()` helper triplicated | `hero.tsx`, `cta-final.tsx`, `footer.tsx` |
| 6 | "— coming soon" hardcoded in EN | `site-navbar.tsx`, `navbar.tsx` |
| 7 | "◆ blink" / "◆ click me" hardcoded in EN | `mark-classic.tsx` |
| 8 | Aria-labels hardcoded in ES (skip-link, back-to-top, theme-toggle, command-palette) | Various |
| 9 | `dictionaries.ts` comment says "300+ keys" but has ~880 | `dictionaries.ts` |

## Accessibility Issues

| # | Issue | Files |
|---|---|---|
| 10 | `SfxLabel` decorative without `aria-hidden` | `mark-classic.tsx` |
| 11 | Interactive circles no `aria-label` or `tabIndex` | `mark-classic.tsx` |
| 12 | Animated marquee no `aria-live` or `role="presentation"` | `site-navbar.tsx` |

## Animation Issues

| # | Issue | Detail |
|---|---|---|
| 13 | anime.js no cleanup on unmount | `mark-classic.tsx` — `useEffect` doesn't call `.kill()` |

## Build/Config Issues

| # | Issue | Detail |
|---|---|---|
| 14 | Turbopack root picks wrong lockfile | `turbopack.root` not set, uses `~` lockfile |
| 15 | `metadataBase` not defined | OG images use `localhost:3000` |
| 16 | `tailwind.config.ts` inert (v3 syntax in v4) | No effect on build |
| 17 | `press-neon` no `:hover` state | Missing hover definition |

## To Fix (Priority Order)

1. `<html lang={lang}>` — make dynamic from LanguageProvider
2. Add dark mode CSS variables (`:root.dark` in globals.css)
3. Replace `[...slug]` catch-all with `not-found.tsx` + `notFound()`
4. Extract duplicate `tt()` helper to `src/lib/i18n-utils.ts`
5. Kill all unused dependencies from `package.json`
6. Delete 4 dead component files
7. Add anime.js cleanup in `mark-classic.tsx`
8. Update `AGENTS.md` to reflect App Router reality
9. Set `metadataBase` in `next.config.ts`
10. i18n hardcoded strings (coming soon, aria-labels, interactive labels)

## Per-File Issue Density

| File | Issues | Lines |
|---|---|---|
| `globals.css` | 4 | 817 |
| `mark-classic.tsx` | 4 | 274 |
| `AGENTS.md` | 1 | — |
| `layout.tsx` | 3 | 111 |
| `site-navbar.tsx` | 2 | 460 |
| `hero.tsx` | 3 | 317 |
| `vanta-data.ts` | 1 | 1,109 |
| `dictionaries.ts` | 1 | 2,830 |
