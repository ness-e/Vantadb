# VantaDB — Bug Workflow

> **Cómo usar:** Al reportar un bug o test failure, sigue esta secuencia. NO intentes fixes sin diagnóstico (salvo Fase 0.5: contener primero si el entorno está roto).
> **Cómo editar:** Modifica los pasos si encuentras un patrón nuevo que debería estar documentado.
> **Referencia desde:** `.opencode/AGENTS.md` — sección Bug Workflow Reference.

---

## Fases

### Fase 0: Diagnosticar el Error

1. Leer el mensaje de error COMPLETO — incluye stack trace, exit code, output capturado
2. Reproducir: `cargo nextest run --profile audit -p <crate> --test <test_name>` — ¿falla siempre?
3. Si no es reproducible, buscar patrón: ¿cuándo falla? (CI sí, local no; Windows sí, Linux no)
4. Buscar en `docs/references/troubleshooting.md` si el error ya está documentado

### Fase 0.5: Contención/Estabilización (solo si el bug rompe el entorno)

> Fuente: `docs/Investigaciones/2026-08-10-agent-engineering/eng-02-systems.md:209-214,397-400` (mitigar primero, root-cause después).
> Decisión TIR-03 (2026-08-12).

**Disparador:** el bug rompe el build (`cargo check`/`clippy`/`nextest` falla en masa),
rompe CI, o afecta un flujo activo del pipeline (test suite en rojo, backoff).

1. Estabilizar ANTES de debuggear: revert del último commit sospechoso (`git revert`)
   o pausar el plan actual y aislar el cambio que lo rompió.
2. Registrar el incidente (qué se rompió, timeline) aunque todavía no haya causa.
3. Recién con el sistema estable, entrar a Fase 1 (Iron Law).

No reemplaza el Iron Law: la contención estabiliza el entorno; el RCA y el fix
siguen siendo obligatorios (Fase 1 → Fase 2).

### Fase 1: Aislar Causa Raíz

Cargar `systematic-debugging` y seguir sus 4 fases:

1. **Root Cause Investigation**:
   - Trazar flujo de datos desde el error hacia atrás (origen del valor malo)
   - Ver git diff reciente: `git diff HEAD~5` — ¿qué cambió?
   - Si el error está en una capa (test → API → storage), instrumentar cada frontera con logs
2. **Pattern Analysis**: Buscar código similar que sí funciona y comparar
3. **Hypothesis**: Una hipótesis clara a la vez. Test mínimo para validarla.
4. **Implementation**: Test fallido → fix → test pasa

### Fase 2: Fix y Verificación

```bash
# 1. Escribir test que reproduzca el bug (debe fallar primero)
cargo nextest run --profile audit -p <crate> --test <test_name>
# Ver: FAIL

# 2. Implementar fix
# (editar código)

# 3. Verificar fix
cargo nextest run --profile audit -p <crate> --test <test_name>
# Ver: PASS

# 4. Verificar que no se rompió nada más
cargo nextest run --profile audit -p <crate>
```

### Fase 3: Commit

```bash
git add -p
git commit -m "fix(<scope>): <descripción corta>

<explicación de causa raíz y solución>"
```

Si el fix tiene más de 100 líneas, dividir en commits atómicos.

---

## Reglas

- **NO** fixes sin root cause — si no sabes qué lo causa, no lo arregles
- **NO** fixes múltiples a la vez — un cambio, una verificación
- **NO** refactorizar "while I'm here" — eso va en otro PR
- **SI** después de 3 intentos de fix el bug sigue: para y cuestiona la arquitectura
- **SIEMPRE** correr `just verify` (o `just verify-quick`) antes de commit final

## Cuándo Escalar

| Señal | Acción |
|-------|--------|
| Test pasa en CI pero no local | Revisar diferencias de entorno (Windows vs Linux) |
| Test pasa local pero no en CI | CI usa `--build-jobs 2` y `ci-windows` profile |
| Error solo en Windows | Buscar en troubleshooting.md primero |
| 3+ fixes fallidos | Cuestionar arquitectura, no intentar fix #4 |
