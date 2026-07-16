# `release-npm-61.yml` — RELEASE: NPM — Publish

## ¿Qué hace?

Publica los paquetes WASM y TypeScript de VantaDB en npm.

## ¿Cómo lo hace?

2 jobs con dependencia (`publish-ts` necesita el artifact WASM):

1. **`publish-wasm`**: (tag `wasm-v*` o `ts-v*`, o dispatch `wasm/both`):
   - Build WASM con `wasm-pack build --release`
   - Sube artifact `wasm-pkg`
   - Publica `vantadb-wasm` a npm (skip si el tag es solo `ts-v*` y no es dispatch)
   - Autenticación vía `NODE_AUTH_TOKEN` (Granular Access Token, bypass-2fa habilitado)
2. **`publish-ts`**: (tag `ts-v*` o dispatch `ts/both`):
   - Descarga el artifact `wasm-pkg` generado por `publish-wasm`
   - `npm install` y `npm run build` en `vantadb-ts`
   - Reescribe la dependencia `vantadb-wasm` en package.json con la versión exacta del build
   - Publica `vantadb` (TypeScript SDK) a npm

Soporta `--dry-run` para verificación sin publicar.

## ¿Qué tests usa?

No ejecuta tests. Solo build y publish.

## ¿Qué verifica?

- El build WASM es exitoso
- El build TypeScript es exitoso
- La versión de dependencia `vantadb-wasm` se sincroniza correctamente
- La publicación a npm es exitosa

## Funcionalidad final

Publicar el binding WASM y el SDK TypeScript de VantaDB en npm para consumo desde aplicaciones web/Node.js.

## ¿Cuándo se ejecuta?

- **Push** de tag `wasm-vX.Y.Z` (publica `vantadb-wasm` únicamente)
- **Push** de tag `ts-vX.Y.Z` (publica ambos — `vantadb-wasm` + `vantadb`)
- **Workflow dispatch** manual: elige paquete (`wasm`, `ts`, `both`) y opción dry-run
