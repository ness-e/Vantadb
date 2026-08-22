# INV-015: Touch Targets < 44px — Auditoría

> **Estado:** ✅ COMPLETADA 2026-08-03 · **Fuente:** docs/Backlog.md INV-015 · **Tipo:** Web Frontend (accesibilidad) — auditoría + propuesta, sin implementación

## Resumen Ejecutivo

**~23 componentes interactivos no cumplen 44×44px** (WCAG 2.5.8 Target Size Minimum: 24×24 mínimo obligatorio, 44×44 recomendado). Todos pasan el mínimo de 24px **salvo 2** icon buttons de 14px (clear-search) que están **< 24px → fallo severo**.

## Inventario por prioridad

### P0 · Navbar (todos los viewports)
| Componente | Ubicación | Tamaño actual | Cumple 44px | Fix |
|---|---|---|---|---|
| Hamburger (menú mobile) | `site-navbar.tsx:378` | `h-9 w-9` = 36×36 | ❌ | `size-11` |
| Search ⌘K button | `site-navbar.tsx:356` | `h-9 w-9` = 36×36 | ❌ | `size-11` |
| Language toggle | `lang-toggle.tsx:14` | `px-2 py-1.5 text-xs` ≈ 32×40 | ❌ | `min-h-[44px] min-w-[44px] px-3` |
| Theme toggle | `theme-toggle.tsx:19` | `h-9 w-9` = 36×36 | ❌ | `size-11` |

### P1 · Close buttons modales/overlays
| Componente | Tamaño | Fix |
|---|---|---|
| `command-palette.tsx:228` | `h-7 w-7` = 28×28 | `size-11` |
| `shortcut-overlay.tsx:101` | `h-7 w-7` = 28×28 | `size-11` |
| `tutorial-modal.tsx:106` | `h-8 w-8` = 32×32 | `size-11` |
| `easter-egg.tsx:97` | `h-8 w-8` = 32×32 | `size-11` |
| `tutorial-modal.tsx:118` step segments | `h-1.5` = 6px alto | `min-h-[44px]` |

### P2 · Copy buttons
| Componente | Tamaño | Fix |
|---|---|---|
| `docs-view.tsx:563` CodeBlock | `h-7 w-7` = 28×28 + `opacity-0` hover-only | `size-11` |
| `docs-view.tsx:648` CliCard | `h-7 w-7` = 28×28 | `size-11` |
| `tutorial-modal.tsx:257` StepCodeBlock | `h-7 w-7` = 28×28 | `size-11` |
| `docs-view.tsx:596` CopyButton | `px-2 py-0.5 text-[10px]` ≈ 22px alto | `min-h-[44px] min-w-[44px]` |
| `code-terminal.tsx:196-213` Run/Copy | `px-2 py-0.5 text-[10px]` ≈ 22px | `min-h-[44px] min-w-[44px]` |

### P3 · Nav links text-only
| Componente | Tamaño | Fix |
|---|---|---|
| `footer.tsx:155` 31 nav buttons | `text-[11px]` sin padding ≈ 16-18px | `min-h-[44px] flex items-center` |
| `footer.tsx:119-142` community links | `text-xs` ≈ 18px | `min-h-[44px] flex items-center` |
| `site-navbar.tsx:398` mobile group headers | `px-1 py-2.5` ≈ 36px | `min-h-[44px]` |
| `site-navbar.tsx:420` mobile group items | `px-2 py-2` ≈ 31px | `min-h-[44px]` |
| `site-navbar.tsx:441` mobile flat links | `px-3 py-2` ≈ 35px | `min-h-[44px]` |
| `docs-view.tsx:167` sidebar sections | `px-2 py-1.5` ≈ 32px | `min-h-[44px]` |

### P4 · Clear/filter icon buttons
| Componente | Tamaño | Fix |
|---|---|---|
| `changelog-section.tsx:81` clear-search | icon 14px sin padding | `size-11` |
| `docs-view.tsx:148` search clear | icon 14px | `size-11` |
| `changelog-section.tsx:93` filters | `px-2 py-1` ≈ 26px | `min-h-[44px] min-w-[44px]` |
| `latency-comparator.tsx:267` dataset | `py-2` ≈ 35px | `min-h-[44px]` |

## Ya cumplen ✅

- back-to-top 48×48
- hero CTAs ≈52px
- cta-final buttons ≈48px
- benchmark-race ≈52px
- FAQ accordion row `p-4` ≈76px
- FAQ CTA links ≈44px
- architecture/benchmarks-view CTA ≈52px

## Dead code excluido (web/AUDIT.md)

`navbar.tsx`, `hero-mark-interactive.tsx`, `ecosystem.tsx`, `metrics-bar.tsx`.

## Patrón de fix

- **Icon-only:** `size-11` (=44px)
- **Text targets:** `min-h-[44px] min-w-[44px]`

## Orden de implementación sugerido

P0 navbar → P1 close → P2 copy → P3 nav links → P4 clear/filter.

## Notas

- No auditar componentes shadcn/ui base (`web/src/components/ui/`) — Radix ya cumple.
- Solo auditoría + propuesta (alcance del backlog). Cero cambios de código.
