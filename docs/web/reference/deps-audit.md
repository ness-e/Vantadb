# Dependency Audit

## Summary

- Total packages in package.json: ~80+
- Active packages (code imports): ~10-15
- Truly dead (0 imports): 16
- Zombie packages (only in dead ui stubs): 28
- Available savings: ~32MB+ if dead deps removed

## Truly Dead Packages (16)

| Package | Est. Size | Notes |
|---|---|---|
| sharp | ~30MB native | Image processing — only needed if using next/image opt |
| react-syntax-highlighter | ~800KB | Syntax highlighting — no code blocks rendered |
| @mdxeditor/editor | ~500KB | Rich text editor — not imported anywhere |
| date-fns | ~300KB | Date utility — never imported |
| next-auth | ~200KB | Auth library — no auth endpoints exist |
| @tanstack/react-table | ~150KB | Table logic — not imported |
| next-intl | ~80KB | i18n — no translations exist |
| @tanstack/react-query | ~70KB | Server state — not imported |
| @dnd-kit/core | ~50KB | Drag-and-drop — not imported |
| @dnd-kit/sortable | ~30KB | Sortable DnD — not imported |
| @dnd-kit/utilities | ~20KB | DnD utilities — not imported |
| zustand | ~15KB | State management — not imported |
| @reactuses/core | ~15KB | React hooks collection — not imported |
| react-markdown | ~12KB | Markdown renderer — not used |
| uuid | ~4KB | UUID generation — not imported |
| z-ai-web-dev-sdk | ? | Unknown SDK — not imported |

## Zombie Packages (28)

All dependencies that exist solely because shadcn/ui stubs import them, but those stubs are never rendered:

| Package | Imported By (Dead Stub) |
|---|---|
| @radix-ui/react-accordion | ui/accordion.tsx |
| @radix-ui/react-alert-dialog | ui/alert-dialog.tsx |
| @radix-ui/react-aspect-ratio | ui/aspect-ratio.tsx |
| @radix-ui/react-avatar | ui/avatar.tsx |
| @radix-ui/react-checkbox | ui/checkbox.tsx |
| @radix-ui/react-collapsible | ui/collapsible.tsx |
| @radix-ui/react-context-menu | ui/context-menu.tsx |
| @radix-ui/react-dialog | ui/dialog.tsx |
| @radix-ui/react-dropdown-menu | ui/dropdown-menu.tsx |
| @radix-ui/react-hover-card | ui/hover-card.tsx |
| @radix-ui/react-label | ui/label.tsx |
| @radix-ui/react-menubar | ui/menubar.tsx |
| @radix-ui/react-navigation-menu | ui/navigation-menu.tsx |
| @radix-ui/react-popover | ui/popover.tsx |
| @radix-ui/react-progress | ui/progress.tsx |
| @radix-ui/react-radio-group | ui/radio-group.tsx |
| @radix-ui/react-scroll-area | ui/scroll-area.tsx |
| @radix-ui/react-select | ui/select.tsx |
| @radix-ui/react-separator | ui/separator.tsx |
| @radix-ui/react-slider | ui/slider.tsx |
| @radix-ui/react-switch | ui/switch.tsx |
| @radix-ui/react-tabs | ui/tabs.tsx |
| @radix-ui/react-toast | ui/toast.tsx / toaster.tsx **(active)** |
| @radix-ui/react-tooltip | ui/tooltip.tsx |
| @radix-ui/react-use-callback-ref | helper (transitive) |
| @radix-ui/react-use-controllable-state | helper (transitive) |
| recharts | ui/chart.tsx |
| react-hook-form | ui/form.tsx |

> Exception: `@radix-ui/react-toast` is genuinely used by `toaster.tsx` and `toast.tsx` — the only zombie that earns its keep.

## shadcn/ui Consumption Reality

Of 30+ installed primitives, only 3 are consumed:

| Stub | Location | Actually Used? |
|---|---|---|
| toaster.tsx | components/ui/toaster.tsx | ✅ SiteShell |
| sonner.tsx | components/ui/sonner.tsx | ✅ SiteShell |
| toast.tsx | components/ui/toast.tsx | ✅ via use-toast.ts |
| button.tsx | components/ui/button.tsx | ❌ Never rendered |
| card.tsx | components/ui/card.tsx | ❌ Never rendered |
| dialog.tsx | components/ui/dialog.tsx | ❌ Never rendered |
| dropdown-menu.tsx | components/ui/dropdown-menu.tsx | ❌ Never rendered |
| form.tsx | components/ui/form.tsx | ❌ Never rendered |
| chart.tsx | components/ui/chart.tsx | ❌ Never rendered |
| + 20+ more | components/ui/* | ❌ Never rendered |

If we kept only consumed stubs, ~27 peer dependencies could be removed.

## Other Notes

- `tailwind.config.ts` is inert — it uses v3 syntax (`module.exports`, `theme.extend`) in a v4 project where Tailwind is configured via CSS `@theme inline {}`.
- `components.json` is used by the shadcn-ui CLI when adding new primitives; it has no runtime impact.
- The project imports both `tw-animate-css` (v4 native, CSS-level animations) **and** `tailwindcss-animate` (v3 PostCSS plugin, same feature) — redundant. The v3 plugin has no effect under Tailwind v4; only `tw-animate-css` is active.
