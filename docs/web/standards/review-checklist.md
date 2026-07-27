# Pre-Merge Review Checklist

## Code

- [ ] `"use client"` directive present as first line of every component/page file
- [ ] No hardcoded user-facing strings — all text via `t()` or `tt()` with fallback
- [ ] Dictionary keys added to **both** `es` and `en` in `src/lib/dictionaries.ts`
- [ ] Props typed with TypeScript interface (`interface XxxProps`)
- [ ] Component handles all states: loading, empty, error, and relevant edge cases
- [ ] Animation cleanup on unmount — no orphaned `loop: true` anime.js instances. MarkClassic's ambient pulse (lines 80–93) stores no ref to cancel. New loops must follow useMarkInteraction's pattern (ref + `.pause()`)
- [ ] No `eslint-disable` comments added without inline justification

## Quality

- [ ] `npm run build` succeeds (the config has `ignoreBuildErrors: true`, which means TypeScript errors won't break the build — **this is not an excuse**; run `npx tsc --noEmit` to catch type errors)
- [ ] No duplicate `tt()` helpers — use the existing pattern or extract to `src/lib/`
- [ ] No new packages added unless required by the feature. Verify with `npm why <pkg>`
- [ ] `prefers-reduced-motion` respected for all new animations — both CSS and JS
- [ ] No imports from `web_old/` or any inactive code path

## Architecture

- [ ] New page route follows **Pattern B** (thin `page.tsx` importing view component from `vanta/`) unless specialized behavior is justified
- [ ] No new page-level `layout.tsx` unless needed for route-specific `<head>` metadata
- [ ] Component placed in correct directory: feature components in `vanta/`, shadcn/ui primitives in `ui/`, hooks in `hooks/`, utilities in `lib/`
- [ ] New feature component is a single file — no multi-file components without justification

## i18n Specific

- [ ] New dictionary keys use single braces `{param}` not double `{{param}}`
- [ ] If route affects SEO, `<html lang="...">` consideration is documented (root is `lang="es"`)
- [ ] If adding new persistence (e.g. localStorage), follow existing key naming patterns

## Dead Code

- [ ] No imports from the 5 dead components: `navbar.tsx`, `hero-mark-interactive.tsx`, `ecosystem.tsx`, `vs-table.tsx`, `metrics-bar.tsx`
- [ ] No hardcoded data that should live in `vanta-data.ts` — if a value appears in multiple places, add it to the central store

## Exceptions

Any item skipped must have a `ponytail:` comment in the diff explaining the shortcut and the ceiling at which it should be addressed.
