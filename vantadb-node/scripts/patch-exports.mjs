// FIND-546: `VantaDB` is a runtime alias of `VantaDb` (same class object).
// `napi build` regenerates index.cjs/index.js on every run and drops the
// `??` fallback, exporting `undefined` for VantaDB. This script reapplies
// the alias idempotently — run it after every napi build (wired into the
// `build` script and .github/workflows/release-npm-node.yml).
import { readFileSync, writeFileSync } from 'node:fs'

const cjsFile = 'index.cjs'
const esmFile = 'index.js'

let cjs = readFileSync(cjsFile, 'utf8')
if (!cjs.includes('nativeBinding.VantaDB ??')) {
  const before = cjs
  cjs = cjs.replace(
    /^module\.exports\.VantaDB = nativeBinding\.VantaDB$/m,
    'module.exports.VantaDB = nativeBinding.VantaDB ?? nativeBinding.VantaDb;',
  )
  if (cjs === before) throw new Error('patch-exports: CJS VantaDB export line not found')
}
writeFileSync(cjsFile, cjs)

let esm = readFileSync(esmFile, 'utf8')
if (!esm.includes('const VantaDB = nativeBinding.VantaDB ??')) {
  const before = esm
  esm = esm.replace(
    /const \{ AbortHandle, VantaDb, VantaDB \} = nativeBinding/,
    'const { AbortHandle, VantaDb } = nativeBinding\nconst VantaDB = nativeBinding.VantaDB ?? VantaDb',
  )
  if (esm === before) throw new Error('patch-exports: ESM VantaDB destructure not found')
}
writeFileSync(esmFile, esm)
console.log('patch-exports: VantaDB alias restored in index.cjs / index.js')
