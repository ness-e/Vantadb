# Component Library — Swiss + Neubrutalism

> Versión: 2.0 | 2026-07
> Convenciones globales: `border-radius: 0`, hard offset shadows, 2px borders, 80ms transitions

---

## Layout Components

| Component | Class | Description |
|---|---|---|
| Section | `.nb-section` | Section container with 96px vertical padding |
| Section (dark) | `.nb-section--dark` | Dark background variant |
| Section (large) | `.nb-section--lg` | 160px vertical padding |
| Inner | `.nb-inner` | Centered content, max 1200px |
| Inner (wide) | `.nb-inner--wide` | Centered content, max 1440px |
| Grid 2-col | `.nb-grid--cols-2` | 2 equal columns, 1px gap |
| Grid 3-col | `.nb-grid--cols-3` | 3 equal columns, 1px gap |
| Grid 4-col | `.nb-grid--cols-4` | 4 equal columns, 1px gap |
| Grid 6-col | `.nb-grid--cols-6` | 6 equal columns, 1px gap |
| Cell | `.nb-cell` | Grid cell — bg `--background`, pad `--space-lg` |
| Cell span 2 | `.nb-cell--span2` | Spans 2 columns |
| Cell span 3 | `.nb-cell--span3` | Spans 3 columns |
| Cell row 2 | `.nb-cell--row2` | Spans 2 rows |
| Asymmetric | `.nb-asymmetric` | 8fr 4fr split |
| Asymmetric right | `.nb-asymmetric--right` | 4fr 8fr split |
| Split 7-5 | `.nb-split-7-5` | 7fr 5fr split |
| Split 5-7 | `.nb-split-5-7` | 5fr 7fr split |
| Bento | `.nb-bento` | Grid with 1px gap |
| Bento 3-col | `.nb-bento--3col` | 3-column bento |
| Bento 4-col | `.nb-bento--4col` | 4-column bento |
| Bento cell | `.nb-bento-cell` | Bento grid cell |
| Bento featured | `.nb-bento-cell--featured` | 2x2 anchor cell |
| Bento span 2 | `.nb-bento-cell--span2` | 2-column span |
| Bento span 3 | `.nb-bento-cell--span3` | 3-column span |
| Divider | `.nb-divider` | 2px horizontal line |
| Divider amber | `.nb-divider--amber` | Amber-colored divider |
| Divider strong | `.nb-divider--strong` | Strong border divider |
| Section header | `.nb-section-header` | Label + heading block |
| Section header bordered | `.nb-section-header--bordered` | With bottom border |
| Section header right-border | `.nb-section-header--right-border` | With right border |

## Surface Components

| Component | Class | Description |
|---|---|---|
| Card | `.nb-card` | 2px border, hard shadow, hover → amber border |
| Card amber | `.nb-card--amber` | Amber border permanently |
| Card strong | `.nb-card--strong` | Strong border (#333) |
| Frame | `.nb-frame` | 2px border with floating label via `data-frame-label` |
| Block warning | `.nb-block-warning` | 3px amber border, `⚠ WARNING` label |
| Icon box | `.nb-icon-box` | 40px square, 2px border, centered amber icon |

## Typography Components

| Component | Class | Description |
|---|---|---|
| Label | `.nb-label` | Section label — micro mono, uppercase |
| Label amber | `.nb-label--amber` | Amber-colored label |
| Index | `.nb-index` | Numeric index — `[01]`, steel → amber on hover |
| Index amber | `.nb-index--amber` | Amber index permanently |
| Bracket | `.nb-bracket` | Auto-wraps in `[ ]` monospace brackets |
| Arrow | `.nb-arrow` | Amber link with `>>>` suffix, gap animates on hover |
| Pill status | `.nb-pill-status` | Status indicator with dot |
| Pill amber | `.nb-pill-status--amber` | Amber-colored pill |
| Pill green | `.nb-pill-status--green` | Green-colored pill |
| Telemetry | `.nb-telemetry` | Data row with `>` prefix |

## Interactive Components

| Component | Class | Description |
|---|---|---|
| Primary button | `.btn-primary` | Amber bg, hard shadow, mechanical press |
| Ghost button | `.btn-ghost` | Transparent bg, border, hard shadow |
| Ghost inverted | `.btn-ghost--inverted` | For dark surfaces |
| Install button | `.btn-install` | Monospace install command button |
| List | `.nb-list` | Telemetry-style list with `>` prefix |
| Table | `.nb-table` | Monospace data table, visible borders |

## Animation Components

| Component | Class | Description |
|---|---|---|
| Ticker | `.nb-ticker` | Opacity flash 0.8s `steps(1)` — live indicator |
| Cursor | `.nb-cursor` | Amber cursor blink 1s `step-end` |
| Split flip | `.nb-split` | Vertical split-flip animation container |
| Split inner | `.nb-split-inner` | Animated inner element |

## Background Textures

| Component | Class | Description |
|---|---|---|
| Scanline | `.scanline` | CRT scanline overlay (fixed, z-noise) |
| Noise | `.noise-overlay` | Fractal noise grain (fixed, 3.5% opacity) |
| Dot grid | `.nb-bg-dot` | Radial dot grid, 24px spacing |
| Cross grid | `.nb-bg-cross` | Cross hatch grid, 24px spacing |
| Cross grid faint | `.nb-bg-cross--faint` | Subtle cross grid, 48px spacing |

## Grid Hairlines

| Component | Class | Description |
|---|---|---|
| Hairline V | `.nb-hairline-v` | 1px vertical grid line |
| Grid overlay | `.nb-grid-overlay` | SVG grid overlay (absolute positioned) |

---

## Usage Examples

### Card with Index
```tsx
<div className="nb-card">
  <span className="nb-index">[01]</span>
  <div className="nb-icon-box">
    <SearchIcon />
  </div>
  <h3 className="text-title">Hybrid Search</h3>
  <p className="text-body">HNSW + BM25 in a single query.</p>
  <a className="nb-arrow">Learn more</a>
</div>
```

### Telemetry Row
```tsx
<div className="nb-telemetry">
  <span>QUERY: 1.2ms</span>
  <span>RECALL: 0.998</span>
  <span>INDEXED: 10K</span>
</div>
```

### Warning Block
```tsx
<div className="nb-block-warning">
  <p>This API will be deprecated in v0.8.0. Migrate to the new query interface.</p>
</div>
```

### Frame with Label
```tsx
<div className="nb-frame" data-frame-label="ARCHITECTURE">
  {/* content */}
</div>
```

### Bento Grid with Anchor
```tsx
<div className="nb-bento nb-bento--3col">
  <div className="nb-bento-cell nb-bento-cell--featured">
    {/* featured content — spans 2x2 */}
  </div>
  <div className="nb-bento-cell">{/* content */}</div>
  <div className="nb-bento-cell">{/* content */}</div>
  <div className="nb-bento-cell nb-bento-cell--span2">{/* content */}</div>
</div>
```

### Mechanical Button
```tsx
<button className="btn-primary">
  GET STARTED
</button>
```
