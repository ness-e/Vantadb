# WDA-07 — F7 Diseño comercial

**Plan:** `docs/plans/2026-08-19-web-design-audit.md` Task 8 · **Ruta:** vanta-worker
**Estado:** ✅ COMPLETADA (sin commit — prohibido por orquestador) · **Reglas duras:** no tocar web/remotion/, desktop/. Sin git commit.

## Objetivo

Hero con value prop completo (qué + para quién) y jerarquía de CTA; pricing honesto
(sin Team plan inexistente); funnel /demo coherente (playground WASM ya disponible,
sin waitlist falsa); JSON-LD logo apuntando a asset local vía SITE_URL.

## Impacto mapeado (Regla 0)

Archivos leídos completos:
- `web/src/components/vanta/hero.tsx` (199L) — H1 solo marca; subhead sin audiencia;
  4 CTAs + install button compitiendo (Quickstart ya es bg-neon pero secundarios
  llevan mismo `btn-neon-glow`).
- `web/src/app/layout.tsx` (151L) — JSON-LD `logo:` apunta a raw.githubusercontent
  avatar_gato.png → **404 confirmado** (`web/public/assets/avatar_gato.png` NO existe).
  `favicon.png` SÍ existe en web/public.
- `web/src/components/vanta/vanta-data.ts` (secciones PRICING_PLANS ~550-590,
  TCO_COMPARISON ~745-775) — fila Enterprise TCO: `"Custom (Team/Enterprise plan)"`
  menciona plan inexistente; comentario "3 plans" obsoleto (hay 2); Enterprise CTA
  "Contact Sales" → ctaLink VANTA.repo (GitHub) = label incoherente.
- `web/src/app/demo/page.tsx` + `demo/layout.tsx` — page redirige a /playground;
  metadata promete beta/waitlist que no existe.
- `web/src/lib/dictionaries.ts` — claves demoPage.* ES+EN: **0 usos fuera del
  diccionario** (rg -ln "demoPage\." → solo dictionaries.ts). Dead keys con copy
  falso de waitlist.
- `web/src/lib/site-config.ts` — constante SITE_URL (WDA-02).
- `web/src/app/cost/page.tsx` (150L) — renderiza TCO_COMPARISON; columna vantadb
  SIN tt() (fallback directo de vanta-data), scenario/note SÍ via tt con claves
  costPage.tco.N.* (la clave tco.3.vantadb no existe — no hay que tocar dicts acá).

Referencias entrantes verificadas:
- hero.tsx ← page.tsx home (no cambia su interfaz: props onNavigate intactas)
- TCO_COMPARISON/PRICING_PLANS ← cost/page.tsx, pricing/page.tsx (solo lectura de datos)
- dictionaries ← LanguageProvider (agregar/borrar claves es seguro)

Veredicto: cambio de copy/metadata acotado a 5 archivos + dicts. Sin lógica nueva,
sin tests requeridos (sitio sin infra de tests). Verificación: greps + lint + build.

## Spec

S1 Hero:
- Nueva línea de audiencia bajo H1 vía clave `hero.audience` (ES+EN): comunica
  PARA QUIÉN (devs construyendo agentes IA / RAG local).
- Jerarquía CTA: Quickstart queda primario único (neón + glow); Benchmarks y Source
  bajan a secundarios (mismo border-4 cream, sin btn-neon-glow, hover invertido).
  Install-copy queda como elemento técnico, sin glow competitivo.

S2 Pricing honesto:
- TCO fila 4: `"Custom (Team/Enterprise plan)"` → `"$0 (self-hosted)"` — coherente
  con las otras filas ($0), tag de página ("$0 forever · self-hosted") y nota
  existente (Enterprise suma soporte/compliance, no licencia).
- Comentario `// Pricing - 3 plans` → `// Pricing - 2 plans`.
- Enterprise CTA label: "Contact Sales" → EN "Contact via GitHub" / ES "Contactar
  vía GitHub" (el destino real es VANTA.repo). Clave `pricingPage.plan.1.cta`.

S3 Funnel demo:
- `demo/layout.tsx`: title/description/OG alineados a lo real — redirige al
  playground interactivo (disponible HOY), sin beta/waitlist/coming soon.
- Borrar dead keys demoPage.* (ES líneas ~1304-1324, EN ~2790-2810): copy falso
  eliminado de la fuente.

S4 JSON-LD:
- `logo: ${SITE_URL}/favicon.png` usando la constante importada (favicon.png existe).
- opengraph-image.tsx existe en app/ — Next lo detecta por convención; no se toca.

## Steps

- ✅ S1: hero audience + jerarquía CTA + claves dict ES/EN
- ✅ S2: TCO row + comentario planes + CTA Enterprise label (data + dicts)
- ✅ S3: demo/layout.tsx metadata + borrar dead keys demoPage.*
- ✅ S4: JSON-LD logo → SITE_URL/favicon.png
- ✅ VERIFY: greps contrato + npm run lint (0 err) + npm run build (exit 0)

## Context Save Point

S1-S4 implementados y verificados en una sola pasada. Ver greps/lint/build en VERIFY_CONTRATO del RESULTADO. Excepción raw.githubusercontent documentada arriba (docs-view.tsx install commands vivos).

## Contrato verificable

- Hero tiene público objetivo explícito visible y UN CTA primario distinto
- `grep -i "team plan" web/src` = 0
- `grep "raw.githubusercontent" web/src` = 0 en layout/assets; ⚠️ EXCEPCIÓN
  DOCUMENTADA: docs-view.tsx conserva 2 URLs raw de scripts/install.{sh,ps1} — son
  comandos de instalación FUNCIONALES (scripts existen y resuelven HTTP 200 en main,
  verificado por fetch), no assets rotos. Eliminarlos rompería instrucciones reales.
- Copy demo sin waitlist/beta · claves nuevas simétricas ES+EN · lint 0 · build exit 0
