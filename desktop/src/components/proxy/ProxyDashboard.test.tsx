// DESKTOP-38: helpers puros del ProxyDashboard — TTL legible + persistencia
// de URL del proxy (localStorage inyectado por jsdom).
import { describe, expect, it, beforeEach } from "vitest";
import { proxyUrl, ttlLabel, PROXY_URL_EVENT } from "./ProxyDashboard";

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
