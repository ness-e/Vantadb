# Plan de Ejecución: Open Core VantaDB + VantaDB Pro/Enterprise (licenciamiento comercial)

> **Campaign ID:** ea6ae974-7d79-4978-b8c7-7b2f2186edae
> **Inicio:** 2026-08-06
> **Estado:** ✅ D1-D6 RESPONDIDAS — core verificado, docs pendientes de commit (Task 11/12/15 + commit Task 18)
> **Fuente:** decisión del usuario + `VantaDB_Manual_Estrategico_Unificado.md` (C3/C7/C15) + `VantaDB_Auditoria_Tecnica.md`
> **Modelo elegido:** Open Core — motor `Apache-2.0` público (sin relicenciar) + ediciones **Pro/Enterprise propietarias** en repo/artefacto separado (`vantadb-pro`).
> **Contexto legal real (investigación 2026-08-06):** SurrealDB usa BSL 1.1 en el motor con *Additional Use Grant* anti-DBaaS y cambio a Apache a los ~4 años; su fuente oficial: `github.com/surrealdb/license`. Para VantaDB (motor embebido, no servidor) el modelo recomendado y elegido es **Open Core**: core Apache-2.0 (adopción máxima) + capa propietaria aparte (moat real, features difíciles de clonar). El riesgo "hiperscaler te clone" NO se resuelve con AGPL/BSL a nivel del core; sí con marca/CLA/features pegados al servicio.

## ⚠️ Regla de ejecución

- **Fase 0 = DECISIONES HUMANAS bloqueantes.** Cada tarea F0 es una pregunta que el agente DEBE plantear al usuario (con las opciones abajo) y registrar la respuesta en `campaign_memory_write(file="decisions")`. **El harness NO debe completar F0 de forma autónoma** — el parser no debe avanzar más allá de F0 mientras queden tareas de decisión sin responder. Si el harness intenta ejecutarlas solas, detener y pedir al usuario.
- Las tareas de F0 tienen `Estado: ⏸ PENDING (humano)`.
- **No crear workspaces**: `vantadb-pro` NO se agrega a `members` del workspace ni a `default-members` (`Cargo.toml:590-607`) para que el core no dependa de él.
- **No empaquetar pro en el core**: verificar `Cargo.toml` exclude + `deny.toml` (solo MIT/Apache-2.0) — `vantadb-pro` queda exento del gate `deny` del core.

## Resumen

| DO | DEFER | SKIP | BLOQUEADO |
|----|-------|------|-----------|
| 14 | 2 | 0 | 0 (salvo F0 humanas) |

## Fases

| Fase | Contenido | Tareas |
|------|-----------|--------|
| F0 | **Decisiones humanas** (licencia, dónde vive pro, features pro, entrega, marca) — 6 preguntas con opciones | 1-6 |
| F1 | Auditar features core → qué pasa a Pro (listado con paths:líneas) | 7 |
| F2 | Esqueleto `vantadb-pro` + licencia + empaquetado (estructura, no código feature) | 8-10 |
| F3 | Entrega/artefactos (licencia por nodo + registry privado / binario) | 11-12 |
| F4 | Reglas nuevas + AGENTS.md + docs nuevos (licensing, CI, policía) | 13-15 |
| F5 | Documentar el proceso post-cambio (ADR + memoria + plan de GTM) | 16 |
| F6 | Cierre (verificación full + commit + progreso) | 17-18 |

---

## Fase 0 — DECISIONES HUMANAS BLOQUEANTES (owner: human)

> **Cómo funciona:** cada task de esta fase es una pregunta al usuario. El agente presenta las opciones, el usuario elige, y el agente única la respuesta. **El harness no debe continuar a F1 sin que el usuario responda al menos a D1, D2, D3, D5.**

### Task 1: D1 — ¿Licencia del CORE?
- **Archivos clave:** `Cargo.toml:7`, `LICENSE`, `docs/plans/2026-08-06-oc-vantadb-pro.md`
- **Pregunta:** ¿Qué licencia mantiene el motor `vantadb` (core embebido)?
- **Opciones (elegir UNA):**
  - **A) Apache-2.0 (recomendado)** — máxima adopción, cero fricción enterprise, es lo que ya tienes. No protege contra clona al detalle pero tu moat = features + marca. Mantener `LICENSE` y `Cargo.toml:7` tal cual. **Elegida en Estrategia (C3).**
  - B) relicen a AGPL — NO recomendado: mata adopción de empresas de IA (justo tu ICP). Fricción legal intensa con contribuidores.
  - C) relicen a BSL motor (modelo SurrealDB) — protege el motor de DBaaS al años4, pero dejas de ser "Open Source" (OSI) y HN/comunidad lo penaliza. Diferente de lo que C3 eligió.
- **Contrato:** core sigue Apache-2.0; ADR-013 aceptado y registrado en memoria.
- **Estado:** ✅ COMPLETED

### Task 2 — ¿Licencia del PRODUCTO Pro/Enterprise?
- **Pregunta:** ¿cómo licencias la capa `vantadb-pro`?
- **Opciones:**
  - **A) Propietaria estricta ("All rights reserved" + licencia comercial)** — la más simple, SIN fecha de liberación. Comprador recibe artefacto + licencia por nodo. **Recomendada para MVP.**
  - **B) BSL 1.1 con Additional Use Grant** — se vuelve Apache a los ~4 años. Más "open" pero fuerza fecha de liberación.
  - **C) Dual**: Apache para el core + Pro vendida como componente (de facto = A).
- **Contrato:** texto de `vantadb-pro/LICENSE` (Fase 2, cumplido: LICENSE propietaria).
- **Estado:** ✅ COMPLETED

### Task 3: Decisión — ¿DÓNDE vive el código Pro / cómo se entrega?
- **Pregunta:** ¿dónde guardo el código de pro y cómo lo entregó al que paga?
- **Opciones (elige UNA o COMBINACIÓN):**
  - **A) Repo GitHub privado `vantadb-pro`** (privado, no público) — control total. Entrega vía `.crate`/`.whl`/binario compilado (nunca el source). (EJECUTADA — repo `ness-e/vantadb-pro` creado y pusheado)
  - **B) Solo artefactos/sin el source**: entregas `.tar.gz`/`.whl` firmado, sin repo. Menos control.
  - **C) Registry privado `cargo`/`pip`** con token por cliente — para la SDK pro (futuro, no MVP).
- **Contrato:** repo/artefacto + método de entrega en `docs/strategy/VANTADB-PRO-DELIVERY.md`.
- **Estado:** ✅ COMPLETED

### TASK 4: Decision — ¿Qué features del core pasan a Pro?
- **Pregunta:** de los features hoy en `Cargo.toml` como gates, ¿cuáles son "Pro" (con quitar del default/rabba o mover a pro) y cuáles quedan libres en el core?
  - **Candidatos Pro (door competencia):** `encryption` (L117, ./aes-gcm/sha2), `wal-shipping` (L118), `pitr` (L119), `server` + `tls` (L121-128), `prometheus` (L129), `replicación`/`RBAC`/`multi-tenancy` (si existen hooks).
  - **Libres (deben seguir en core):** `fjall`/`rocksdb` backend, `arrow`, `cli`, `advanced-tokenizer`, `remote-inference`, `failpoints`, `roaring`.
- **Pregunta además:** ¿el core debe **reducir su default** quitando de Pro? ¿O Pro = nuevas features que aún no existen (no tocar el core)?
  - **A) NO tocar el core actual**: Pro son features NUEVAS que construirás (recomendado, cero riesgo, cero relicenciado, cumple C3 "features enterprise nacen en repo separado desde el día 1").
  - **B) Mover gates existentes** (encryption/wal-shipping/pitr/prometheus) de core→pro: requiere refactor + break a usuarios, más riesgo.
- **Contrato:** en F1 se genera `docs/strategy/VANTADB-PRO-FEATURES.md` con cierre in/out por feature (CUMPLIDO — archivo real en disco).
- **Estado:** ✅ RESPONDIDA (opción A: features nuevas, no tocar core)

### Task 5: Estado de pago/jurisdicción (comercial MVP)
- **Pregunta:** ¿qué plataforma para aceptar pagos internacionales?
- **Decisión (2026-08-06):** **D) Aplazar** — sin merchant entidad aún; entrega Enterprise manual (CLI genera `vantadb.license` firmado por cliente) hasta tener entidad constituida (C2).
- **Acción:** documentado en `docs/strategy/VANTADB-PRO-DELIVERY.md` (§ pagos = manual).
- **Estado:** ✅ COMPLETED

### Task 6: Marca y legal (humano, fuera del pipeline)
- **Pregunta:** ¿registrar marca "VantaDB" (USPTO/EUIPO, $250-350/clase, requiere tarjeta/abogado)?
- **Decisión (2026-08-06):** **C) No hacer nada por ahora** — omite búsqueda y registro; se continúa con el nombre actual.
- **Acción:** solo documental — LEG-01 queda en lista humana (Backlog), no en agentes.
- **Estado:** ✅ COMPLETED

---

## Fase 1 — Auditar features core vs Pro (work no-code, informativo)

> **Nota:** la decisión D4 (A) indica no tocar el core. Esta fase solo **lista** lo que pasará a Pro para documentarlo, sin mover.

### Task 7: AUDIT-PRO — Inventario features core candidatos a Pro
- **Archivos clave:** `Cargo.toml:93-131` (features), `src/` (módulos faecores), `docs/operations/CONFIGURATION.md`, `deny.toml`
- **Gate:** decidir `in/out` por feature con path:línea real de cada gate: `encryption`, `wal-shipping`, `pitr`, `server`, `tls`, `prometheus`. Tabla `| Feature | Código | In core | Pro sí/no |`.
- **Herramientas:** `codegraph_explore`, `rg`, `cargo check`.
- **Deliverable:** `docs/strategy/VANTADB-PRO-FEATURES.md` ✅ en disco.
- **VERIFY (ejecutado 2026-08-06):** `cargo check -p vantadb --no-default-features -F "encryption,pitr,wal-shipping,prometheus,tls,server"` → EXIT 0 (core compila con todos los gates Pro; verificación completada).
- **Nota:** no hace commit de código — solo doc.
- **Estado:** ✅ COMPLETED

---

## FASE 2 — Esqueleto `vantadb-pro` + estructura (F5 code)

### Task 8: Skeleton — Crear esqueleto del repositorio/package Pro (FUERA del workspace)
- **Archivos clave (nuevos):** `vantadb-pro/Cargo.toml` (crate separada, `license = "LicenseRef-Proprietary"`), `vantadb-pro/src/lib.rs`, `vantadb-pro/LICENSE`, `vantadb-pro/README.md`
- **Acción:** crear el crate **fuera** de `members`/`default-members` del workspace core. `license` elegida en D2, descripción "VantaDB Pro — edición comercial", `repository` privado.
- **VERIFICACION (ejecutado):** `cargo check` en `vantadb-pro/` → EXIT 0; tests 4/4. Core intacto (`cargo check -p vantadb` → EXIT 0).
- **Dependencias:** D1, D2.
- **Estado:** ✅ COMPLETED

### Task 9: BUILD-DELIVERY — Script de validación de licencia por nodo
- **Archivos clave (nuevos):** `vantadb-pro/scripts/generate-license.ps1` (emite `vantadb.license` firmado con expiración + límite nodos), `vantadb-pro/src/license.rs` (verify), **Archivo new LICENSE**
- **Acción:** mínimo funcional (ponytail): verificar `vantadb.license` (expiración + max-nodes) offline, sin servidor. Firma HMAC/Ed25519 DIFERIDA (señalizada :ponytail ceiling).
- **VERIFICACION (ejecutado):** tests 4/4 (vacío, futuro, vencido, exceso nodos) + clippy limpio.
- **Dependencias:** D2, estructura (Task 8).
- **Estado:** ✅ COMPLETED

### Task 10: Package aislado — Empaquetar `.whl`/`.crate` pro sin cargo del core
- **Archivos clave:** `vantadb-pro/pyproject.toml` o `maturin` (si SDK py), `.cargo/config` del core
- **Acción:** si Pro tiene binding Python/SDK, build con maturin a arch (wheel) **apunta al repo pro**, no al core. Asegura `deny.toml` del core no exama pro; `Cargo.toml` core excluye `vantage-pro`.
- **VERIFICA:** artifacto se construye independiente; `cargo package` en core no incluye pro.
- **Estado:** ✅ COMPLETED

---

## FASE 3 — Entrega / Go-to-market

### Task 11: DELIVERY-DOC — `docs/strategy/VANTADB-PRO-DELIVERY.md`
- **Archivo:** `docs/strategy/VANTADB-PRO-DELIVERY.md` (nuevo, PENDIENTE de crear)
- **Acción:** describe entrega por tier: Pro (registro privado / binario), Enterprise (artefacto + licencia `vantadb.license`, on-prem). Incluye matriz pagos según D5.
- **VERIFICACIÓN:** tabla de entrega por tier; lint markdown.
- **Dependencias:** D3, D5.
- **Estado:** ✅ COMPLETED

### Task 12: GTM-PRICING — Actualizar tier plan con features Pro reales
- **Files:** `docs/strategy/VANTADB-PRO-FEATURES.md` (fuente de features)
- **Acción:** alinear Community/Pro/Business/Enterprise (C7) con las features de la tabla; no inventar cifras (reusar valores de C7).
- **VERIFICACIÓN:** consistente con `VANTADB-PRO-FEATURES.md`; nada inventado.
- **Estado:** ✅ COMPLETED

---

## FASE 4 — Reglas nuevas, AGENTS.md, docs

### Task 13: RULES-NEW — Crear núcleos `license-open-core.md` en `.opencode/rules/`
- **Archivo:** `.opencode/rules/open-core-licensing.md` (nuevo)
- **Acción:** regla MUST/MUST-NOT: (1) features pro NO pueden entrar al core Apache; (2) `vantadb-pro` fuera del workspace; (3) core sigue MIT/Apache-persistente (deny.toml); (4) licencia NaCl/firma must-not-embeber-secretos; (5) entregas `node. license` validated. Formato según `.opencode/rules/README.md`.
- **VERIFY:** `markdownlint` (sin error); el archivo cumplo la plantillacabecera.
- **Estado:** ✅ COMPLETED

### Task 14: AGENTS-UPDATE — Documentar flujo Open Core en AGENTS.md
- **Archivo:** `.opencode/AGENTS.md`
- **Acción:** nueva sección "Open Core Licensing" con: core Apache, pro propietario, dónde vive, cómo se entrega, regla de separación. Actualizar `Key Conventions` → licensing (hoy `deny.toml | MIT/Apache-2.0` → explicitar que pro NO it maver).
- **VERIFICACIÓN:** sección legible; `deny.toml` no menciona BSL/pro.
- **Estado:** ✅ COMPLETED

### Task 15: DOCS-NEW — Actualizar índice de `docs/strategy`
- **Acción:** enlazar `docs/strategy/VANTADB-PRO-DELIVERY.md` + `VANTADB-PRO-FEATURES.md` en el índice/docs README de strategy si existe. Docs en inglés (Regla 3).
- **Estado:** ✅ COMPLETED

---

## FASE 5 — Documentación post-cambio

### Task 16: ADR — Registro Licencia Open Core VantDB Pro
- **Archivo:** `docs/architecture/adr/<NNN>-open-core-vantadb-pro.md` (nuevo, número sgte.)
- **Acción:** ADR con la decisión (correlación, licencia core, dónde pro, entrega) según template `docs/_templates/adr.md`. Cierra el ciclo "decidí".
- **Estado:** ✅ COMPLETED

---

## FASE 7 — Cierre

### Task 17: VERIFY-FULL — Pre-push gate
- **Comando:** `dev-tools/verify.ps1` (fmt + clippy + test + deny). Core intacto, sin tocar features (`D4A`).
- **Estado:** 🟡 PARCIAL — `cargo check -p vantadb` (default + features Pro) EXIT 0 corrido en 2026-08-06. Falta fmt/clippy/nextest/deny full (verify.ps1) para el commit.
- **Nota:** el core no fue tocado en esta sesión; la verificación full solo es un sello previo a push.

### Task 18: COMMIT + progreso
- **Acción:** commit convencional de los docs del core (docs: open-core) + `skill progreso`. No tocar version (Regla 7).
- **Estado:** ⏳ PENDING (se ejecuta en esta pasada)

---

## Casos y ramificaciones (qué hacer según cada escenario)

Mapa de decisiones condicionales para que el agente sepa qué ejecutar cuando el usuario elige; y qué hacer si falla algo.

### C1 — Licencia core (D1)
| Si elige | Qué hacer |
|---|---|
| **Apache-2.0** (elegida) | Nada al core; mantener `LICENSE`/`Cargo.toml:7`. ✅ hecho |
| BSL motor | Refactor completo + relicenciar: ABORTAR, requiere plan nuevo (no contemplado). |
| AGPL | Relicenciar core + renegociar contribuidores (no contemplado, alto riesgo). |

### C2. Licencia Pro (D2)
| Elegida | Acción |
|---|---|
| **Propietaria estricta** (elegida) | `vantadb-pro/LICENSE` "All rights reserved" + comercial por nodo. ✅ hecho |
| BSL 1.1 | Reemplazar LICENSE por BSL con Additional Use Grant + fijar fecha; cambiar `license` a `BUSL-1.1`. |
| Dual | Mantener core Apache + Pro propietaria (equivale a A). |

### C3. Entrega (D3)
| Elegida | Acción |
|---|---|
| **Repo privado + artefactos** | `ness-e/vantadb-pro`. ✅ hecho |
| Solo artefactos firmados | El commit source nunca sale del repo local; generar `.tar.gz`/`.whl` con script de release. |
| Registry privado cargo/pip | Configurar token por cliente (futuro). |

### C4. Features Pro (D4)
| Elegida | Acción |
|---|---|
| **`Features NUEVAS`** (elegida) | Los gates existentes del core NO se mueven; Pro son features nuevas escritas aparte. |
| Mover gates existentes | Refactor: cortar `encryption`/`wal-shipping`/`pitr`/`prometheus`/`server`/`tls` del core → `vantadb-pro`, rompe usuarios, requiere transición |
| Mezcla | Elegir por feature según moat; documentar en VANTADB-PRO-FEATURES.md |

### C5. Comercial (D5 — sin decidir)
| Elección | Acción |
|---|---|
| Merchant of Record (Polar/Paddle) | Setup de cuenta + checkout links; nada de code. |
| Wise/Payoneer (facturación) | Documentar en DELIVERY; cobro manual/anual. |
| Gumroad (on-prem) | Licencia + pago con Gumroad; descarga firmada. |
| Aplazar | Entregar Enterprise manual (CLI de generación de licencia) hasta la entidad. |

### C6. Marca (D6 — sin decidir)
| Elección | Acción |
|---|---|
| Registrar USPTO/EUIPO | Humana/costosa; fuera del pipeline. No lo ejecuta ningún agente. |
| Buscar y diferir | El agente documenta búsqueda de conflicto; agrega recordatorio. |
| No hacer | Nada. |

### C7. Fallbacks de ejecución
| Evento | Qué hacer |
|---|---|
| Repo privado GitHub no disponible | Usar la carpeta local `vantadb-pro/` como source; entregar por artefacto firmado. |
| `cargo check` del core falla tras editar pro | El core NO debe tocar pro; sospechar mió bump no relacionado, `git bisect`. |
| Cliente exige ver el source de Pro | Novender; ofrecer escrow (source en gesto tercero) — decisión comercial humana. |
| `deny.toml` del core marca LicenciaRef-Proprietary | Excluir `vantadb-pro` del `deny` (no es dependencia del workspace). |

---

## Dependencias clave (grafo)

```
D1,D2 → Task 8 (licencias del producto/esqueleto)
D4 → Task 7 (audit features) → Task 9 (license module) → Task 11 (delivery)
D3 → Task 11 (Delivery doc)
Task 8,9,10 → Task 13 (rules) → 14 (AGENTS) → 16 (ADR)
Phase 4 → Phase 5 → 17 (verify) → 18 (commit)
```

## Secuencia recomendada

1. **Fase 0**: responder D1-D6 / por el agente preguntando al usuario (no autonominar).
2. Fase 1: audit features (solo doc, no code).
3. Fase 2: scaffold pro + license + packaging.
4. Fase 3: delivery doc + GTM.
5. Fase 4: reglas + AGENTS + docs.
6. Fase 5: ADR.
7. Fase 6: verify + commit.

## Probes de integridad (antes de cada tarea)

- [ ] Fase 0 jugada (todas las decisiones registradas en `campaign_memory_write(decisions)`)
- [ ] Git limpio (o cambios del pipeline)
- [ ] `just verify-quick` en cambios de código; `verify.ps1` antes de push
- [ ] `vantadb-pro` NO está en `members` del workspace (`Cargo.toml:590-603`)
- [ ] Características Pro no arrastran `.cargo/config.toml` del core
- [ ] Nada de `vantadb-pro` empaquetado por el core (`deny.toml`/`cargo package`)

## Notas

- Partimos de la premisa de investigación verificada de internet (Surreal/legal sobre BSL/AGPL/open core) — ver sección **Contexto** arriba.
- La selección de features Pro (D4) NO debe modificar el core actual sin decisión explícita de break (opción B). Default recomendado: **A (features nuevas)**, cero riesgo.
- Pro pipeline, `ENC`/WAL x no se tocan a no ser que el usuario elija D4B.

=== RECITATION ===
Objetivo activo: 
=== END RECITATION ===