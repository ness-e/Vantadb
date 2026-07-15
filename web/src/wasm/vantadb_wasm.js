/* @ts-self-types="./vantadb_wasm.d.ts" */
import * as wasm from "./vantadb_wasm_bg.wasm";
import { __wbg_set_wasm } from "./vantadb_wasm_bg.js";

__wbg_set_wasm(wasm);

export { VantaDB } from "./vantadb_wasm_bg.js";
