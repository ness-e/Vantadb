---
title: "FIND-795: la lista se partía por comas y un texto con coma se verificaba por un fragmento suelto"
kind: task
status: completed
description: "`-ExpectPresent 'a,b'` se parseaba con `-split ','` sobre CADA elemento, incluido el contenido: `-ExpectAbsent '...verbatim, 1428 chars'` se partió y el fragmento `verbatim`, que existe en otra línea, decidió el resultado. Ahora el array (`@('a','b')`) no se parte nunca, y la forma con comas solo si TODAS las partes son `<path>::`; si no, se verifica UN literal."
---

# FIND-795: un separador que aparece en el contenido no puede ser la lista

## Metadata

- **Fuente:** `docs/dev/Backlog.md` §`FIND-795` (fila retirada al cerrar)
- **Esfuerzo:** 🟢 1-2h → real ~1h (comparte el parseo con `FIND-793/796`)
- **Prioridad:** 🟠 Alta
- **Tipo:** bug-fix · dev-tools
- **Creado:** 2026-11-26 · **Cerrado:** 2026-11-27
- **Estado:** ✅ implementado, verificado y commiteado

## Impacto mapeado (Regla 0)

- **Leído completo:** `dev-tools/git-commit-isolated.ps1` (`param()` y el bloque de
  normalización de `-Paths`/`-Expect*`), `dev-tools/tests/git-commit-isolated.tests.ps1`,
  `docs/dev/workflow/RULES.md` §11 (el ejemplo con `-ExpectAbsent docs/dev/Backlog.md::FIND-760`).
- **Referencias hacia dentro:** `docs/dev/plans/2026-11-26-batch-parciales-y-nuevas.md:67`
  enseña la forma `-ExpectAbsent '<path>::| \`<ID>\` |'` (sin comas). El plan no se toca
  (fuera de alcance), pero la forma sigue funcionando: no tiene comas.
- **Referencias hacia fuera:** ninguna nueva. Se conserva el `-split` de `-Paths`
  (mismo problema de `pwsh -File`, y un path con coma es raro y no tiene contenido libre).
- **Veredicto:** cambio contenido al parseo; cualquier invocación existente con
  specs bien formados sigue funcionando, y la que llevaba una coma en el contenido
  deja de pasar por el fragmento.

## Diagnóstico (medido)

Al corregir `FIND-789` se declaró `-ExpectAbsent '...verbatim, 1428 chars'`. El
helper la partió por la coma:

```
$ExpectAbsent = @($ExpectAbsent | ForEach-Object { $_ -split ',' } | Where-Object { $_ })
```

El fragmento `verbatim` existe legítimamente en otra línea del repo. El resultado
fue `BAD EXPECTATION` + `REVERTED`. **Ese desenlace es el bueno; el peligroso es
el simétrico:** si el fragmento hubiera existido y el segundo trozo hubiera tenido
`::`, la expectativa habría pasado **sin verificar la frase declarada** y el commit
se habría dado por verificado por contenido. Es la misma clase que `FIND-794`/
`FIND-785` — el gate midiendo otra cosa que lo que dice medir — con el disparador
en un **carácter del contenido**: cualquier fecha, lista o nombre con sufijo cae.

## Lo que se implementó

`Resolve-ExpectationList` decide por argumento, no por elemento:

| Entrada | Resultado |
|---|---|
| `@('a::x','b::y')` (array, 2 elementos) | dos specs; **ningún elemento se parte** |
| `@('a::x, y')` (array, 1 elemento con coma) | **un** spec: la coma es contenido; NOTE |
| `'a::x,b::y'` (string con comas, todas con `<path>::`) | dos specs (compatibilidad) |
| `'a::x, texto suelto'` (no todas con `<path>::`) | **un** spec literal; NOTE |

Además, una spec sin `::` se rechaza **antes** de cualquier trabajo de git
(`exit 2`, nada creado): un commit creado con una expectativa injuzgable es la
forma exacta de `FIND-793`/`FIND-796` (exit de fallo con el commit correcto en
`HEAD`).

Pre-mortem (b) respetada: `-ExpectPresent 'a,b'` **sigue funcionando** cuando
`a` y `b` son specs completas — verificado en el test y en el escenario 19b, que
usa la forma de un solo spec con backticks y barras.

## Verificación

- Escenario 16 (array, coma en el contenido): el helper **nuevo** devuelve exit 0,
  imprime `ok present` para la frase entera, no imprime `BAD EXPECTATION`, y
  comprueba identidad (`git rev-parse HEAD:<path>` == `git hash-object <path>`).
- Escenario 16b (string con coma no-spec): exit 1 **sin** `ok present`; el helper
  nuevo dice `ONE literal`. El de `HEAD` imprime `ok present: … :: verificacion`
  (pasó por el fragmento) → **4 FAILED** contra el viejo.
- Suite completa contra el helper de `HEAD`: **33 FAILED**, todos en 16-20;
  escenarios 1-15 verdes. Suite contra el nuevo: **131/131 PASS**.

## Review P2-01

Tier **Fast** (`dev-tools/**` + `docs/dev/**`). Una ronda para las cuatro filas:
ver `docs/dev/tasks/FIND-792.md` §Review.

## Deuda / techo declarado

- Una spec cuyo **path** contenga `::` no es expresable (mismo `-split '::', 2`
  de siempre). En este repo los paths son relativos y no llevan `::`.
- La forma con comas sigue siendo ambigua por naturaleza (una lista `a::x,b::y` y
  un contenido `x,b::y` son indistinguibles **antes** de mirar el blob). La regla
  elegida — partir solo si todas las partes son specs — nunca produce un falso
  verde; en el peor caso verifica de más un literal y falla como contenido.
- `-Paths` sigue partiéndose por coma (compatibilidad con `verify.ps1` y con
  `pwsh -File`); un path con coma no es soportado y no lo era antes.
