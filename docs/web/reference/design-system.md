# Design System Reference

## Palette (5 colors)

| Token | Hex | Usage |
|---|---|---|
| cream | #FBF9F5 | Page background, card backgrounds |
| ink | #000000 | Text, borders, rigid shadows |
| neon | #FF5500 | Accent — CTAs, highlights, active states (95/5 rule) |
| paper | #F2EDE2 | Secondary surfaces, muted sections |
| smoke | #1A1A1A | Dark variant elements (near-black) |

## Typography

| Font | Variable | CSS Class | Usage | Weights |
|---|---|---|---|---|
| Geist Sans | --font-geist-sans | font-sans | Body text | Variable |
| Geist Mono | --font-geist-mono | font-mono | Code/mono | Variable |
| Anton | --font-anton | font-display | Display/headlines | 400 |
| Space Mono | --font-space-mono | font-tech | Tech labels, stencil | 400, 700 |

## shadcn/ui Tokens

All 31 CSS variables in `:root` are set as hex values (NOT hsl). Key tokens:

| Token | Value |
|---|---|
| --background | #FBF9F5 |
| --foreground | #000000 |
| --primary | #000000 |
| --accent | #FF5500 |
| --border | #000000 |
| --radius | 0.125rem |

## CSS Utility Classes (42+)

### Text Treatments
| Class | Effect |
|---|---|
| .text-stencil | Stencil-cut text effect (Space Mono) |
| .text-outline | Hollow outline text |
| .text-outline-neon | Neon-outlined text |
| .marker-neon | Neon highlight marker sweep |

### Press Interactions
| Class | Effect |
|---|---|
| .press | Inset shadow press |
| .press-lg | Large press inset |
| .press-neon | Neon-colored press effect |

### Textures
| Class | Effect |
|---|---|
| .paper-bg | Subtle paper texture overlay |
| .paper-grain | Grain noise texture |
| .halftone | Halftone dot pattern |
| .halftone-neon | Neon halftone pattern |
| .hatch | Diagonal hatch pattern |
| .hatch-neon | Neon hatch pattern |
| .speed-lines | Manga-style speed lines |
| .speed-lines-radial | Radial speed lines |

### Shadows & Glow
| Class | Effect |
|---|---|
| .shadow-brutal | Hard 4px offset black shadow |
| .shadow-brutal-sm | 2px offset hard shadow |
| .shadow-brutal-lg | 6px offset hard shadow |
| .shadow-brutal-neon | Brutal shadow with neon tint |
| .glow-neon | Neon text glow |
| .glow-box-neon | Neon box glow |

### Decorative
| Class | Effect |
|---|---|
| .ink-corner | Ink-blot corner decoration |
| .ink-drip | Ink drip decoration |
| .manga-frame | Comic/manga panel frame |
| .ink-divider | Ink-style section divider |
| .tape | Masking tape element |
| .scanlines | CRT scanline overlay |
| .stripe-accent | Diagonal stripe accent bar |
| .accent-bar-top | Top accent bar |

### Hover Effects
| Class | Effect |
|---|---|
| .glitch-hover | Digital glitch on hover |
| .neon-underline | Neon underline on hover |
| .btn-neon-glow | Neon glow button on hover |

### Layout
| Class | Effect |
|---|---|
| .grid-tech | Technical grid background |
| .shadow-throw | Extended shadow cast |
| .animated-gradient-border | Animated gradient border |

## CSS Animations (13 keyframes)

| Name | Class | Duration | What It Does |
|---|---|---|---|
| fadeIn | animate-fade-in | 0.5s | Opacity 0→1 |
| fadeInUp | animate-fade-in-up | 0.5s | Opacity 0→1, translateY 20px→0 |
| fadeInDown | animate-fade-in-down | 0.5s | Opacity 0→1, translateY -20px→0 |
| fadeInLeft | animate-fade-in-left | 0.5s | Opacity 0→1, translateX -20px→0 |
| fadeInRight | animate-fade-in-right | 0.5s | Opacity 0→1, translateX 20px→0 |
| scaleIn | animate-scale-in | 0.3s | Scale 0.95→1, opacity 0→1 |
| slideUp | animate-slide-up | 0.3s | translateY 10px→0, opacity 0→1 |
| slideDown | animate-slide-down | 0.3s | translateY -10px→0, opacity 0→1 |
| neonPulse | animate-neon-pulse | 2s | Neon color intensity oscillation |
| glitch | animate-glitch | 0.3s | CSS clip-path displacement glitch |
| inkSpread | animate-ink-spread | 0.8s | Scale + opacity ink-bloom effect |
| grainShift | animate-grain-shift | 0.5s | Noise texture position shift |
| tapeReveal | animate-tape-reveal | 0.6s | Tape strip slide-across reveal |

## Dark Mode

**Not implemented.** No `.dark` or `[data-theme="dark"]` CSS variables exist. The project is light-mode only. `next-themes` is installed but dark mode would produce inverted contrast issues.

## Responsive Design

No breakpoints in CSS. All responsive behavior via Tailwind utility classes (`md:*`, `lg:*`, etc.) in component TSX. Default Tailwind v4 breakpoints.

## Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  *, *::before, *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
```

## Build Chain

- PostCSS: `@tailwindcss/postcss` (single plugin)
- No autoprefixer, postcss-preset-env, or custom plugins
- Tailwind v4 purges unused classes at build time
