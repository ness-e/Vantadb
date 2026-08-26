# Licenciamiento OSS y Monetización Open-Core para VantaDB

**Fecha:** 2026-08-25 · **Autor:** Agente C (research delegada) · **Estado:** investigación completa; requiere revisión legal profesional antes del anuncio público

---

## Resumen ejecutivo

> ### RECOMENDACIÓN
> **Licencia del núcleo (motor Rust + SDKs Python/TS/WASM + CLI): FSL-1.1-MIT** (Functional Source License, creada por Sentry, estándar del movimiento *Fair Source*).
> **Módulos servidos (sync cloud, consola, RBAC server, embeddings gestionados): propietarios**, en reposo privado — open-core duro en la capa servida.
> **Arquitectura de planes: 4 en total — Free + 3 pagos** (Starter $19 · Pro $249 · Enterprise custom), con overages usage-based.

**Por qué FSL y no otra cosa:** VantaDB necesita simultáneamente (a) fricción cero en PyPI/npm/crates.io para adopción masiva, (b) disuasión contra resale por clouds, y (c) compatibilidad enterprise. Las licencias restrictivas puras fallan en (a) y/o (c): Elastic y Redis terminaron **retrocediendo hacia open source real** (AGPL) tras perder comunidad y sufrir forks (OpenSearch, Valkey); HashiCorp generó OpenTofu; CockroachDB eliminó su versión gratuita y cosechó críticas. La FSL resuelve el trilema: uso, estudio, modificación y redistribución libres desde el día 1; la única restricción es **no competir con el producto del licenciante**, y cada versión **convierte automáticamente a MIT a los 2 años**. Un competidor cloud que quiera resellar queda fuera; un developer que embebe el motor en su app no nota diferencia alguna con MIT.

*Alternativa válida si la fricción FSL preocupa:* mantener el núcleo en Apache-2.0/MIT como el resto del sector memoria (6/9 líderes lo hacen) y aceptar el riesgo de resale, compensando con velocidad de distribución + moat del servicio. Decisión final = revisión legal + prueba empírica de registro SPDX en crates.io.

---

## Parte 1 — Opciones de licencia y casos reales (estado 2024–2026)

### 1.1 Tabla de opciones

| Licencia | Quién la usa | Resultado observado | Riesgo para VantaDB |
|---|---|---|---|
| **MIT / Apache-2.0** | Supabase, Neon, PostHog (core), Ghost, SQLite* (PD), Mem0, Qdrant | Adopción máxima; monetización vía servicio gestionado | **Alto**: cualquier cloud puede resellar el motor sin pagar |
| **MPL-2.0** | Firefox; antiguo Terraform | Copyleft débil por archivo; convivencia pacífica | Alto como única defensa (no cubre SaaS) |
| **GPL-3.0** | GitLab CE histórico | Fuerte en distribución, hueco total en SaaS | Medio: incomoda legal departments sin frenar clouds |
| **AGPL-3.0** | Grafana, Cal.com, MinIO, Elastic/Redis (opción añadida 2024/2025) | Cierra hueco SaaS; empresas con política "no-AGPL" evitan la dependencia (SFC documenta grandes empresas listando Affero GPL como *"Never Allowed Here"* por enforcement MongoDB) | Medio-alto en una **librería embebida**: ambigüedad de linking espanta adopción enterprise |
| **SSPL** | MongoDB (2018→), Elastic (2021–24), Redis (2024–25, retirada) | No aprobada por OSI; excluida de distros; Elastic y Redis la abandonaron/diluyeron | Muy alto: percepción "fake open source" |
| **ELv2** | Elastic (junto a SSPL/AGPL) | Funcional pero perpetuamente restrictiva | Alto: sin fecha de conversión |
| **BSL 1.1** | HashiCorp (Terraform), CockroachDB (2019–2024), MariaDB MaxScale (origen) | Restrictiva 4 años luego GPLv2+-compatible; HashiCorp provocó fork OpenTofu (CNCF) | Medio: mejor que SSPL/ELv2 pero 4 años "no open source" |
| **FSL 1.1** | **Sentry** (desde dic-2023), PowerSync, Fair Source | Sin incidentes públicos; Sentry mantiene control comercial mientras cada versión madura hacia MIT | Bajo-medio: no es OSI (fricción distros menor); exige CLA propio |
| **Dual licensing (AGPL + comercial)** | Qt, MySQL clásico, iText | Ingresos por cumplimiento; requiere CLA estricto | Medio: hereda fricción AGPL |

\* SQLite es public domain explícito (`sqlite.org/copyright.html`).

### 1.2 Casos documentados (con fuentes primarias)

| Caso | Qué hizo | Qué pasó | Lección |
|---|---|---|---|
| **MongoDB** | Oct-2018: AGPL → SSPL (contra el servicio gestionado de AWS) | SSPL rechazada por OSI; AWS reimplementó (DocumentDB) en vez de cumplir | Restringir hospedaje no impide reimplementación; aleja a la comunidad |
| **Elastic** | Feb-2021: Apache-2.0 → SSPL+ELv2; **29-ago-2024: añade AGPLv3** | Fork OpenSearch (AWS); Shay Banon reconoce que el cambio fue por "issues with AWS" y 3 años después "Amazon está totalmente invertido en su fork" | El daño comunitario dura años aunque se revierta; elegir bien desde el inicio |
| **HashiCorp/Terraform** | 10-ago-2023: MPL-2.0 → BUSL 1.1 | Fork inmediato: OpenTofu, donado a Linux Foundation, hoy CNCF, drop-in replacement | Relicensar sin conversión garantizada ni buena voluntad previa = nace tu competencia |
| **Redis** | Mar-2024: SSPL/RSALv2; **1-may-2025: Redis 8 añade AGPLv3** (tri-license) | Nace Valkey (Linux Foundation); CEO Rowan Trollope admite que SSPL *"hurt our relationship with the Redis community"* | Segundo retroceso consecutivo del sector: las restricciones duras no retienen valor, retienen rencor |
| **Sentry** | Dic-2023: crea y adopta la **FSL** | Estable: código público total, no-compete, conversión MIT/Apache a los 2 años por versión | El modelo recomendado: probado por una empresa comparable |
| **CockroachDB** | Oct-2019: ASL → BSL; ago-2024: elimina Core gratuito (licencia propietaria source-available v24.3) | Gratis solo <$10M/año; crítica fuerte por telemetría obligatoria en tier gratuito; NixOS tuvo que reempaquetar | Eliminar el free tier en fase de crecimiento mata el embudo; no repetir |
| **MinIO** | 2021: todo AGPLv3 | Enforcement activo: README advierte riesgo legal del uso comercial; caso público contra Weka (mar-2023); demanda SFC 2021 [cita NO VERIFICADA] | AGPL da armas legales reales, pero genera preguntas de compliance en cada cliente de una librería embebida |
| **Supabase** | Core Apache-2.0; monetiza hosting/dashboards | 108k+ estrellas; razón citada: evitar que la licencia mate el efecto sustrato (caso Elastic/MongoDB/HashiCorp) | Permisivo funciona si el moat es el servicio — pero deja el motor expuesto |
| **Neon** | Core Apache-2.0 + servicio | Adquirida por Databricks (2025); historia limpia para procurement | Ídem Supabase |
| **PostHog** | MIT core + carpeta `ee/` propietaria *(confianza media)* | Crece rápido | Patrón organizativo replicable ("carpeta EE") |
| **Cal.com** | AGPLv3 core + licencia comercial EE *(confianza media)* | Self-host libre; enterprises pagan | Variante viable en apps servidas, no librerías embebidas |
| **Ghost** | MIT core + hosting de pago *(confianza media)* | Sostenible vía hosting oficial | Ídem |
| **GitLab** | Open-core clásico CE/EE | Modelo open-core más longevo B2B | Valida split dos-capas |
| **SQLite** | Dominio público + soporte/patrocinio | Éxito total como software; monetización casi nula vía licencias | Dominio público = máximo regalo, mínimo control |

### 1.3 Evaluación contra criterios de VantaDB

| Criterio | MIT/Apache | MPL-2.0 | AGPL-3.0 | SSPL | BSL 1.1 | ELv2 | **FSL 1.1** ⭐ |
|---|---|---|---|---|---|---|---|
| Fricción cero en crates/PyPI/npm | 🟢 | 🟢 | 🟡 | 🔴 | 🟡 | 🟡 | 🟢 |
| Disuasión resale cloud | 🔴 | 🔴 | 🟢 | 🟢 | 🟢 | 🟢 | 🟢 |
| Compatibilidad enterprise | 🟢 | 🟢 | 🔴 | 🔴 | 🟡 | 🟡 | 🟢 |
| Percepción comunidad | 🟢 | 🟢 | 🟡 | 🔴 | 🟡 | 🔴 | 🟢 |
| Recupera "open source" con el tiempo | — | — | — | ❌ nunca | ✅ 4 años | ❌ nunca | ✅ **2 años** |

Nota: FSL-1.1-MIT tiene identificador SPDX (confianza media); npm/PyPI aceptan campos arbitrarios; la aceptación en crates.io debe probarse empíricamente antes del anuncio (ver Lagunas).

---

## Parte 2 — Split gratis/pago para VantaDB

**Principio rector:** *todo lo que corre local/embebido en la máquina del usuario es gratis para siempre; se paga lo que requiere nuestra infraestructura, ahorra meses de trabajo operacional a un equipo, o sirve a organizaciones.* Replica lo validado: Turso cobra exactamente el sync embebido (límites GB/mes por plan), Mem0 separa SDK open-source de Platform, Qdrant separa OSS de Cloud.

| Feature | Tier |
|---|---|
| Motor embebido completo: put/get, búsqueda híbrida BM25+HNSW+RRF | 🆓 Free (para siempre) |
| Persistencia, WAL, ACID, compactación local | 🆓 Free |
| SDKs completos: Python, TypeScript/JS, WASM (npm) | 🆓 Free |
| CLI básica | 🆓 Free |
| MCP server | 🆓 Free |
| Embeddings client-side con BYO-key (OpenAI/Voyage/local) | 🆓 Free |
| Cifrado en reposo local con clave del usuario | 🆓 Free |
| **Sync cloud multi-dispositivo** (gancho de conversión) | 💰 Starter+ |
| Backups gestionados / Point-in-Time Restore | 💰 Starter (7d) → Pro (30d) → Enterprise (custom) |
| Consola/dashboard de administración web | 💰 Pro+ |
| **Embeddings automáticos gestionados** (nuestra infra + keys) | 💰 Pro+ (Free/Starter quedan en BYO-key) |
| Telemetría/observabilidad (dashboards de uso, latencias) | 💰 Pro+ |
| Cifrado gestionado KMS/BYOK | 💰 Pro / Enterprise |
| **RBAC + servidor multi-tenant** | 💰 Enterprise |
| Soporte con SLA (99.9%, 24×7) | 💰 Enterprise |
| **Licencia OEM / redistribución comercial** | 💰 Enterprise (contrato aparte) |

---

## Parte 3 — Arquitectura de planes

| Plan | Precio | Contenido | Justificación |
|---|---|---|---|
| **Free** | $0 | Motor + SDKs + CLI + MCP completos, ilimitado en local. Embeddings BYO-key. Comunidad Discord/GitHub | Embudo de adopción; espeja Qdrant Cloud Free (perpetuo, sin tarjeta) y Mem0 Hobby |
| **Starter** | **$19/mes** | Sync multi-dispositivo hasta 5 GB/mes, backups 7 días, 1 proyecto cloud, soporte email best-effort, seats ilimitados | Ancla exacta en Mem0 Starter ($19). Por debajo de Turso Scaler ($24.92). Micro-pago temprano |
| **Pro** | **$249/mes** | Sync 50 GB/mes, PITR 30 días, consola completa, embeddings gestionados incluidos (fair-use), dashboards telemetría, Slack privado, proyectos ilimitados | Ancla exacta en Mem0 Pro ($249). Cubre rango mid-market Zep ($104–$312/mes) |
| **Enterprise** | Custom (~desde $1.000–2.000/mes equivalente) | Sin límites negociados + RBAC/multi-tenant server self-hostable con licencia OEM, SSO/SAML, audit logs, BYOK/KMS, SLA 99.9% con crédito, DPA, VPC dedicado/on-prem, soporte 24×7 | Estándar del sector: Turso, Qdrant Premium, MotherDuck Business/Enterprise y Mem0 Enterprise todos custom |

**Modelo de cobro recomendado — híbrido flat + usage:** suscripción plana por predictibilidad (los developers odian sorpresas) con overages usage-based como Mem0: GB de sync adicional ~$0.35/GB y operaciones de embedding gestionado a tarifa unitaria, calzados con la escala de Turso ($0.75/GB Developer → $0.45/GB Pro). Seats: **ilimitados** en Starter/Pro (evita fricción; el valor está en el uso del servicio, no en las personas).

**Complementos tempranos:**
1. **GitHub Sponsors + OpenCollective: activar día 1.** Coste cero, señal de sostenibilidad, canal para empresas pequeñas.
2. **Stripe: activar al abrir la beta pública del sync** (trigger sugerido: ≥500 usuarios activos semanales o lista de espera sync ≥200, lo primero que ocurra). Antes, facturar es sobrecarga sin demanda demostrada.

---

## Riesgos y mitigaciones

| # | Riesgo | Mitigación |
|---|---|---|
| 1 | A los 2 años cada versión pasa a MIT → fork de versión congelada | Es el diseño: el fork siempre queda 24 meses atrás. Velocidad de release + marca registrada + moat del servicio lo hacen irrelevante (patrón Sentry, 2+ años sin incidentes) |
| 2 | FSL no es OSI → exclusión Debian/Fedora y fricción procurement | Transparencia radical (todo el código visible), página `/licensing`; si hay fricción medible, liberar bindings SDK bajo MIT manteniendo motor en FSL |
| 3 | Sin control de copyright no se aplica FSL ni vende OEM | CLA ligero (o DCO + grant amplio) **desde el primer commit externo** |
| 4 | Percepción "open source de mentira" en HN/Reddit | Mensajería Fair Source explícita ("código 100% visible, MIT en 2 años"); nunca llamarlo "open source" en marketing |
| 5 | Precio mal calibrado | Revisión a los 6 meses con datos reales de conversión; overages como válvula; anclar cambios a movimientos de Mem0/Turso |
| 6 | Soporte abruma a equipo pequeño | SLA solo en Enterprise; Starter/Pro best-effort; comunidad como primera línea |
| 7 | Resale directo del motor durante ventana FSL | Cláusula no-compete de FSL-1.1 lo prohíbe; **registrar marca VantaDB antes del mes 24** (la conversión a MIT no transmite marcas) |

---

## Fuentes consultadas

**Primarias extraídas íntegras:**
- Elastic — "Elasticsearch Is Open Source. Again!", Shay Banon, 29-ago-2024: https://www.elastic.co/blog/elasticsearch-is-open-source-again
- Redis — "Redis is now available under the AGPLv3 open source license", Rowan Trollope, 01-may-2025: https://redis.io/blog/agplv3/

**Primarias verificadas:**
- Elastic licensing FAQ: https://www.elastic.co/pricing/faq/licensing · IR press release: https://ir.elastic.co/News--Events/news/news-details/2024/Elastic-Announces-Open-Source-License-for-Elasticsearch-and-Kibana-Source-Code/default.aspx
- Redis licenses: https://redis.io/legal/licenses/ · LWN: https://lwn.net/Articles/1019686/ · Phoronix: https://www.phoronix.com/news/Redis-8.0-Goes-AGPLv3
- OpenTofu fork: https://opentofu.org/blog/opentofu-announces-fork-of-terraform/
- FSL: https://fsl.software/ · https://open.sentry.io/licensing/ · InfoQ: https://www.infoq.com/news/2023/12/functional-source-license/
- CockroachDB: https://www.cockroachlabs.com/blog/enterprise-license-announcement/ (vía NixOS #335274 e ItsFOSS) · https://www.infoq.com/news/2024/09/cockroachdb-license-concerns · https://itsfoss.com/news/cockcroachdb-no-open-source · https://linuxiac.com/cockroachdb-shifts-to-enterprise-solution-only/ · https://docs.cockroachlabs.com/docs/stable/licensing-faqs
- MinIO: https://www.min.io/blog/from-open-source-to-free-and-open-source-minio-is-now-fully-licensed-under-gnu-agplv3 · https://github.com/minio/minio (LICENSE/README) · caso Weka: https://news.ycombinator.com/item?id=35299665
- Supabase LICENSE: https://github.com/supabase/supabase/blob/master/LICENSE · Neon LICENSE: https://github.com/neondatabase/neon/blob/main/LICENSE
- Precios: mem0.ai/pricing · qdrant.tech/pricing · turso.tech/pricing · motherduck.com/product/pricing/

**Análisis secundario:**
- Goodwin Law, "Moving Away From Open Source" (sep-2024): goodwinlaw.com
- Simon Willison: simonwillison.net/2024/Aug/29/elasticsearch-is-open-source-again/ · TechCrunch Fair Source
- Zep pricing vía agregadores (costbench.com) — tratar como orden de magnitud

**[cita NO VERIFICADA]:** demanda SFC en nombre de MinIO contra Vizivity Interactive/WeWork (2021) — resolver URL antes de citar públicamente.

---

## Lagunas

1. **Revisión legal profesional obligatoria** antes de anunciar FSL (elección de Change License MIT y redacción del Grant Criteria propio).
2. **Verificación empírica pendiente:** aceptación de `FSL-1.1-MIT` en crates.io y PyPI (SPDX ID existe con confianza media; probar con crate dummy).
3. URL demanda SFC–MinIO 2021 sin resolver.
4. Licencias PostHog/Ghost/GitLab/Cal.com confirmadas solo vía fuentes secundarias; contrastar contra LICENSE de cada repo.
5. Exclusión SSPL de Debian/Fedora afirmada con confianza media (dato histórico establecido, no re-verificado).
6. Este documento propone estrategia, no asesoría jurídica: el texto definitivo de la licencia aplicada (Change Date, Change License, Grant Criteria) debe redactarlo un abogado de licenciamiento.
