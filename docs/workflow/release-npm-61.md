# `release-npm-61.yml` — RELEASE: NPM — Publish

## ¿Qué hace?

Publica los paquetes WASM y TypeScript de VantaDB en npm.

## ¿Cómo lo hace?

2 jobs paralelos (con dependencia interna):

1. **`publish-wasm`**: (si tag `wasm-v*` o dispatch con `package: wasm/both`):
   - Build WASM con `wasm-pack build --release`
   - Sube artifact `wasm-pkg`
   - Publica `vantadb-wasm` a npm
2. **`publish-ts`**: (si tag `ts-v*` o dispatch con `package: ts/both`):
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

- **Push** de tag `wasm-v*.*.*` (publica WASM)
- **Push** de tag `ts-v*.*.*` (publica TS)
- **Workflow dispatch** manual con opción de elegir paquete (wasm, ts, both) y dry-run
