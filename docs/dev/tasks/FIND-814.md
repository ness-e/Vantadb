---
title: "FIND-814 - `tests/` allowlisteado entero: un secreto real ahi era invisible para el Secret Scan"
kind: task
status: completed
description: "El allowlist de directorio de `.gitleaks.toml` (y el `tests/**` de `.secretlintignore`, que alimenta el MISMO job) dejaban `tests/` entero fuera del escaneo. Se reemplazan por huellas de linea, y se prueba en las dos direcciones con gitleaks 8.24.3 (la version del job)."
tags: [vantadb, ci, gitleaks, secretlint, allowlist, secrets]
---

# FIND-814 - un directorio entero fuera del escaneo de secretos

## Contexto

`.gitleaks.toml` allowlisteaba `(^|/)tests/` como **directorio**, y el mismo job
(`Secret Scan (gitleaks + secretlint)`, `ci-rust.yml:204`) corre **dos** escaneres:
el segundo (`secretlint`) tenia `tests/**` y `**/tests/**` en `.secretlintignore`.
O sea que un secreto real dentro de `tests/` era invisible para **ambos**.

Lo midio el carril del `fail-path` de `FIND-787` al buscar donde poner una credencial
de prueba: un probe en `tests/` no habria disparado nada, y el experimento habria
concluido "el guard no funciona" cuando el problema era que el escaneo no miraba.

## Contrato

Reemplazar el allowlist de **directorio** por allowlists **por fichero o por huella**;
los fixtures que necesiten una credencial falsa la declaran **en su linea**
(`gitleaks:allow` para gitleaks, `secretlint-disable-line` para secretlint). El resto
de `tests/` vuelve a estar escaneado.

## Impacto mapeado (Regla 0)

**Leidos completos:** `.gitleaks.toml`, `.secretlintignore`, `.secretlintrc.json`, el
job `secret-scan` (`ci-rust.yml:204-232`), y la fila `FIND-786` (el mismo patron con
`docs/`, declarado como deuda alli).

**Referencias hacia dentro:** `ci-rust.yml:224` pasa `GITLEAKS_CONFIG: .gitleaks.toml`;
`ci-rust.yml:231-232` invoca `secretlint "**/*"` (que lee `.secretlintignore` del cwd).
Ningun otro workflow usa esos dos ficheros (`Select-String` sobre `.github/workflows`).

**Referencias hacia fuera:** ninguna. Los dos ficheros son configuracion pura.

**Veredicto:** cambio acotado a dos ficheros de configuracion, sin codigo, sin cambios
en el job. El riesgo real no es romper: es **aflojar** (agregar una huella de mas y
tapar un secreto futuro) o **apretar de mas** y poner rojo el `Secret Scan` en cada PR.
Las dos direcciones se miden abajo.

## Spec - decisiones

| # | Decisión | Elegido | Alternativa descartada | Por qué |
|---|---|---|---|---|
| D1 | Que reemplaza al directorio | **huellas de linea** (una `regexes` por valor sintetico) | allowlist **por fichero** | el fichero entero sigue escaneado: otra credencial que entre en `security_audit.rs` o `prx07_redact.rs` dispara. Un `paths` por fichero taparia eso. |
| D2 | Donde vive la huella | `.gitleaks.toml` (`regexes`, `regexTarget="line"`) | `gitleaks:allow` en la linea del fixture | `gitleaks:allow` es preferible y esta documentado en el fichero, pero los dos fixtures viven en `.rs` de `tests/`, **fuera de la zona de escritura de este lote**. La huella consigue lo mismo desde el fichero de configuracion. |
| D3 | `.secretlintignore` | **se retiran `tests/**`, `**/tests/**` y `evals/fixtures/**`** | dejarlo | es el **mismo job**: arreglar un escaner y dejar el otro ciego es la mitad de un fix. `evals/fixtures/**` ni siquiera existe (0 ficheros trackeados): era una exencion preventiva a un directorio vacio. |
| D4 | `docs/` | **no se toca** | retirarlo tambien | es la deuda declarada de `FIND-786` (docs cita ejemplos), otra fila y otro riesgo medido. Fuera de alcance. |
| D5 | Como se prueba el "no dispara" | `gitleaks:allow` **en la misma linea** del probe | confiar en la doc | una prueba que no prueba el mecanismo de excepcion no prueba nada. |

## Verificacion — las dos direcciones, medidas

Todas con **gitleaks 8.24.3**, la version del job (no la ultima), y el probe
`AKIAZ7Q2XK4M9PLR3TUV`. **Trampa medida:** `AKIAIOSFODNN7EXAMPLE` **no** dispara
(`EXAMPLE` es stopword del config por defecto de gitleaks) - con esa cadena se
concluye lo contrario de lo que pasa.

### Antes: el agujero, con el config viejo (`HEAD`)

```text
$ echo 'AKIAZ7Q2XK4M9PLR3TUV' > tests/__gl_probe.txt
$ gitleaks dir tests --config .gitleaks.toml
INF no leaks found                                                  exit=0
```

### Despues: las dos direcciones

```text
$ gitleaks dir tests --config .gitleaks.toml            # tests/ limpio
INF no leaks found                                                  exit=0

$ echo 'AKIAZ7Q2XK4M9PLR3TUV' > tests/__gl_probe.txt
$ gitleaks dir tests --config .gitleaks.toml
WRN leaks found: 1                                                  exit=1

$ echo 'AKIAZ7Q2XK4M9PLR3TUV # gitleaks:allow' > tests/__gl_probe.txt
$ gitleaks dir tests --config .gitleaks.toml
INF no leaks found                                                  exit=0

$ echo 'AKIAZ7Q2XK4M9PLR3TUV' > src/__gl_probe.txt   # control: el resto del repo
$ gitleaks dir src --config .gitleaks.toml
WRN leaks found: 1                                                  exit=1
```

### El modo `git` (el que corre CI), sobre historia real

```text
# el probe de produccion del fail-path de FIND-787, que SI estaba en una ruta escaneada
$ gitleaks git . --config .gitleaks.toml --log-opts="de8127334~1..de8127334"
WRN leaks found: 1                                                  exit=1
$ gitleaks git . --config <config viejo> --log-opts="de8127334~1..de8127334"
WRN leaks found: 1                                                  exit=1        # control

# el ultimo commit que toco un fixture con credencial falsa dentro de tests/
$ gitleaks git . --config <config viejo> --log-opts="db6ad7f79~1..db6ad7f79"
INF no leaks found                                                  exit=0   <-- EL AGUJERO
$ gitleaks git . --config .gitleaks.toml --log-opts="db6ad7f79~1..db6ad7f79"
WRN leaks found: 1                                                  exit=1
```

El segundo par es la prueba que importa: **el mismo commit, el mismo rango, el mismo
binario**, verde con el config viejo y rojo con el nuevo. La diferencia es solo el
allowlist de directorio.

### Lo que aparecio al retirarlo, y su triage (historial completo)

```text
$ gitleaks git . --config .gitleaks.toml --log-opts="--all"
INF 8728 commits scanned.
INF scanned ~554656984 bytes (554.66 MB) in 3m56s
WRN leaks found: 6                                                  exit=1
```

Los **6** son **una sola linea** en **6 versiones del mismo fichero**:

```text
vanta-proxy/tests/prx07_redact.rs:57  github-pat  (x6 commits: db6ad7f79, 19f759980,
                                                    06f8f84f8, cd22b55a4, 4d06ac810, d4536d895)
```

`"ghp_0123456789abcdef0123456789abcdef0123"` - secuencia hexadecimal, uno de los dos
tokens falsos contra los que `generic_tokens_masked` prueba que el `Redactor` los
enmascara. **Fixture, no credencial** -> huella de linea. (El otro fixture,
`tests/security/security_audit.rs:123`, salio en el barrido `dir` antes de anadir su
huella y ya no aparece.)

Despues de las dos huellas, el barrido completo del historial vuelve a **0**:

```text
$ gitleaks git . --config .gitleaks.toml --log-opts="--all"
INF 8732 commits scanned.
INF scanned ~554703713 bytes (554.70 MB) in 2m28s
INF no leaks found                                                   exit=0
```

(8732 vs 8728: el barrido final corrio despues de que el lote creara los commits de
`FIND-787` y de este mismo fichero.)

### Los dos escaneres del job

```text
$ npx --yes -p secretlint -p @secretlint/secretlint-rule-preset-recommend secretlint "tests/**/*"
(exit 0, 9.1 s)          # con el `.secretlintignore` nuevo: tests/ ya no esta exento
```

## Deuda

- **`docs/` sigue allowlisteado entero** en los dos ficheros (`.gitleaks.toml` y
  `.secretlintignore`). Es la fila `FIND-786`, con su propio riesgo medido (docs cita
  ejemplos de credenciales): no se toca aca.
- **Techo de la huella:** silencia la **linea exacta**, no el fichero, asi que una
  credencial nueva en ese mismo fichero dispara. Lo que no hace es distinguir un tercer
  valor sintetico que alguien agregue a `prx07_redact.rs`: ese disparara y pedira su
  propia huella o su `gitleaks:allow`. Es el comportamiento buscado (explicito y
  auditable), no un defecto.
- **`secretlint` no se probo en modo "debe disparar"**: no hay un fixture con una
  credencial que el preset reconozca y que gitleaks no, asi que la direccion "dispara"
  se probo solo con gitleaks. La direccion "no dispara de mas" si esta medida (exit 0
  sobre `tests/` completo, 9.1 s).
- **Sin compilar Rust** (dos carriles vivos): nada de este cambio lo necesita.

## Review

**P2-01 - `vanta-review`, contexto fresco, una ronda.** Veredicto y hallazgos al final
de la fila (ver `RESULTADO` del lote); lo aplicado esta en el diff de `.gitleaks.toml` y
`.secretlintignore`.

## NOTICED BUT NOT TOUCHING

- `.gitleaks.toml` allowlistea tambien `(^|/)scripts/docs/` y `(^|/)\.opencode/`. El
  primero son scripts del generador de docs (mismo genero que `docs/`, deuda
  `FIND-786`); el segundo es un repo git separado que no puede volver al historial
  (declarado en el propio fichero). Ninguno de los dos se midio aca.
- `secretlint` corre con `npx --yes` sobre `**/*` en cada PR: la lista de extensiones
  que escanea la decide el preset, no este carril.
