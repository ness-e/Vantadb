# Plan: gaps pendientes del harness `.opencode/` (handoff entre chats)

> **Campaign ID:** 7d276566-4a14-4761-bd19-048c1cc5a4fb
> **Inicio:** 2026-09-24
> **Estado:** ⬜ PENDING (4 tareas, independientes — ejecutables en cualquier orden o en paralelo)
> **Fuente:** auditoría harness 2026-09-24 (`dda76a4` + `ceca4d7` en repo `configOpencode`)
> **Backlog:** FIND-148..151 (`docs/dev/Backlog.md` § Hallazgos pendientes de reportes)
> **Cómo usar en otro chat:** `/pipeline task FIND-14X` por tarea, o leer la tarea y ejecutar sus Acciones.
> Todo lo ya aplicado vive en `dda76a4` (fixes F1–F9, opts O1–O8, `/harness`, `vanta-harness`).

## Resumen

| Tarea | Qué | Esfuerzo | Se ejecuta en |
|---|---|---|---|
| FIND-148 | Frontmatter v2 + descriptions trigger (11 agents) | 🟡 | repo `configOpencode` |
| FIND-149 | Semgrep OSS + MCP + ast-grep en L7 (static analysis) | 🟡 | ambos repos |
| FIND-150 | Higiene dependencias: OSV-Scanner + machete/udeps + llvm-cov (DoD v2) | 🟢 | repo VantaDB (CI host) |
| FIND-151 | Eval de agentes (Giskard) + Lurkr obligatorio en CI host | 🟡 | repo VantaDB (CI host) |

## FIND-148 — Frontmatter v2 + descriptions trigger

**Contexto:** los 11 `agents/vanta-*.md` usan vocabulario de permisos v1 (`bash:`,
`task:` — ver `agents/vanta-lead.md:9-35`). La doc OpenCode v2
(https://v2.opencode.ai/docs/agents) recomienda `shell` (antes `bash`), `edit`
(cubre write/patch) y `subagent` (hijos), con regla "broad wildcard first,
exceptions after". Además, Claude Code delega por `description`
(https://code.claude.com/docs/en/subagents.md): las nuestras son descriptivas en
tercera persona; el modelo delega mejor con triggers en primera persona
("Use me when…"). Modelo a imitar: `agents/vanta-review.md:3-10`.
**Por qué no se hizo:** migrar 11 frontmatters a ciegas puede romper el mapeo del
OpenCode instalado; requiere prueba en 1 agente primero.

**Tarea:** migrar 1 agente piloto (`vanta-review`), verificar que OpenCode lo
acepta, luego los 10 restantes + reescritura de descriptions.

**Acciones:**
1. `git log --oneline -3` en `.opencode/` (confirmar base `ceca4d7`).
2. En `agents/vanta-review.md` frontmatter: renombrar `bash:` → `shell:`,
   `task:` → `subagent:` (mantener comentarios `# TSYS11:` y valores).
3. Reiniciar OpenCode, pedir al modelo listar subagentes (debe aparecer `vanta-review`).
4. Si OK: repetir en los 10 restantes (un commit por rol o uno global).
5. Reescribir cada `description:` a formato trigger ("Use me when X… Never use me for Y…"),
   ≤1024 chars (límite OpenCode), sin cambiar el resto del system prompt.
6. Actualizar tabla de límites en `.opencode/AGENTS.md` (nombres de columnas `Bash`→`Shell`, `task`→`subagent`).

**Pruebas (contrato):** OpenCode arranca sin warnings de permisos + `vanta-review`
visible y delegable por nombre + `grep -rn "bash:\|task:" agents/` vacío +
sesión de prueba: pedir "revisa este plan con vanta-review" rutea solo.
**Riesgos:** si v2 no mapea v1, rollback del piloto (`git revert`). No tocar `mode:` ni `model:`.

## FIND-149 — Semgrep OSS + MCP + ast-grep (L7 static analysis)

**Contexto:** L7 Security hoy es `cargo audit/deny` + review manual. Semgrep OSS
(Community Edition, https://github.com/semgrep/semgrep) aporta SAST 30+ lenguajes
con reglas como código, y su MCP server (`semgrep mcp`) deja a `vanta-audit`
correr scans directos. ast-grep (MIT, https://ast-grep.github.io) aporta búsqueda
estructural versionable ("unwrap en prod", "unsafe sin SAFETY") mejor que regex.
**Estado:** ninguno instalado; `vanta-audit.md` no los referencia.

**Acciones:**
1. `pip install semgrep` (o binario) + `semgrep --version`; `cargo install ast-grep` (o binario).
2. Crear `.opencode/configs/semgrep-vanta.yml` (reglas propias: `unwrap()` en
   `src/**` fuera de tests, `unsafe` sin `// SAFETY:` en ventana ±5 líneas,
   `expect(` con mensaje genérico) + `p/rust`, `p/python`, `p/typescript` defaults.
3. Probar: `semgrep --config .opencode/configs/semgrep-vanta.yml --error` en raíz host (contrato: exit 0 en main).
4. Registrar Semgrep MCP en `opencode.jsonc` del host ( Disabled default, activar por perfil).
5. `agents/vanta-audit.md` §Technical Constraints: añadir pasos semgrep/ast-grep a L7.
6. `commands/harness.md` §Notas: documentar comandos + dónde enganchan.

**Pruebas:** `semgrep` exit 0 en `main` + 1 regla que falle a propósito en rama test
y pase tras fix + `vanta-audit` cita output de semgrep en un dictamen real.
**Riesgos:** falsos positivos iniciales → empezar `warn`, endurecer a `error` tras 1 semana verde.

## FIND-150 — Higiene dependencias + cobertura DoD v2

**Contexto:** DoD v2 (`references/definition-of-done.md:125-130`) exige 70%
cobertura en módulos nuevos + security checklist obligatorio, pero no hay
herramienta que lo mida. Deuda P2-5/P2-8 pide detectar deps no usadas.
**Estado:** `cargo nextest` en uso; sin `cargo-llvm-cov`, sin `cargo-machete/udeps`, sin OSV-Scanner.

**Acciones:**
1. `cargo install cargo-llvm-cov cargo-machete` (+ `cargo-udeps` requiere nightly: evaluar).
2. `cargo llvm-cov nextest --workspace --fail-under-lines 70` en 1 crate piloto (`vantadb`); registrar baseline.
3. `cargo machete` en workspace; eliminar o justificar cada dependencia no usada (Regla 6: saldo neto ≤0).
4. OSV-Scanner (Apache-2.0, https://google.github.io/osv-scanner): `osv-scanner ./...` en CI junto a `cargo audit` (licencias siguen en `deny.toml`).
5. Cablear los 3 en Fast Gate / workflow CI del host + documentar en DoD v2 (quitar "pendiente de herramienta").

**Pruebas:** coverage ≥70% en crate piloto + `cargo machete` 0 sin justificar + OSV exit 0 + CI verde.
**Riesgos:** `cargo-udeps` nightly puede romper el gate — mantenerlo informativo hasta estable.

## FIND-151 — Eval de agentes (Giskard) + Lurkr obligatorio

**Contexto:** no hay eval de los propios agentes (alucinación/robustez de
`vanta-research`, calidad de digest). Giskard OSS (Apache-2.0,
https://www.giskard.ai) es la opción abierta. Lurkr ya está citado como Rule 17
informativa (`task-system/RULES.md:398-418`) pero nunca se volvió obligatorio.
**Estado:** ninguno implementado; CI del host fuera de este repo.

**Acciones:**
1. `pip install giskard`; crear `evals/agent/` en host con 10 casos dorados
   (digest research ≤500 palabras + URLs verificadas; veredicto review con evidencia).
2. Gate: `/audit full` L9 corre la eval; regresión = NO-GO.
3. Lurkr: `npx lurkr scan .opencode/ --report-format json` en CI host como job
   informativo 2 semanas → obligatorio (falla el build) si 0 falsos positivos.
4. Documentar ambos en `commands/harness.md` + `question-gates.md` Gate H.

**Pruebas:** eval 10/10 en main + Lurkr exit 0 en `.opencode/` + CI verde 2 semanas.
**Riesgos:** Giskard LLM-judge cuesta tokens — 10 casos, solo en `full`, nunca en quick.
