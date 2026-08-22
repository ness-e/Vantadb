#!/usr/bin/env node
// GOV-B4 — OpenAPI ↔ router parity check.
//
// Extracts every `.route("...", get(..).post(..))` registration from
// src/cli_server.rs (static regex + paren scanner, multi-line safe) and
// compares against the paths/methods declared in docs/api/openapi.yaml
// (parsed with a minimal indentation-aware reader — node stdlib only).
//
// Exit 0: full parity. Exit 1: lists missing/extra paths and method diffs.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const RS_FILE = path.join(ROOT, "src", "cli_server.rs");
const YAML_FILE = path.join(ROOT, "docs", "api", "openapi.yaml");

const METHODS = new Set(["get", "post", "put", "delete", "patch", "head", "options"]);

/** Normalize axum path syntax to OpenAPI templates: `{*name}` -> `{name}`. */
function normalizePath(p) {
  return p.replace(/\{\*\w+\}/g, (m) => `{${m.slice(2, -1)}}`);
}

/**
 * Extract {path, methods[]} from every `.route(` call in the Rust source.
 * Handles single- and multi-line chains like:
 *   .route("/x", get(h).post(h2))
 *   .route(
 *       "/y",
 *       get(h),
 *   )
 */
function extractRsRoutes(src) {
  const routes = [];
  let idx = 0;
  while ((idx = src.indexOf(".route(", idx)) !== -1) {
    let i = idx + ".route(".length;
    // Parse the quoted path literal.
    const ws = /\s*/y;
    ws.lastIndex = i;
    ws.exec(src);
    i = ws.lastIndex;
    if (src[i] !== '"') {
      throw new Error(`.route( at offset ${idx}: expected quoted path`);
    }
    const closeQuote = src.indexOf('"', i + 1);
    if (closeQuote === -1) throw new Error(`.route( at offset ${idx}: unterminated string`);
    const rawPath = src.slice(i + 1, closeQuote);
    i = closeQuote + 1;

    // Scan until the route call's matching close paren; collect method idents.
    let depth = 1;
    const methods = [];
    const ident = /[A-Za-z_][A-Za-z0-9_]*/y;
    while (i < src.length && depth > 0) {
      const c = src[i];
      if (c === "(") depth++;
      else if (c === ")") {
        depth--;
        i++;
        continue;
      } else if (/[A-Za-z_]/.test(c)) {
        ident.lastIndex = i;
        const m = ident.exec(src);
        const name = m[0];
        if (METHODS.has(name) && src[m.index + name.length] === "(") {
          methods.push(name);
        }
        i = m.index + name.length;
        continue;
      }
      i++;
    }
    if (depth !== 0) throw new Error(`.route("${rawPath}": unbalanced parens`);
    if (methods.length === 0) throw new Error(`.route("${rawPath}": no HTTP method found`);
    routes.push({ path: normalizePath(rawPath), methods: methods.sort() });
    idx += 7;
  }
  return routes;
}

/**
 * Minimal reader for the `paths:` section of openapi.yaml.
 * Relies on the file's stable formatting: 2-space path keys, 4-space method
 * keys (`get:`/`post:`/...) directly under each path. Not a general YAML
 * parser — enough for this contract-first file, which we own.
 */
function extractYamlPaths(text) {
  const lines = text.split(/\r?\n/);
  const start = lines.findIndex((l) => l === "paths:");
  if (start === -1) throw new Error("openapi.yaml: no top-level 'paths:' section");
  const result = new Map(); // path -> Set(methods)
  let current = null;
  for (let n = start + 1; n < lines.length; n++) {
    const line = lines[n];
    if (/^\S/.test(line)) break; // next top-level key: end of paths section
    const pathMatch = /^ {2}(\/\S*):$/.exec(line);
    if (pathMatch) {
      current = pathMatch[1];
      if (!result.has(current)) result.set(current, new Set());
      continue;
    }
    const methodMatch = /^ {4}([a-z]+):(?:\s|$)/.exec(line);
    if (methodMatch && current && METHODS.has(methodMatch[1])) {
      result.get(current).add(methodMatch[1]);
    }
  }
  return result;
}

function main() {
  const rsRoutes = extractRsRoutes(fs.readFileSync(RS_FILE, "utf8"));
  const yamlPaths = extractYamlPaths(fs.readFileSync(YAML_FILE, "utf8"));

  const rsMap = new Map();
  for (const r of rsRoutes) {
    if (!rsMap.has(r.path)) rsMap.set(r.path, new Set());
    for (const m of r.methods) rsMap.get(r.path).add(m);
  }

  const missing = []; // registered in Rust but absent from yaml
  const extra = []; // documented in yaml but not registered in Rust
  const methodDiffs = [];

  for (const [p, methods] of rsMap) {
    if (!yamlPaths.has(p)) {
      missing.push(`${p} (${methods.size} op${methods.size > 1 ? "s" : ""})`);
      continue;
    }
    const yMethods = yamlPaths.get(p);
    const onlyRs = [...methods].filter((m) => !yMethods.has(m));
    const onlyYaml = [...yMethods].filter((m) => !methods.has(m));
    if (onlyRs.length || onlyYaml.length) {
      methodDiffs.push(`${p}: rs=[${[...methods].sort().join(",")}] yaml=[${[...yMethods].sort().join(",")}]`);
    }
  }
  for (const p of yamlPaths.keys()) {
    if (!rsMap.has(p)) extra.push(p);
  }

  const v2Count = [...rsMap.keys()].filter((p) => p.startsWith("/api/v2/")).length;
  const opCount = rsRoutes.reduce((n, r) => n + r.methods.length, 0);
  console.log(`Router (src/cli_server.rs): ${rsMap.size} paths, ${opCount} operations (${v2Count} paths under /api/v2/*)`);
  console.log(`OpenAPI (docs/api/openapi.yaml): ${yamlPaths.size} paths`);

  let ok = true;
  if (missing.length) {
    ok = false;
    console.error("\nFAIL: routes in cli_server.rs missing from openapi.yaml:");
    for (const m of missing.sort()) console.error(`  - ${m}`);
  }
  if (extra.length) {
    ok = false;
    console.error("\nFAIL: paths in openapi.yaml not registered in cli_server.rs:");
    for (const e of extra.sort()) console.error(`  - ${e}`);
  }
  if (methodDiffs.length) {
    ok = false;
    console.error("\nFAIL: method mismatches:");
    for (const d of methodDiffs.sort()) console.error(`  - ${d}`);
  }
  if (ok) {
    console.log("\nParity OK: openapi.yaml matches the registered router exactly.");
    process.exit(0);
  }
  process.exit(1);
}

main();
