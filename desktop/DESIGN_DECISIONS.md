# Vanta Studio — Design Decisions

> Decisions binding for anyone touching `desktop/` styling or theming.
> Registered 2026-08-23 (FIND-26, owner-decided). Update this file, not the tokens.

## 1. Tokens are intentionally decoupled from web/

The desktop app and the marketing site (`web/`) **do not share a design-token
package**. They evolved different identities on purpose:

| | `desktop/` (Vanta Studio) | `web/` (marketing) |
|---|---|---|
| Purpose | Data-dense product UI | Editorial landing/site |
| Theme | Light + Dark (WCAG AA audited) | **Light-only by design** |
| Palette source of truth | `src/index.css` (`:root` + `.dark`) | `src/app/globals.css` |

**Rule:** never copy token values from `web/src/app/globals.css` into desktop
(or vice versa) without an explicit recorded decision. Similar hex values
(`#FBF9F5`, `#FF5500`, `#000000`) are brand constants shared *conceptually*;
every other token (borders, muted ramps, chart colors) may diverge.

## 2. Canonical palette = post-audit `desktop/src/index.css`

After FIND-24 the manga/linocut palette is WCAG AA calibrated. The contrast
audit table at the bottom of `index.css` is normative: any new color pair must
pass ≥4.5:1 (text) / ≥3:1 (non-text) before merging.

- Brand orange `#FF5500` stays as accent **background/border**; its text
  foreground is `#000000` (6.55:1), never cream.
- Destructive is a distinct red (`#C41E25` light / `#F26D6D` dark), never the
  accent orange.

## 3. Typography is shared (exception to §1)

Both apps use the same four families (Geist, Geist Mono, Anton, Space Mono)
via `@fontsource` woff2 files. Sharing fonts is fine; sharing *tokens* is not.

## 4. Theming behavior (FIND-22)

Startup preference order: stored manual choice (`vanta-theme`) → OS
(`prefers-color-scheme`). While no manual choice exists, live OS changes
propagate. A manual toggle persists and stops OS-following.
