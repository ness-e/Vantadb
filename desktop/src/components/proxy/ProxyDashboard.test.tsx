// DESKTOP-38: helpers puros del ProxyDashboard — TTL legible + persistencia
// de URL del proxy (localStorage inyectado por jsdom).
import { describe, expect, it, beforeEach } from "vitest";
import { proxyUrl, ttlLabel, PROXY_URL_EVENT } from "./ProxyDashboard";

// FIND-63: stub localStorage en memoria POR ARCHIVO (Node ≥26 sin
// --localstorage-file → undefined; no config global, no toca worker node).
if (
  typeof (globalThis as { localStorage?: unknown }).localStorage === "undefined"
) {
  const __find63map = new Map<string, string>();
  (globalThis as Record<string, unknown>).localStorage = {
    getItem: (k: string) => __find63map.get(String(k)) ?? null,
    setItem: (k: string, v: string) => void __find63map.set(String(k), String(v)),
    removeItem: (k: string) => void __find63map.delete(String(k)),
    clear: () => __find63map.clear(),
    key: () => null,
    length: 0,
  } satisfies Storage;
}

describe("ttlLabel", () => {
  it("reports no TTL for terminal sessions (undefined expires_at_ms)", () => {
    expect(ttlLabel(undefined)).toBe("sin TTL");
  });

  it("formats minutes above 60s", () => {
    expect(ttlLabel(Date.now() + 23 * 60 * 1000)).toBe("23m");
  });

  it("formats seconds under 60s and expired as expired", () => {
    expect(ttlLabel(Date.now() + 45_000)).toBe("45s");
    expect(ttlLabel(Date.now() - 1000)).toBe("expirado");
  });
});

describe("proxyUrl", () => {
  beforeEach(() => {
    localStorage.removeItem("vanta.proxy.url");
  });

  it("defaults to empty and roundtrips through storage + event", () => {
    expect(proxyUrl()).toBe("");
    let fired = 0;
    window.addEventListener(PROXY_URL_EVENT, () => fired++, { once: true });
    localStorage.setItem("vanta.proxy.url", "http://127.0.0.1:8096");
    expect(proxyUrl()).toBe("http://127.0.0.1:8096");
    window.dispatchEvent(new Event(PROXY_URL_EVENT));
    expect(fired).toBe(1);
  });
});
