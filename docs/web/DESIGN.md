# Design System — VantaDB Web

**Aesthetic:** Manga / Linocut / Neo-Brutalist — "ink on paper, neon accents"

## Palette

| Token | Value | CSS Variable | Usage Rule |
|---|---|---|---|
| `cream` | `#FBF9F5` | `--color-cream` | Page background, card surfaces, `bg-background` |
| `ink` | `#000000` | `--color-ink` | Text (100%), borders, hard shadows |
| `neon` | `#FF5500` | `--color-neon` | Accent only — CTAs, highlights, markers (95/5 rule) |
| `paper` | `#F2EDE2` | `--color-paper` | Secondary surfaces, muted sections, `--secondary` |
| `smoke` | `#1A1A1A` | `--color-smoke` | Near-black elements, `--muted-foreground` |

**Rule:** 95% cream/ink, 5% neon. Accent only for active states, CTAs, and critical signals.

## Typography

| CSS Class | Font | Variable | Weight | Used For |
|---|---|---|---|---|
| `font-sans` | Geist Sans | `--font-geist-sans` | 400–700 | Body, navigation, general UI |
| `font-mono` | Geist Mono | `--font-geist-mono` | 400–700 | Code blocks, technical data |
| `font-display` | Anton | `--font-anton` | 400 | Headlines, hero titles, giant type |
| `font-tech` | Space Mono | `--font-space-mono` | 400, 700 | Tech labels, stencil text, stats |

### Text Treatments

- `.text-stencil` — tight letter-spacing, line-height 0.9 (ink stamp feel)
- `.text-outline` — `-webkit-text-stroke: 2px #000`, transparent fill (hollow title)
- `.text-outline-neon` — same with `#FF5500` stroke
- `.marker-neon` — underline highlight via gradient (58%–92% yellow-marker style)

## Shadows

Every shadow is **rigid** — no blur, no spread. Physical-paper offset shadows:

| Class | Shadow |
|---|---|
| `shadow-brutal-sm` | `4px 4px 0 0 #000` |
| `.shadow-brutal` | `6px 6px 0 0 #000` |
| `.shadow-brutal-lg` | `8px 8px 0 0 #000` |
| `.shadow-brutal-neon` | `6px 6px 0 0 #FF5500, 6px 6px 0 2px #000` |
| `.shadow-throw` | `6px 6px 0 0 #000, 12px 12px 0 0 rgba(0,0,0,0.15)` |

## Press Effects

Interactive elements get physical-press animation (transform + shadow reduction):

| Class | Hover | Active |
|---|---|---|
| `.press` | `translate(2,2)` shadow→4px | `translate(6,6)` shadow→0 |
| `.press-lg` | `translate(3,3)` shadow→5px | `translate(8,8)` shadow→0 |
| `.press-neon` | `translate(2,2)` + neon bg | `translate(6,6)` shadow→0 |

## Texture & Pattern Effects

| Class | Description |
|---|---|
| `.paper-bg` | Subtle dot-grid noise on cream (`rgba(0,0,0,0.035)` dots, 22px grid) |
| `.paper-grain` | CSS filter noise overlay (SVG feTurbulence, 6% opacity) |
| `.halftone` | Black halftone dots (1.4px, 12px grid) |
| `.halftone-neon` | Neon-orange halftone dots |
| `.halftone-fade` | Masked fade (bottom-right gradient) |
| `.hatch` | 45° diagonal lines (1px black, 6px spacing) |
| `.hatch-neon` | 45° diagonal lines in neon |
| `.speed-lines` | Vertical stripes (8px gap, manga action) |
| `.speed-lines-radial` | Conic speed lines from center |
| `.grid-tech` | Crosshatch tech grid (28px cells, 6% opacity) |
| `.scanlines` | Horizontal CRT scan lines (2px interval, 3% opacity) |
| `.stripe-accent` | Repeating diagonal neon stripes |
| `.ink-divider` | Section divider — horizontal line + `◆` diamond marker |

## Glow & Light Effects

| Class | Description |
|---|---|
| `.glow-neon` | `text-shadow` with neon glow (8px/16px spread) |
| `.glow-box-neon` | Box glow with pulse animation |
| `.btn-neon-glow` | Button with neon border + light sweep animation |
| `.glitch-hover` | Chromatic offset on hover (red + cyan split) |
| `.accent-bar-top` | 3px dashed neon-accent top border |
| `.neon-underline` | Animated underline (0→100% width on hover) |

## Decorative Elements

| Class | Description |
|---|---|
| `.manga-frame` | Clip-path polygon with corner cuts |
| `.tape` | Washi-tape effect (neon stripe, rotated, dashed border) |
| `.ink-drip` | Bottom-ink radial drips (8% opacity) |
| `.ink-corner` | Top-right ink splatter dot |
| `.animated-gradient-border` | Animated neon/ink gradient bar |
| `.vanta-slider` | Brutalist range slider with neon thumb |

## Borders & Radius

- `border-radius: 0.125rem` (`--radius`) — nearly sharp, minimal rounding
- All borders default to `border-border` → `#000000`
- `border-4` common on cards, buttons, containers

## Animation (CSS)

| Class | Keyframes | Duration | Purpose |
|---|---|---|---|
| `.animate-marquee` | `vanta-marquee` | 28s linear infinite | Scrolling tech strip |
| `.animate-blink` | `vanta-blink` | 1.1s step-end infinite | Cursor blink |
| `.animate-flicker` | `vanta-flicker` | 4.5s linear infinite | Neon flicker |
| `.animate-shake` | `vanta-shake` | 0.18s ease-in-out infinite | Shake effect |
| `.animate-rise` | `vanta-rise` | 0.5s | Page entrance slide-up |
| `.animate-stamp` | `vanta-stamp` | 0.5s | Stamp/slam entrance |
| `.animate-scan` | `vanta-scan` | 3.2s linear infinite | Scanning line |
| `.animate-pulse-ring` | `vanta-pulse-ring` | 2s ease-out infinite | Ping/pulse ring |
| `.animate-float` | `vanta-float` | 3s ease-in-out infinite | Floating label |
| `.stagger-children` | `vanta-rise` per child | 0.4s × delay | Staggered entrance |

## Accessibility

- `:focus-visible` → 3px neon outline (`#FF5500`), 2px offset
- `prefers-reduced-motion: reduce` → all animations to 0.01ms
- `.skip-link` → visually hidden, shows on focus (neon, brute style)
- Scanlines/halftone/paper-grain use `pointer-events: none` (decorative only)

## Anti-Patterns (AUDIT.md Highlights)

| Issue | Detail |
|---|---|
| Dark mode broken | `next-themes` installed, `ThemeToggle` in dead navbar, no dark CSS vars in `:root.dark` |
| `press-neon` no hover | Only `:active` defined, no `:hover` state |
| 50+ blank lines | `globals.css` lines 389–461, 464–517, 556–557 |
| `tailwind.config.ts` inert | v3 syntax in v4 project — no effect |
