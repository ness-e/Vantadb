# Custom Hooks — `src/hooks/`

8 hooks total. All single default exports. No external hook libraries used.

## Hook Reference

| Hook | File | Lines | Signature | Purpose |
|---|---|---|---|---|
| `useVantaNavigate` | `use-vanta-navigate.ts` | 92 | `() => (view: View) => void` | Legacy View enum → `router.push()` adapter for 3 core routes (`/`, `/benchmarks`, `/docs`). Also exports `LIVE_ROUTES` set (35 paths) and `isLiveRoute()` checker |
| `useCountUp` | `count-up.tsx` | 134 | `<T extends HTMLElement>(target: number, options?: { duration?, decimals?, threshold? }) => { value, ref }` | `IntersectionObserver` + rAF with `easeOutCubic`. Parses stat strings via `parseStat()`. Respects `prefers-reduced-motion`. Also exports `CountUpStat` component and `parseStat` utility |
| `useReveal` | `use-reveal.ts` | 55 | `<T extends HTMLElement>(options?: { threshold?, rootMargin?, once? }) => { ref, visible }` | `IntersectionObserver` for scroll-triggered entrance. Default threshold 0.15, rootMargin `0px 0px -60px 0px`. Respects `prefers-reduced-motion` (shows immediately) |
| `useIsMobile` | `use-mobile.ts` | 19 | `() => boolean` | Matches `--mobile` CSS breakpoint (768px) via `matchMedia`. Returns `boolean` |
| `useParallax` | `use-parallax.ts` | 42 | `(intensity?: number) => { x, y }` | Mouse-based parallax offset normalized to [-1, 1] via rAF. Respects `prefers-reduced-motion` (returns 0,0) |
| `useFocusTrap` | `use-focus-trap.ts` | 82 | `(ref: RefObject, active: boolean) => void` | Traps Tab/Shift+Tab within a container for modals. Focuses first focusable element on mount, restores previous focus on unmount |
| `useTypingLines` | `use-typing-lines.ts` | 68 | `(totalLines: number, options?: { threshold?, lineDelay? }) => { ref, visibleLines, done }` | Typewriter effect for code terminal. Reveals lines one-by-one on scroll. Respects `prefers-reduced-motion` (shows all immediately) |
| `useToast` | `use-toast.ts` | 194 | `() => { toasts, toast, dismiss }` | shadcn/ui toast state management. Module-level reducer pattern with listener array. Also exports standalone `toast()` function |

## Details

### `useVantaNavigate`
Bridges the old SPA `onNavigate: (view: View) => void` interface (used by 20+ components) to the Next.js App Router. Maps `View` to real URLs and calls `router.push()` + `window.scrollTo(0,0)`.

### `useCountUp` + `CountUpStat`
Two exports from one file. The hook handles the animation logic; the component wraps it for convenient use:
```tsx
<CountUpStat value="5,400 vec/s" />
```
Parses numeric prefix from strings like `"1.2ms"`, `"5,400"`, `"100%"`.

### `useReveal`
Accepts `once?: boolean` (default `true`). Set `once: false` for elements that re-animate on each scroll-into-view.

### `useFocusTrap`
Takes an `active: boolean` — only traps when `active` is true. Cleans up `keydown` listener on deactivation.

### `useTypingLines`
Takes `totalLines: number` (not `string[]`). Use with `<CodeTerminal>` to reveal syntax-highlighted lines progressively.

## Patterns

- All hooks that interact with the DOM use `RefObject` with a generic `T extends HTMLElement` parameter
- Animation hooks (`useCountUp`, `useReveal`, `useTypingLines`, `useParallax`) all check `prefers-reduced-motion` and short-circuit for accessibility
- No `"use client"` directive in `use-mobile.ts` — it's the only hook that works server-side (no DOM interaction, just `matchMedia` in `useEffect`)
