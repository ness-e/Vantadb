# Routing

## Page Patterns (3)

Based on analysis of all 30 `page.tsx` files:

### Pattern A — Legacy View Wrapper (4 pages)

Pages: `/` (home), `/benchmarks`, `/architecture`, `/docs`

```tsx
"use client";
import { ViewComponent } from "@/components/vanta/view-component";
import { useVantaNavigate } from "@/hooks/use-vanta-navigate";
export default function Page() {
  const navigate = useVantaNavigate();
  return <ViewComponent onNavigate={navigate} />;
}
```

These bridge from the old SPA `View` enum to App Router via `useVantaNavigate` adapter. Only 4 legacy views remain.

### Pattern B — i18n Direct (22 pages)

Pages: all `about/*`, `blog/*`, `case-studies/*`, `config`, `cost`, `integrations`, `latency`, `maint`, `pricing`, `security`, `showcase`, `solutions/*`, `storage`, `use-cases`, `why-vantadb`, `[...slug]`

```tsx
"use client";
import { PageHeader, PageSection } from "@/components/vanta/page-header";
import { useLanguage } from "@/lib/language-provider";
export default function Page() {
  const { t } = useLanguage();
  const tt = (key: string, fallback: string) => {
    const v = t(key);
    return v === key ? fallback : v;
  };
  // ... renders using PageHeader + PageSection + t()/tt()
}
```

Dominant pattern (73% of pages). Uses the custom i18n system directly.

### Pattern C — Custom Specialized (3 pages)

Pages: `/changelog` (ChangelogSection), `/engine` (CoreEngine + WalSimulator), `/playground` (CodePlayground)

```tsx
"use client";
import { SpecificComponent } from "@/components/vanta/specific-component";
export default function Page() {
  return <SpecificComponent />;
}
```

These have unique interactive components that don't follow the standard layout.

## Server Redirect Pages (2)

- `/demo/page.tsx` — `redirect("/playground")` (no `"use client"`)
- `/docs-api/page.tsx` — `redirect("/docs")` (no `"use client"`)

These are pure server components with no rendering.

## Special Server Files (3, not page.tsx)

- `opengraph-image.tsx` — Dynamic OG image via `next/og` ImageResponse
- `robots.ts` — `/robots.txt`, disallows `/api/` and `/_next/`
- `sitemap.ts` — `/sitemap.xml`: 22 static routes + 4 blog posts + 3 case studies

## Catch-All 404

`[...slug]/page.tsx` (Pattern B) catches all undefined routes. It renders a styled page instead of calling `notFound()`. No `not-found.tsx` exists at any level.

## Layouts

30 `layout.tsx` files total (1 root + 29 per-route). Each layout is a simple server-component wrapper that passes `{children}` through. Root layout provides:

- Font variables (Geist Sans/Mono, Anton, Space Mono)
- `LanguageProvider` context
- `SiteShell` (navbar, footer, modals)
- `Toaster` + `Sonner` notification components
- Skip-link for accessibility

## Notable Gaps

- **No `error.tsx`** at any level — errors bubble to default Next.js error overlay
- **No `loading.tsx`** at any level — no Suspense boundaries
- **No `not-found.tsx`** — uses catch-all route instead
- **No `middleware.ts`** — no locale negotiation, redirects, or auth
- **No route groups `(group)`** — flat route structure
- **`html lang="es"` is hardcoded** — never updated server-side when user switches language
