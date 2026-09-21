/**
 * FND-05 — Prototipo: `await using` / `Symbol.asyncDispose` para el SDK TS.
 *
 * Gap cubierto: TS-2 — `NativeVantaDB` tiene `async close()` pero no implementa
 * `[Symbol.asyncDispose]`, así que no se puede escribir `await using db = ...`
 * (Explicit Resource Management, TypeScript 5.2+, fuente:
 * https://devblogs.microsoft.com/typescript/announcing-typescript-5-2/).
 *
 * Este archivo es un PROTOTIPO/ejemplo — NO modifica el SDK. Muestra el patrón
 * que el SDK debería exponer, implementado como wrapper aquí.
 *
 * Requisito de compilación (solo para este ejemplo): tsconfig del SDK necesita
 * `lib: ["ES2022", "ESNext.Disposable"]` (ver recomendación al final).
 *
 * Este archivo NO se compila con el tsconfig del SDK (`include: src/**`) — es
 * una referencia de cómo se usaría el patrón. Para probarlo, copiarlo a un
 * proyecto con la lib configurada.
 */

import { NativeVantaDB, type NativeConnectOptions } from "../vantadb-ts/src/native.js";

/** Crea un `NativeVantaDB` que implementa `AsyncDisposable`. */
export async function connectDisposable(
  path: string = ":memory:",
  options?: NativeConnectOptions,
): Promise<DisposableNativeVantaDB> {
  const inner = await NativeVantaDB.connect(path, options);
  return new DisposableNativeVantaDB(inner);
}

export class DisposableNativeVantaDB implements AsyncDisposable {
  #inner: NativeVantaDB;

  private constructor(inner: NativeVantaDB) {
    this.#inner = inner;
  }

  /**
   * Método idiomático del prototipo: permite `await using db = ...` y el close
   * corre automáticamente al salir del scope (incluso con early return/throw).
   */
  async [Symbol.asyncDispose](): Promise<void> {
    await this.#inner.close();
  }

  // Delega el resto de la API al backend nativo async.
  put(input: Parameters<NativeVantaDB["put"]>[0]): Promise<ReturnType<NativeVantaDB["put"]>> {
    return this.#inner.put(input);
  }

  search(request: Parameters<NativeVantaDB["search"]>[0]): Promise<ReturnType<NativeVantaDB["search"]>> {
    return this.#inner.search(request);
  }

  get(namespace: string, key: string): Promise<ReturnType<NativeVantaDB["get"]>> {
    return this.#inner.get(namespace, key);
  }
}

/**
 * Uso idiomático objetivo:
 *
 *   await using db = await connectDisposable("./my_brain");
 *   await db.put({ namespace: "docs", key: "a", payload: "first" });
 *   // close() corre solo al salir del scope — no hace falta try/finally.
 */
async function demo(): Promise<void> {
  // Fallback ACTUAL del SDK (try/finally manual):
  const dbOld = await NativeVantaDB.connect(":memory:");
  try {
    await dbOld.put({ namespace: "docs", key: "a", payload: "first" });
  } finally {
    await dbOld.close();
  }

  // Uso idiomático objetivo (con `await using`):
  await using db = await connectDisposable(":memory:");
  await db.put({ namespace: "docs", key: "b", payload: "second" });
  const hits = await db.search({
    namespace: "docs",
    query_vector: [0.1, 0.2, 0.3],
    top_k: 5,
  });
  console.log("hits:", hits.length);
  // close() corre automáticamente aquí.
}

void demo();

// ── Recomendación de implementación (para el SDK real, NO aplicada aquí) ──
//
// 1. En `vantadb-ts/src/native.ts`, en `NativeVantaDB` (ya tiene `async close()`):
//
//      async [Symbol.asyncDispose](): Promise<void> {
//        await this.close();
//      }
//
// 2. En `vantadb-ts/src/vantadb.ts`, en `VantaDB` (WASM sync, close() sync):
//
//      [Symbol.dispose](): void {
//        this.close();
//      }
//
// 3. En `vantadb-ts/tsconfig.json`:
//
//      "target": "ES2022",
//      "lib": ["ES2022", "ESNext.Disposable"]
//
// 4. Actualizar README: `await using db = await NativeVantaDB.connect(...)`
//    (esto además arregla TS-3, el `await` engañoso sobre la API sync).