# Monetización Local-First Sin Nube y Sin Capital — Corrección de la Estrategia v1

**Fecha:** 2026-08-25 · **Autor:** vanta-lead (corrección post-feedback) · **Reemplaza a:** las recomendaciones de monetización de `validacion/02`, `validacion/03` y `validacion/00-SINTESIS-EJECUTIVA.md`

---

## 0. Corrección de premisas (errata de la v1)

La síntesis v1 cometió un error de marco: propuso monetizar como si VantaDB ya tuviera (o pudiera financiar) infraestructura cloud. Las premisas REALES del proyecto:

| Premisa | v1 (incorrecta) | v2 (correcta) |
|---|---|---|
| Producto | "Embedded + sync cloud pronto" | **100% embebido local-first. No hay nube ni la habrá por ahora** |
| Capital | Implicaba costes operativos asumibles | **Cero capital. Coste fijo mensual = $0 obligatorio.** Solo se puede pagar con % sobre ventas reales |
| Cobro | Stripe + planes desde el día X | **Cobrar solo cuando existan clientes que paguen**, con plataformas de coste-fijo-cero |
| Gancho de pago | Sync multi-dispositivo cloud | **Debe ser algo que corre en la máquina del cliente o papel legal/soporte** |

Los datos de mercado de los documentos 01–04 siguen siendo válidos (competencia, precios ajenos, patrones). Lo que cambia es **qué puede vender VantaDB HOY y con qué mecanismo**.

---

## 1. Investigación: vender software sin capital fijo (Merchant of Record)

Un **Merchant of Record (MoR)** es una plataforma que vende en tu nombre: maneja pago, impuestos/VAT global, facturación y reembolsos; cobra **solo un % sobre cada venta** (coste fijo $0). Es exactamente el instrumento para un proyecto sin capital ni entidad fiscal madura.

| Plataforma | Fee verificado | Notas |
|---|---|---|
| **Lemon Squeezy** | **5% + $0.50/venta** [confianza alta — corroborado por 2 fuentes independientes, ago-2026] | Adquirida por Stripe pero operativa standalone; license keys nativas para apps desktop; storefront lista en minutos |
| **Paddle** | **5% + $0.50/venta** [misma corroboración] | Más orientado SaaS; aprobación de cuenta más lenta |
| Gumroad | ~10% flat [confianza media — verificación bloqueada por bot-challenges] | El más simple; fee mayor |
| GitHub Sponsors | 0% (Stripe cobra su processing) | Donaciones/patrocinio; canal fase 0 |

**Regla de decisión:** Lemon Squeezy como storefront principal (fee menor que Gumroad, license keys incluidas). Stripe directo solo cuando el MRR justifique gestionar impuestos uno mismo (>~$1k MRR).

---

## 2. Qué se puede vender SIN servidores propios (evidencia verificada)

### SKU A — App Desktop Pro (el más realista y ya existe la base)
VantaDB YA tiene app desktop (Tauri, triple transport — verificado contra el repo en doc `06`). Modelo probado por herramientas DB desktop:

| Referencia | Precio verificado | Modelo |
|---|---|---|
| **TablePlus** | **$99 Basic (1 dispositivo) · $129 Standard · $79/seat Team — compra única, perpetua, 1 año de updates incluido** [extraído de tableplus.com/pricing hoy] | Freemium: versión free limitada + licencia perpetua |
| DBeaver PRO/Lite | ~$19–25/mes por usuario [confianza media — página de pricing no accesible en esta sesión] | Suscripción |

Propuesta concreta: **VantaDB Desktop Free** (put/get/search básico, 1 BD) + **VantaDB Desktop Pro $79 pago único** (explorador visual del grafo, multi-BD/dashboard, gestor de backups/export, benchmark suite integrado, temas, actualizaciones prioritarias). Distribución: binario firmado + license key vía Lemon Squeezy. Coste marginal: ~0 (la app ya existe; el trabajo es pulir features Pro).

### SKU B — Licencia comercial / OEM (papel, no infraestructura)
Para empresas que embeben VantaDB en productos cerrados y quieren garantía/indemnidad legal o exención de copyleft: contrato B2B por cotización. Anclajes históricos: Qt Commercial, iText (dual licensing AGPL+comercial), SEE de SQLite ~$2.000/producto [doc 02]. Rango sugerido: **desde ~$500–2.000/año según tamaño y redistribución**. Requiere CLA desde hoy (ver §3).

### SKU C — Soporte y warranty (modelo SQLite/Hwaci)
Contratos de soporte/warranty legal para empresas: costo marginal casi cero, credibilidad instantánea. Community gratis (GitHub/Discord) → Priority email ~$49–99/mes → Enterprise custom. Activar solo cuando haya usuarios empresariales reales preguntando.

### Lo que NO se puede vender todavía (requiere nube/capital)
Sync multi-dispositivo, backups gestionados, embeddings gestionados, consola web multiusuario, SLA 99.9% con uptime — todo eso es **FASE 2**, financiada con los ingresos de los SKUs A/B/C. La escalera Free/$19/$249/Enterprise del doc 03 queda intacta como **plan de fase 2**, no cancelada.

---

## 3. Licencia bajo cero capital (corrección de la recomendación FSL inmediata)

La v1 recomendaba migrar a FSL-1.1-MIT ya. Problema: requiere revisión legal profesional (coste real, capital que no existe) y prueba empírica de SPDX en crates.io. Recomendación corregida:

1. **HOY: mantener Apache-2.0** (ya está en el repo). Coste legal $0, fricción cero, es la licencia estándar del nicho memoria (6/9 líderes — doc 01). Acepta el riesgo teórico de resale: con 347 descargas/mes nadie va a resellar todavía; el enemigo actual es la invisibilidad, no los competidores.
2. **CLA ligero desde el primer commit externo** ($0, puro proceso): preserva el derecho futuro a relicenciar, ofrecer dual licensing OEM y defender el proyecto.
3. **Decisión FSL/AGPL+dual SOLO si:** aparece un reseller real, O antes de firmar el primer cliente enterprise. Cambiar de licencia con userbase ≈ 0 es barato (nadie a quien alienar); cambiarla tarde es el caso Elastic/OpenSearch (doc 03). La ventana existe — pero no hay que gastarla ni forzarla ahora.
4. **Marca registrada:** diferir hasta primer ingreso significativo (~$350 USPTO clase 9; en otras jurisdicciones menos). Apuntarlo como gatillo condicionado, no como tarea de hoy.

---

## 4. Sistema de cobro corregido (fase por fase, coste fijo siempre $0)

| Fase | Gatillo | Mecanismo | Coste fijo |
|---|---|---|---|
| **0 — hoy** | Ninguno (se activa ya) | GitHub Sponsors + OpenCollective | $0 |
| **1 — al lanzar Desktop Pro** | Desktop Pro listo para publicar | Lemon Squeezy storefront (licencias perpetuas + license keys) | 5%+$0.50 solo por venta |
| **1b — primeros clientes B2B** | Primera empresa interesada en OEM/soporte | Cotización manual + invoice/factura directa (MoR también sirve para esto) | $0 fijo |
| **2 — post-tracción** | MRR >$1k o demanda clara de sync | Stripe directo + construir el cloud (sync/backups/embeddings) → activar escalera Free/$19/$249/Enterprise del doc 03 | Ahora sí hay COGS, financiados por ingresos |

Gatillos medibles fase 1→2: ≥500 usuarios activos semanales del motor O ≥200 emails en waitlist del sync O Desktop Pro generando los primeros ~$500 MRR.

---

## 5. Impacto en las demás conclusiones (qué sigue válido)

- **Docs 01, 02 (mercado):** válidos al 100% como datos. Su sección "Lecciones" se reinterpreta: Turso sigue siendo la plantilla, pero para la **fase 2**.
- **Doc 04 (marketing/GTM):** válido casi completo — ya estaba diseñado a presupuesto cero. Única corrección: nada depende de una beta de sync; Show HN y Product Hunt se lanzan con playground WASM local + Desktop Free instalable.
- **Docs 05, 06, 07 (auditoría/docs/repo):** válidos completos (son hechos sobre el código, no supuestos de negocio). `.github/FUNDING.yml` sube de P1 a **P0 inmediato**: es el canal de cobro de la fase 0.
- **First-run contract (doc 07):** intacto y prioritario.

---

## Fuentes

- tableplus.com/pricing — extraído íntegro 2026-08-25 ($99/$129/$79-seat, perpetual) ✅
- apiscout.dev/guides/paddle-vs-lemon-squeezy-api-2026 + versustool.com/paddle-vs-lemonsqueezy — ambos citan "5% + $0.50" (corrobora ≥2 fuentes) ✅
- globalsolo.global/blog/stripe-vs-paddle-vs-lemon-squeezy-2026 — MoR handling tax ✅
- dbeaver.com/pricing — 404; búsquedas alternativas bloqueadas por bot-challenge → dato marcado confianza media
- Modelos dual-license y SQLite/warranty: ver fuentes de `validacion/02` y `validacion/03` (siguen vigentes)

## Lagunas

- Fee exacto vigente de Gumroad 2026 (no verificado; irrelevante si se elige Lemon Squeezy).
- Precios actuales de DBeaver PRO (marcados media).
- Términos exactos de license keys de Lemon Squeezy para apps offline (verificar al implementar; soportan validación offline con clave firmada).
