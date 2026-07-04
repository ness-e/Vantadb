# Router Migration Evaluation: TanStack Router → React Router

**Author:** AI-assisted evaluation  
**Date:** 2026-07-03  
**Ticket:** WEB-17  

---

## 1. Current State

| Property | Value |
|---|---|
| **Router** | TanStack Router v1.168.25 |
| **Plugin** | `@tanstack/router-plugin` v1.167.28 (Vite) |
| **Total routes** | 23 unique paths + `__root__` |
| **Route convention** | File-based (`routes/` directory) with auto-generated `routeTree.gen.ts` |
| **Lazy loading** | Yes — every route uses `createLazyRoute`/`.lazy()` split |
| **Route params** | 1 — `/blog/$slug` |
| **Redirects** | 1 — `/docs-api` → `/docs` |
| **Search params** | 0 |
| **Loaders** | 0 |
| **`<Link>` usages** | 28 across 6 components + 2 route files |
| **`import` references** | 57 files importing from `@tanstack/react-router` |

### Dependencies

```json
"@tanstack/react-router": "^1.168.25",
"@tanstack/router-plugin": "^1.167.28"
```

### Manual chunking

Already chunked separately in `vite.config.ts`:
```ts
if (id.includes("@tanstack/react-router") || id.includes("@tanstack/react-query"))
  return "vendor-router";
```

---

## 2. Pros of Current Setup (TanStack Router)

- **Type-safe routing** — Full type inference from file tree to `<Link to="/...">`. TypeScript catches broken route references at compile time.
- **File-based convention** — Routes mirror file structure. Adding a new page = adding a file. No manual route config to maintain.
- **Built-in head/meta management** — `Route.head()` provides SSR-compatible `<head>` control per route without extra libraries.
- **Lazy loading by default** — Every route is automatically code-split via `.lazy()` pattern. No config required.
- **Route context injection** — `createRootRouteWithContext<{ queryClient }>()` provides clean DI of React Query client to all routes.
- **Scroll restoration** — Built-in (`scrollRestoration: true`).
- **Error/404 boundaries** — Built-in `notFoundComponent` and `errorComponent` at root level.
- **Route transitions** — `useMatches()` + `routeId` enables AnimatePresence keyed animations per route.

---

## 3. Cons of Current Setup

- **Generated file with `@ts-nocheck`** — `routeTree.gen.ts` (577 lines) has `// @ts-nocheck` and `/* eslint-disable */`, meaning the file bypasses all type/lint checks. Brittle if the generator produces unexpected output.
- **Heavy dependency** — 2 packages + 1 Vite plugin for routing. Bundle already manually chunked, but adds complexity.
- **Overkill for 23 mostly-static pages** — No loaders, no search params, no guards, no authentication. TanStack Router's loader pipeline, search param parsing, and middleware features are unused.
- **Learning curve** — `createFileRoute`, `createLazyRoute`, `createRootRouteWithContext`, `getParentRoute`, `useMatches`, `Route.useParams()`, `Route.useRouteContext()`. Contributors must learn TanStack-specific patterns.
- **Plugin dependency** — `@tanstack/router-plugin/vite` must run during dev/build to generate `routeTree.gen.ts`. If the plugin breaks or lags during Vite/React upgrades, routing is blocked.
- **Route file split required** — Every route needs 2 files (`route.tsx` + `route.lazy.tsx`) by convention, even for trivial pages. Doubles file count (50 route files for 25 routes).
- **React Router ecosystem reuse** — Components, hooks, and patterns learned elsewhere (e.g., React Router's `useNavigate`, `useParams`, `<NavLink>`) don't transfer directly. TanStack Router uses similar but subtly different APIs.

---

## 4. React Router Alternative

| Aspect | React Router (v7) | TanStack Router (v1) |
|---|---|---|
| **Package** | `react-router-dom` (1 package) | `@tanstack/react-router` + `@tanstack/router-plugin` (2+ packages + plugin) |
| **Type safety** | Via `generatePath` + manual types | Automatic from file tree |
| **Code generation** | None | Required (`routeTree.gen.ts`) |
| **API surface** | Simpler: `<Routes>`, `<Route>`, `<Link>`, `useParams`, `useNavigate` | More: `createFileRoute`, `createLazyRoute`, `createRootRouteWithContext`, `useMatches`, `Route.useParams()`, etc. |
| **Lazy loading** | `React.lazy(() => import(...))` | Built-in `.lazy()` convention |
| **Head management** | None built-in (use `react-helmet-async` or `@unhead/react`) | Built-in via `Route.head()` |
| **Community** | Larger, more resources, more job experience | Smaller but growing |
| **Bundle size** | ~14 KB (min+gzip) | ~25 KB (min+gzip, router only, before plugin) |
| **Static routes** | Route config or `<Route>` elements | File-based convention |

---

## 5. Migration Cost Analysis

### 5.1 Route Definitions

**TanStack Router (current):**
- 2 files per route: `routes/page.tsx` (definition) + `routes/page.lazy.tsx` (component)
- Generated `routeTree.gen.ts` connects them

**React Router (target):**
- 1 file per route component
- Central `<Routes>` config in `router.tsx` (or `main.tsx`)

> **Cost:** Medium. Flatten 50 files → ~25 files. Write a central route config. Remove `routeTree.gen.ts`.

### 5.2 Link Components: 28 usages

**Current:** `<Link to="/docs" className="...">`  
**Target:** `<Link to="/docs" className="...">` (identical JSX, different import)

**Cost:** Low. Search-and-replace import path in 8 files.

### 5.3 Navigation Hooks

| Current | Target | Usages |
|---|---|---|
| `useLocation()` | `useLocation()` | 1 (Nav.tsx) — API differs slightly (TanStack returns `URL`; React Router returns `Location`) |
| `useRouter()` | N/A | 1 (`__root.tsx` error boundary — `router.invalidate()`). Need to handle error recovery differently. |
| `useMatches()` | `useMatches()` | 1 (`__root.tsx` — for route transition key). Both have `useMatches` but return shapes differ. |

**Cost:** Low-Medium. `useRouter().invalidate()` is the only non-trivial migration — the error boundary would need a different recovery approach (e.g., window.location.reload or React state reset).

### 5.4 Route Params

**Current:** `Route.useParams()` in `blog/$slug.lazy.tsx`  
**Target:** `useParams()` from `react-router-dom`

**Cost:** Low. Single usage.

### 5.5 Redirect

**Current:** `throw redirect({ to: "/docs" })` in `docs-api.tsx`  
**Target:** `<Navigate to="/docs" />` or `useNavigate()('/docs')`

**Cost:** Low. Single usage.

### 5.6 Head/Meta Management

**Current:** `Route.head()` provides route-level `<title>`, `<meta>`, `<link>`, `<script>` tags.  
**Target:** No built-in head management. Would need `react-helmet-async`, `@unhead/react`, or a custom `<Helmet>` component in each route.

**Cost:** Medium-High. 15+ route files with `head()` calls. Largest migration item.

### 5.7 Route Context (React Query)

**Current:** `createRootRouteWithContext<{ queryClient: QueryClient }>()`  
**Target:** Wrap `<Routes>` in `<QueryClientProvider>` at the top level. No route-level context needed.

**Cost:** Low. Already have `<QueryClientProvider>` wrapping in root. Would move to `main.tsx` or stay around the `<Routes>` element.

### 5.8 Scroll Restoration

**Current:** `scrollRestoration: true` (built-in)  
**Target:** `useScrollRestoration()` or `<ScrollRestoration />` in React Router v7

**Cost:** Low.

### 5.9 Route Transitions (AnimatePresence)

**Current:** `useMatches()` → `routeId` as motion key  
**Target:** `useLocation().pathname` or `useMatches()` key

**Cost:** Low. Change the key source from `routeId` to `location.pathname`.

### 5.10 Error / Not Found Boundaries

**Current:** `notFoundComponent` + `errorComponent` on root route  
**Target:** React Router v7 `errorElement` on `<Route>`, custom 404 `<Route path="*">`

**Cost:** Low. Pattern exists in both routers.

---

## 6. Estimation Summary

| Category | Files Affected | Effort |
|---|---|---|
| Route definition files | ~25 route `.tsx` + 25 `.lazy.tsx` | Medium |
| `routeTree.gen.ts` | 1 (delete) | Trivial |
| `router.tsx` | 1 | Medium |
| `main.tsx` | 1 | Low |
| `vite.config.ts` | 1 (remove plugin) | Trivial |
| `package.json` | 1 (swap deps + add helmet) | Low |
| Link imports | 8 component files | Low |
| Head/meta | 15+ route files | Medium-High |
| Navigation hooks | 2 files (`Nav.tsx`, `__root.tsx`) | Low-Medium |
| Route params | 1 file (`blog/$slug`) | Low |
| Redirect | 1 file (`docs-api.tsx`) | Low |
| Scroll restoration | 1 file (`router.tsx`) | Low |
| Route transitions | 1 file (`__root.tsx`) | Low |

**Total estimated effort: 2-4 days for a confident migration.**

---

## 7. Recommendation

**Keep TanStack Router for now — not blocking launch.**

Rationale:
- The current setup works, is type-safe, and has zero known bugs.
- 23 pages is small enough that neither router causes meaningful performance issues.
- TanStack Router's automatic lazy loading and head management are actively beneficial.
- The bundle is already manually chunked (`vendor-router` separate from main app).
- The `@ts-nocheck` in `routeTree.gen.ts` is cosmetic — the generated types are consumed correctly via the `FileRoutesByPath` module augmentation.

**Re-evaluate post-launch if any of these become true:**
1. Bundle size for the `vendor-router` chunk exceeds 30 KB gzipped.
2. Developing new routes becomes cumbersome due to the 2-file-per-route convention.
3. A team member reports friction with TanStack Router's DX vs React Router.
4. The `routeTree.gen.ts` generated file causes merge conflicts in team PRs.
5. React Router v7 delivers significant enough ergonomic improvements to justify the migration cost.

**If migrating later:**
- Use `react-helmet-async` or `@unhead/react` for head management.
- Keep page components as-is (they're already Router-agnostic); only swap the routing layer.
- Use React Router v7's `createBrowserRouter`/`RouterProvider` for the closest API parity with the current code structure.
