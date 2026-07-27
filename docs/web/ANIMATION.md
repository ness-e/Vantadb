# Animation — VantaDB Web Frontend

Three animation layers: CSS animations (utility classes), framer-motion (page transitions + reveals), anime.js (interactive SVG).

## CSS Animations

Defined in `globals.css` as `@keyframes` + utility classes. See [DESIGN.md](./DESIGN.md) for full listing.

| Animation | Used On | Purpose |
|---|---|---|
| `.animate-rise` | Every `page.tsx` root div | Page entrance — slide-up + fade (0.5s) |
| `.animate-marquee` | Navbar top strip | Scrolling tech strings (28s loop) |
| `.animate-stamp` | `mark-classic.tsx` | Stamp entrance (scale + rotate) |
| `.stagger-children` | Feature cards | Staggered entrance (6 children, 80ms interval) |
| `.animate-flicker` | Neon elements | Flickering neon effect |
| `.animate-blink` | Cursor | Terminal cursor blink |
| `.animate-pulse-ring` | CTA buttons | Neon pulse ring |
| `.animate-shake` | Hover states | Glitch shake |
| `.animate-scan` | Scanlines | Scanning line overlay |

### Reduced Motion

All animations respect `prefers-reduced-motion: reduce` — durations collapse to 0.01ms.

## Framer Motion

| Component | File | Usage |
|---|---|---|
| `PageTransition` | `page-transition.tsx` | Wraps all route children in `<AnimatePresence mode="wait">` |
| `Reveal` | `reveal.tsx` | Scroll-triggered reveal (6 directions: up/down/left/right/scale/fade) |

### PageTransition

```tsx
<AnimatePresence mode="wait">
  <motion.div
    key={pathname}
    initial={{ opacity: 0, y: 12 }}
    animate={{ opacity: 1, y: 0 }}
    exit={{ opacity: 0, y: -8 }}
    transition={{ duration: 0.28, ease: [0.2, 0.8, 0.2, 1] }}
  />
</AnimatePresence>
```

- Duration: 280ms (snappy manga feel)
- Easing: cubic-bezier(0.2, 0.8, 0.2, 1) — custom "manga-ease"

### Reveal Component

```tsx
<Reveal direction="up" delay={200} duration={600}>
  <YourContent />
</Reveal>
```

- Directions: `up` (default), `down`, `left`, `right`, `scale`, `fade`
- Driven by `IntersectionObserver` (from `useReveal` hook)
- CSS transitions (not framer-motion) — uses `transition-[transform,opacity] will-change-transform`
- Respects `prefers-reduced-motion`

## Anime.js

Used in three files for the interactive SVG product mark:

| File | Usage | Cleanup |
|---|---|---|
| `mark/use-mark-interaction.ts` (225 lines) | Mouse tracking → eye/sphere offset animation (rAF-throttled), blink cycle, squint effect | ✅ Explicit `.pause()` on unmount + before starting new animations |
| `mark/mark-classic.tsx` | Node hover pulse (`.animate()` on circle radius), ambient stagger pulse on mount | ❌ **No cleanup on unmount** — ambient `loop: true` animations continue after component unmount |
| `mark/mark-cta.tsx` | Click reactions per button: sphere pulse (install), spin (docs), bounce (github) | ✅ Single-shot animations, no loop |

### ⚠️ Known Issue

`mark-classic.tsx` line 76-93 starts ambient pulsing animations with `loop: true` inside a `useEffect(() => {...}, [])` with NO cleanup function. The comment on line 93 ("Cleanup handled by anime.js on unmount") is incorrect — anime.js does NOT auto-cleanup when a DOM element is removed. Risk of memory leaks and ghost animations on component remount. Fix: store animation instances in a ref and call `.kill()` in the effect's cleanup.

## Count-up Hook

`useCountUp` in `src/hooks/count-up.tsx`:

- Uses `IntersectionObserver` to trigger when element enters viewport
- `requestAnimationFrame` loop with `easeOutCubic` (0 → target over 1200ms default)
- Handles complex stat strings: `"1.2ms"`, `"5,400"`, `"100%"`
- `CountUpStat` component wraps the hook for simple usage

## Synthetic Events (Cross-component Communication)

| Trigger | Listener | Mechanism |
|---|---|---|
| ⌘K button | `CommandPalette` | `window.dispatchEvent(new KeyboardEvent("keydown", { key: "k", metaKey: true }))` |
| `?` button | `ShortcutOverlay` | Same pattern — `{ key: "?" }` |
| Type "vanta" | `EasterEgg` | Global keydown listener |

⚠️ **Fragile coupling**: If `CommandPalette` changes its shortcut key, navbar/ShortcutHintButton break silently.
