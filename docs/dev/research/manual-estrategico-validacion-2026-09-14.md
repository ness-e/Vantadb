---
title: "Validación externa del Manual Estratégico Unificado (2026-09-14)"
type: research
status: active
tags: [vantadb, research, validacion, estrategia]
last_reviewed: 2026-09-15
aliases: []
related: []
---
# Validación externa del Manual Estratégico Unificado (2026-09-14)

> Origen: `docs/dev/strategy/VantaDB_Manual_Estrategico_Unificado.md` (v1.0, 31-jul-2026, fusión Gemini+GPT+Sonnet+GLM).
> Método: cada proposición accionable del manual se contrastó con fuentes primarias (docs oficiales, fee schedules) y literatura 2025-2026.
> Estado: las cifras del manual marcadas abajo con ❌ quedan corregidas; el marco URG/6M/A1 se mantiene.

## 1. Cobrar desde Venezuela (C15) — VERIFICADO con matices

- **Wise Business**: cuenta multidivisa con datos bancarios locales USD/EUR/GBP (ACH/SWIFT), conversión a mid-market, fees 0.45%-3%. Sirve para recibir de clientes y convertir. ([wise.com](https://wise.com/us/business/freelancer))
- **Payoneer**: 190+ países, recibe USD por transferencia local/SWIFT/ACH/tarjeta, retira a bancos locales y billeteras integradas (Bitso, Airtm, Belo, Prex — relevantes para VE). Fee FX 0.5%. ([payoneer.com](https://pages.payoneer.com/es/payment-account-b2b-duplicated))
- **Merchant of Record (clave para Eros)**: Polar Starter **5% + 50¢** (pricing 2026), Paddle **5% + 50¢**, Creem **3.9% + 40¢** todo-incluido, Gumroad ~13% (el más caro). El MoR es el vendedor legal: permite facturar software global **sin entidad US propia**. ([polar.sh](https://polar.sh/resources/pricing), [paddle.com](https://www.paddle.com/compare/gumroad), [creem.io](https://www.creem.io/comparisons/gumroad))
- **Conclusión**: el manual pide "contactar 2-3 proveedores" (correcto), pero omite la vía más corta: **MoR primero (días), entidad US después (semanas)**. Para USD 5.000 no hace falta LLC.

## 2. Stripe Atlas / LLC para venezolanos (C1) — VERIFICADO, matiz importante

- Stripe **no opera en VE** (lista oficial de países, [stripe.com/es-us/global](https://stripe.com/es-us/global)); Cuba/Irán/NK/Siria están prohibidos explícitos, VE no.
- La elegibilidad de Stripe se basa en el **país de registro del negocio, no la nacionalidad del dueño**: una US LLC 100% extranjera abre Stripe con EIN + banco US + dirección US. ([buykii.com 2026](https://www.buykii.com/banking/open-stripe-us-llc-non-resident), [terms.law 2026](https://terms.law/Invest-USA/bank-compliance/stripe-for-foreign-llc.html))
- EIN sin SSN vía Form SS-4: 2-6 semanas en 2026. Mercury/Relay abren 100% remoto para LLCs extranjeras.
- **Conclusión**: el manual es demasiado pesimista ("excluyen explícitamente a Venezuela" para Atlas — verificar caso por caso), pero su recomendación (preguntar directo antes de pagar) es correcta. Ruta: MoR ya → LLC Wyoming/Delaware + EIN + Mercury solo si el volumen lo justifica.

## 3. Trademark USPTO (C4/LEG-01) — VERIFICADO

- Base **$350/clase** (regla 2025, TEAS Plus/Standard eliminados), +$100 info insuficiente, +$200 free-form por clase. Mantenimiento: $325+$325/clase cada 10 años. ([uspto.gov](https://www.uspto.gov/trademarks/fees-payment-information/summary-2025-trademark-fee-changes), [costos](https://www.uspto.gov/trademarks/basics/how-much-does-it-cost))
- **Conclusión**: la estimación de LEG-01 ($2-5K con abogado, 1-2 clases) es coherente. Sin corrección.

## 4. Show HN (C11/C13) — COMPLEMENTADO

- Reglas 2026: sin vote-rings (hellban), primer comentario técnico propio, responder TODO rápido y sin defensividad, demo sin signup, pricing transparente, título específico. 500-2.000 visitas/24h un buen post. ([launchsuite.co 2026](https://launchsuite.co/blog/hacker-news-launch-guide), [stackmatix 2026](https://www.stackmatix.com/blog/launching-on-hacker-news), [yli.html](https://news.ycombinator.com/yli.html))
- **Conclusión**: el manual exige validación pre-lanzamiento (correcto) pero no da táctica HN. SHOW_HN_PREP.md ya tiene el draft + Q&A; le falta el checklist táctico (día/hora, primer comentario, rotación de respuestas). Propuesto como adición a EXE-03.

## 5. Números técnicos del manual — ❌ CORREGIDOS

| Claim manual (línea 38) | Realidad (fuente) |
|---|---|
| 9 adaptadores publicados en PyPI | Código existe, NO publicados (MKT-18f) |
| Recall 100% en GloVe | 24.5% en subset 10K (BENCHMARKS.md §7) |
| 622 QPS / ACORN 100% | Sin medición que lo respalde; usar BENCHMARKS.md |

## 6. Mapeo C1–C15 → backlog (2026-09-14)

| Bloque manual | Estado en backlog |
|---|---|
| C1 jurisdicción/banca | ❌ FALTA — proponer BIZ-02 |
| C2 IP assignment | ❌ FALTA — proponer BIZ-07 |
| C3 licencia Apache | ✅ cubierto (PRO-FEATURES, D4) |
| C4 marca | ✅ LEG-01 (negocio) |
| C5 ToS mínimo | ❌ FALTA — proponer BIZ-04 |
| C6 one-pager + C8 propuesta valor | ❌ FALTA — proponer BIZ-05 |
| C7 unit economics | ⏸️ A1, no fila (anotar) |
| C9 soporte | ✅ parcial (DISC negocio) |
| C10 ICP/vertical | ❌ FALTA decisión owner — proponer BIZ-08 |
| C11 design partners | ✅ MGR-20 + EXE-03 |
| C12 moat | ✅ PRO-FEATURES |
| C13 canal lanzamiento | ✅ SHOW_HN_PREP + EXE-03 (+ táctica HN) |
| C14 pricing inicial | ❌ FALTA — proponer BIZ-03 |
| C15 facturación VE | ❌ FALTA — proponer BIZ-02 (§1 y §7.1 de este doc son su research) |

## 7. Parte 2 (2026-09-14, profundización BIZ-02–08)
### 7.1 Payouts a Venezuela: el filtro que decide el MoR (BIZ-02)

- **Paddle ❌ como supplier venezolano**: su help center excluye explícitamente a Venezuela ([paddle.com](https://paddle.com/help/start/intro-to-paddle/which-countries-are-supported-by-paddle)). (Su lista de "supported countries" en dev-docs es de países *compradores*, no suppliers — no confundir.)
- **Lemon Squeezy ⚠️ parcial**: bank payouts en ~79 países (VE no listado); PayPal payouts en 200+ países/regiones ([docs](https://docs.lemonsqueezy.com/help/getting-started/supported-countries)). Si PayPal-VE recibe, la ruta existe pero el retiro local es el cuello (PayPal-VE no retira a bancos locales; termina en Airtm/cuentas US).
- **Polar ❓ a verificar**: cobra globalmente como MoR; los payouts salen por Stripe Connect Express (cobertura mayor que Stripe Payments). VE en esa lista **no confirmado** — es LA pregunta de BIZ-02.
- **Payoneer ✅ operativa base**: recibe USD (local/SWIFT/ACH/tarjeta) y retira a bancos y billeteras integradas incluyendo Airtm ([payoneer.com](https://pages.payoneer.com/es/payment-account-b2b-duplicated)). No es MoR (no factura por ti), pero como riel de cobro directo funciona hoy.
- **Alcance propuesto BIZ-02**: 1) confirmar por email/soporte si Polar/Creem/Lemon Squeezy pagan a VE (persona o banco); 2) si ninguno paga, operar con Payoneer directo + facturación propia; 3) documentar la ruta elegida con fees reales. Preguntas de alcance al owner tras este doc.

### 7.2 IP: assignment vs CLA vs DCO (BIZ-07, veracidad validada)

- **Assignment** = transferencia total de titularidad (FSF, Canonical). **CLA-licencia** (Apache ICLA) = licencia amplia sin transferir. **DCO** = atestación por commit, sin acuerdo ([davidru/OSS-Legal-Background](https://github.com/davidru/OSS-Legal-Background/blob/master/knowledge-base/04-contributor-agreements/cla-vs-dco.md), [Apache ICLA](https://www.apache.org/licenses/icla.pdf), [OASIS](https://www.oasis-open.org/open-projects/cla)).
- **Matiz honesto**: con 0 contribuidores externos, un "assignment" a nadie es teatro. Lo correcto en orden: 1) declaración de autoría y chain-of-title propia (1 página, hoy); 2) archivo DCO/CLA listo para el primer contribuidor; 3) assignment real cuando exista la entidad. BIZ-07 se crea con ese alcance escalonado, no como "documento corto" sin más.

### 7.3 ToS / pricing / one-pager / ICP (BIZ-03/04/05/08)

- **ToS (BIZ-04)**: MoR y Stripe exigen Terms/Privacy/Refund publicados para verificación ([terms.law 2026](https://terms.law/Invest-USA/bank-compliance/stripe-for-foreign-llc.html)). Alcance: 3 páginas desde plantillas + adaptación a VantaDB. Sin esto no hay BIZ-02.
- **Pricing (BIZ-03)**: en investigación de alcance — requiere comparar devtools comparables y decidir tiers contra un MoR concreto. Preguntas al owner tras este doc.
- **One-pager (BIZ-05)**: redactable por agente una vez fijado ICP (depende de BIZ-08).
- **ICP (BIZ-08)**: GTM trae 3 verticales con research serio (LanceDB→VantaDB, namespaces vs Chroma/PostgresSaver, MCP como canal). La evidencia interna apunta a **AI-IDE tooling vía MCP** como el de menor fricción (cero integración, 6 IDEs a la vez), pero la decisión es del owner. Se crea como decisión pendiente con ese resumen.

