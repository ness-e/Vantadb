---
title: "`gate-docs-secrets.yml` — GATE: Docs — Secret leak"
kind: runbook
status: active
description: "Escanea la prosa de docs/ (1651 ficheros, incluido README.md) buscando formas de"
tags: [vantadb, ci, gate-docs, security, documentation]
related: [.github/workflows/gate-docs-secrets.yml, RULES.md, gate-docs-links.md, gate-docs-21.md]
---

# `gate-docs-secrets.yml` — GATE: Docs — Secret leak

## ¿Qué hace?

Escanea la **prosa** de `docs/` (1651 ficheros, incluido `README.md`) buscando formas de
credencial, y falla el PR si aparecen por encima del presupuesto de su severidad.

> **Por qué existe:** en abril de 2025 (aviso CISA) un repositorio privado se hizo público
> por error y atacantes rascaron credenciales filtradas en horas. Este repo publica `docs/`
> como repositorio navegable, y una pasada de documentación escrita por agente acaba de
> reescribir los 1648 ficheros. El objetivo no es "el entorno está limpio" sino que
> **"se escribió un secreto en un documento" falle CI en vez de descubrirse después**.

| Job | Comprueba | Script | Gate |
|---|---|---|---|
| `docs-secrets` | Las reglas siguen detectando sus propias formas | `check-secrets.mjs --self-test` | sí |
| `docs-secrets` | Sin credencial en la prosa, dentro de presupuesto | `check-secrets.mjs` | sí, sobre presupuesto |
| `docs-secrets-json` | El mismo escaneo, como agregado legible por máquina | `check-secrets.mjs --json` | sí |

## Estado medido 2026-09-29

| Métrica | Valor |
|---|---|
| Ficheros escaneados | **1651** (1650 en `docs/` + `README.md`) |
| **Errores** | **0** (presupuesto 0) |
| Avisos | 28 (presupuesto 28) |
| Self-test | **29/29** aserciones |

Los 28 avisos son todos `internal-hostname` en 9 notas de investigación, y son
**seudónimos de anonimización, no infraestructura real**: `docs.lan` (13), `blog.lan` (4),
`www.lan` (3), `info.lan` (3), `reference.lan` (1), `smith.lan` (2), `ipc.local` (2 — el
endpoint IPC del desktop propuesto en la investigación de quickwins).

Se reportan en vez de suprimirse porque **el recuento es lo que merece vigilar**: si una
nota futura nombra un host que no sea uno de esos siete, es una divulgación real, y que el
número suba es la señal. Bájalo a medida que esas notas se redacten.

## Sólo prosa. Por qué

Todo se compara contra los segmentos seguros de `segment()` (`scripts/docs/lib.mjs`), nunca
contra el fichero crudo. **Frontmatter YAML, bloques de código delimitados y código inline
quedan fuera**, y no por comodidad: `docs/` contiene legítimamente claves de API de ejemplo,
tokens de relleno y fixtures de test dentro de bloques de código. Un escáner que lee bloques
de código reporta los 1648 ficheros el primer día, y un gate que nace rojo se apaga.

El escáner reutiliza `segment()` en vez de reimplementar el seguimiento de vallas porque esa
lógica ya la comparten los gates de enlaces. Usa los segmentos directamente en lugar de
`proseOf()` porque `proseOf()` descarta los números de línea, y un hallazgo al que no se
puede navegar es un hallazgo que nadie corrige.

## Reglas

| Regla | Severidad | Detecta |
|---|---|---|
| `aws-access-key-id` | error | `AKIA` / `ASIA` + 16 alfanuméricos en mayúsculas |
| `aws-secret-access-key` | error | 40 chars base64-ish, sólo en prosa, con mezcla de casos y dígito |
| `google-api-key` | error | `AIza` + 35 |
| `github-token` | error | `ghp_` `gho_` `ghu_` `ghs_` `ghr_` `github_pat_` |
| `slack-token` | error | `xox[baprs]-` |
| `stripe-live-key` | error | `sk_live_` `rk_live_` |
| `llm-provider-key` | error | `sk-` + 20+ , `sk-ant-` + 20+ |
| `credential-assignment` | error | `password` / `secret` / `token` / `api_key` = literal no-relleno |
| `private-key-pem` | **aviso** | Cabecera `-----BEGIN ... PRIVATE KEY-----` |
| `internal-hostname` | **aviso** | `*.internal` `*.corp` `*.local` `*.lan`, IP RFC1918 como host de URL |

**Severidad** — `error` es una credencial que autenticaría; presupuesto 0, y debe fallar CI
en cuanto aparezca. `aviso` es una forma que a veces es legítima: un bloque PEM es material
real en un documento sobre TLS, y un host interno es material real en una nota de diseño.

**`127.0.0.0/8` está excluido a propósito.** El loopback no es privado, es "tu propia
máquina", y `docs/api/HTTP_API.md` lo usa legítimamente en prosa.

## Qué NO detecta

- **Nada dentro de bloques de código o frontmatter** (arriba, deliberadamente).
- **Cadenas genéricas de alta entropía.** Medido el 2026-09-29: un barrido de entropía
  (24+ chars, clases mixtas) sobre la prosa devuelve **241 cadenas distintas, y todas son
  rutas de fichero, URLs, ids de sesión o un PNG base64 embebido**. Una regla de entropía
  aquí sería ~100% falso positivo y enterraría la señal real.
- **Si una credencial está VIVA.** Nada llama a un proveedor para comprobarlo.
- **Secretos en código, configuración, historial de git o el entorno.** Esta puerta es
  sobre documentación; las demás superficies necesitan otra herramienta.

## Ejecutar en local

Sin dependencias, sin red, Node >= 18. Corre en ~1s sobre 1651 ficheros:

```bash
node scripts/docs/check-secrets.mjs                    # gate, exit 1 sobre presupuesto
node scripts/docs/check-secrets.mjs --self-test        # 29 aserciones, no escanea
node scripts/docs/check-secrets.mjs --json             # legible por máquina
node scripts/docs/check-secrets.mjs --max-errors=0     # ajusta sin tocar código
node scripts/docs/check-secrets.mjs --max-warnings=28
```

Códigos de salida: `0` dentro de presupuesto · `1` presupuesto excedido o self-test fallido ·
`2` uso incorrecto o allowlist malformado.

## Qué significa el presupuesto

Dos presupuestos, **separados por severidad a propósito**, para que la tolerancia de avisos
nunca pueda ablandar el de errores. `--max-warnings=28` no hace pasar una clave `AKIA`.

| Presupuesto | Valor | Por qué no es cero | Se drena en |
|---|---|---|---|
| Errores | **0** | No hay deuda: las 8 reglas de credencial están en 0 sobre 1651 ficheros | — mantener en 0 |
| Avisos | 28 | Los 28 son seudónimos de anonimización en notas de investigación, no credenciales | al redactar esas notas |

Bájalo con `--max-warnings=N` al corregir notas; el error se baja sólo a 0, que ya está.

## Allowlist: declarar en vez de silenciar

Un documento de seguridad **debe** poder enseñar una clave de ejemplo. Eso se declara en
`scripts/docs/secrets-allowlist.json`, no se silencia:

```json
{
  "entries": [
    {
      "file": "docs/dev/workflow/rotacion-claves.md",
      "rule": "llm-provider-key",
      "pattern": "^sk-ant-api03-EXAMPLE",
      "reason": "Clave de ejemplo de la documentación de rotación, no una credencial real."
    }
  ]
}
```

| Campo | Obligatorio | Significado |
|---|---|---|
| `file` | sí | Ruta relativa al repo, o `*` para todo el repo |
| `pattern` | sí | fuente RegExp, comparada contra el texto **crudo** del match |
| `rule` | no | Acota la entrada a una regla; si se omite, cubre todas |
| `reason` | **sí, no vacío** | Por qué es seguro |

Un `reason` ausente o vacío es **error duro (exit 2)**, no aviso: una puerta de seguridad que
acepta una exención inexplicada es una puerta con botón de mute. `pattern` se compara contra
el texto crudo porque el extracto viene redactado y sería imposible de casar.

## Cuándo se ejecuta

- **Push** a `main` / `develop` con cambios en `docs/**`, `scripts/docs/**` o `README.md`
- **Pull request** a `main` / `develop` con los mismos paths
- **Workflow dispatch** manual

Sin `schedule`: a diferencia del rot de enlaces externos, un secreto no se pudre con el
tiempo. Si no está en el PR que lo introduce, tampoco aparece por sorpresa después — y
`push: [main]` ya cubre la ruta post-merge.

## Invariantes

- **Ningún secreto se imprime entero.** Todo extracto va redactado (se enmascara el medio, se
  conserva lo justo para localizarlo), porque esta salida aterriza en logs de CI que se
  pegan en issues. Única excepción: `internal-hostname`, donde el valor *es* el hallazgo y
  enmascararlo haría el informe inaccionable — y no es una credencial.
- **`--self-test` es lo que demuestra que las reglas funcionan.** El corpus está en 0 para
  las 8 reglas de credencial, así que sin el self-test la puerta sería indistinguible de un
  escáner que no casa nada. Una ejecución limpia no prueba nada cuando el conjunto de
  patrones nunca se ha mostrado casando una clave real.
- Ningún script adivina si una credencial está viva. Esto es forma, no validez.
