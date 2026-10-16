// VS-16 deep-link parsing tests — pure logic, no DOM/IPC (node --test).
import { test } from "node:test";
import assert from "node:assert/strict";
import { parseVantaUrl } from "./vanta.ts";

test("parseVantaUrl: full namespace + key", () => {
  assert.deepEqual(parseVantaUrl("vanta://memorias/abc123"), {
    namespace: "memorias",
    key: "abc123",
    query: null,
  });
});

test("parseVantaUrl: query only", () => {
  assert.deepEqual(parseVantaUrl("vanta://?query=cats"), {
    namespace: null,
    key: null,
    query: "cats",
  });
});

test("parseVantaUrl: namespace + query", () => {
  assert.deepEqual(parseVantaUrl("vanta://memorias?query=cats%20dogs"), {
    namespace: "memorias",
    key: null,
    query: "cats dogs",
  });
});

test("parseVantaUrl: trailing slashes collapse", () => {
  assert.deepEqual(parseVantaUrl("vanta://memorias//"), {
    namespace: "memorias",
    key: null,
    query: null,
  });
});

test("parseVantaUrl: namespace only", () => {
  assert.deepEqual(parseVantaUrl("vanta://memorias"), {
    namespace: "memorias",
    key: null,
    query: null,
  });
});

test("parseVantaUrl: percent-encoded path segments decode", () => {
  assert.deepEqual(parseVantaUrl("vanta://my%20ns/my%2Fkey"), {
    namespace: "my ns",
    key: "my/key",
    query: null,
  });
});

test("parseVantaUrl: malformed percent-encoding rejected", () => {
  assert.equal(parseVantaUrl("vanta://%zz/key"), null);
});

test("parseVantaUrl: rejects non-vanta schemes (security: fake CLI args)", () => {
  assert.equal(parseVantaUrl("https://evil.example/vanta://x"), null);
  assert.equal(parseVantaUrl("vanta:memorias"), null);
  assert.equal(parseVantaUrl(""), null);
  assert.equal(parseVantaUrl("  "), null);
});