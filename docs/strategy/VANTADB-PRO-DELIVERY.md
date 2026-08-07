# VantaDB Pro — Delivery & Distribution

> Source plan: `docs/plans/2026-08-06-oc-vantadb-pro.md`
> Date: 2026-08-06
> Decisiones: D2 (propietaria), D3 (repo privado + artefactos), D5 (**pago **diferido** — entrega manual Enterprise** hasta entidad).
> Márgenes: el core `vantadb` (Apache-2.0) nunca empaqueta código Pro; `vantadb-pro` queda fuera del workspace.

## Principio de entrega

- **Nunca se entrega el source Pro.** El comprador obtiene un artefacto (binario/`.crate`) + una licencia (`vantadb.license`) firmada, válida por nodos y con expiración.
- El repo `vantadb-pro` es **privado** (`ness-e/vantadb-pro`); es la única fuente del artefacto.
- `cargo package`/`deny` del core **no** incluye ni audita `vantadb-pro` (ver `VANTADB-PRO-FEATURES.md` § verificación).

## Matriz de entrega por tier

| Tier | Licensing | Entrega | Licencia | Soporte |
|------|-----------|---------|----------|---------|
| **Community** | Apache-2.0 | Core + SDKs (público) | n/a (OSI) | Community (#help) |
| **Pro** | Propietaria (LicenseRef-Proprietary) | `.crate`/binario desde repo privado | `vantadb.license` por nodo | SLA (futuro) |
| **Business** | Propietaria | binario + tar | `vantadb.license` por nodo | SLA + consulting |
| **Enterprise** | Propietaria | on-prem, artefacto firmado | `vantadb.license` (generado manualmente por cliente) | SLA 24/7 + on-site |

> Los precios base (C7) están en `docs/strategy/GO_TO_MARKET.md` § Pricing; **no duplico cifras aquí** para no divergir.

## Cobro (D5 — APLAZADO)

- **Estado:** sin merchant/entidad aún. Se entrega Enterprise por **factura manual**; el vendedor (humano) ejecuta `scripts/generate-license.ps1` para emitir `vantadb.license` por cliente/nodo.
- **Cuando construir entidad:** elegir Merchant of Record (Polar/Paddle) para Pro/Business recurrente, o Wise/Payoneer para Enterprise. Documento aquí como nota, no como configuración.

## Generación y verificación de licencia

- **Emisión (manual, lado vendedor):** `vantadb-pro/scripts/generate-license.ps1 -Customer <email> -Nodes <n> -Expiry <yyyy-mm-dd>` → escribe `vantadb.license`.
- **Verificación (cada nodo):** `vantadb-pro/src/license.rs::verify_string` valida caducidad + límite de nodos de forma **offline**. 4 tests unitarios (vacío, válido futuro, vencido, exceso nodos).
- **Firma:** HMAC/Ed25519 **DIFERIDA** (ponytail ceiling). El check no incluye integridad criptográfica de la cadena; el secreto de firma **nunca** vive en VCS.