# Icon System — Swiss + Neubrutalism

> Versión: 2.0 | 2026-07

---

## Styling Rules

| Property | Value |
|---|---|
| Stroke | Lucide icons, `stroke-width: 2` minimum |
| Size (inline) | `1.125rem` (18px) |
| Size (nav) | `1.25rem` (20px) |
| Size (hero) | `1.5rem` (24px) |
| Color default | `currentColor` |
| Color on cards | `var(--amber)` |
| Container | `.nb-icon-box` |
| Container size | `2.5rem` (40px) square |
| Container border | `2px solid var(--border-visible)` |
| Container radius | `0` |

## Container Pattern (`.nb-icon-box`)

```
┌────────────────────┐
│                    │
│       [icon]       │  20×20px stroke icon
│                    │
└────────────────────┘
  ← 40px →
```

| State | Border | Background |
|---|---|---|
| Resting | `var(--border-visible)` | `transparent` |
| Card hover | `var(--amber)` | — |

## Implementation

```tsx
<div className="nb-icon-box">
  <Search size={20} strokeWidth={2} />
</div>
```

## Prohibitions

- ❌ Filled icons (except brand mark)
- ❌ Custom SVG that doesn't fit 2px stroke style
- ❌ Multi-color icons
- ❌ Icon-only buttons without labels (except hamburger)
- ❌ Gradient, shadow, or glow on icons

## Approved Categories

| Category | Token |
|---|---|
| Search / Vector | `Search`, `Scan`, `Network` |
| Database / Storage | `Database`, `HardDrive`, `Server` |
| Performance | `Zap`, `Gauge`, `Activity` |
| Code / CLI | `Terminal`, `Code2`, `Brackets` |
| Security | `Shield`, `Lock`, `Key` |
| Arrow / Action | `ArrowRight`, `ChevronRight`, `ExternalLink` |
| Status | `Check`, `X`, `AlertCircle` |
| Navigation | `Menu`, `X`, `Home` |
