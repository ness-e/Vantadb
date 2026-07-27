# Animation System

Three animation layers: CSS keyframe animations, framer-motion (page transitions + reveals), anime.js (interactive SVG mark).

## CSS Animations (globals.css)

13 keyframe animations, all prefixed `vanta-`:

| Class | Keyframe | Duration | Effect |
|---|---|---|---|
| `animate-rise` | `vanta-rise` | 0.5s | Slide-up + fade (entrance) |
| `animate-marquee` | `vanta-marquee` | 28s linear inf | Horizontal scroll (navbar strip) |
| `animate-stamp` | `vanta-stamp` | 0.5s | Scale + rotate -12deg (ink stamp) |
| `animate-flicker` | `vanta-flicker` | 4.5s linear inf | Neon flicker |
| `animate-blink` | `vanta-blink` | 1.1s step-end | Binary blink |
| `animate-pulse-ring` | `vanta-pulse-ring` | 2s ease-out | Neon pulse ring |
| `animate-shake` | `vanta-shake` | 0.18s ease-in | Glitch shake |
| `animate-scan` | `vanta-scan` | 3.2s linear inf | Scanning line |
| `animate-float` | `vanta-float` | 3s ease-in-out | Gentle hover bob |

All respect `prefers-reduced-motion: reduce` — durations collapse to 0.01ms.

## Framer Motion

- **PageTransition** — wraps all routes in `<AnimatePresence mode="wait">`. Slide+fade: 280ms, cubic-bezier(0.2, 0.8, 0.2, 1) "manga-ease".
- **Reveal** — scroll-triggered component (6 directions: up/down/left/right/scale/fade). Uses CSS transitions via IntersectionObserver, not framer-motion.

## Anime.js

Used exclusively in the `mark/` components for interactive SVG animation:

| File | Usage | Cleanup |
|---|---|---|
| `mark/use-mark-interaction.ts` | Mouse → eye/sphere offset, blink cycle, squint | `.pause()` on unmount |
| `mark/mark-classic.tsx` | Hover pulse, ambient stagger on mount | ❌ No cleanup — `loop: true` persists after unmount (known bug) |
| `mark/mark-cta.tsx` | Click reactions (pulse, spin, bounce) | Single-shot, no loop |

## Count-up Hook

`useCountUp` in `hooks/count-up.tsx`: IntersectionObserver + rAF + easeOutCubic. Handles complex strings (`"1.2ms"`, `"5,400"`).

## Synthetic Events

- ⌘K → CommandPalette via `window.dispatchEvent(new KeyboardEvent(...))`
- `?` → ShortcutOverlay (same pattern)
- Typing "vanta" → EasterEgg (keydown listener)
