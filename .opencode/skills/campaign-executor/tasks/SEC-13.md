---
id: "SEC-13"
name: "CSP nonce + HSTS headers"
created: "2026-07-14"
module: "web"
status: "ready"
estimate: "1 turn"
---

## Contract
"CSP nonce funcional en prod build, HSTS headers presentes"

## Atomic Steps
1. Add `'nonce-...'` to `style-src-elem` in `web/middleware.ts`
2. Verify: `npx tsc --noEmit`

## Skills
- security-and-hardening (CSP best practices)
- ponytail full

## Checks
- npx tsc --noEmit (web typecheck)

## Blast Radius
- **Files:** `web/middleware.ts` only
- **Callees:** Vite production build, Vercel edge
- **API Changes:** None — CSP directive change only, runtime behavior unchanged

## Investigation Notes
- HSTS already configured in `web/vercel.json` (max-age=63072000; includeSubDomains; preload) ✅
- CSP middleware already generates nonce for `script-src` but not for `style-src-elem`
- `style-src-attr 'unsafe-inline'` is needed for GSAP `element.style` manipulation (acceptable)
- Static CSP in `vercel.json` lacks nonce entirely but is overridden by middleware for HTML pages
