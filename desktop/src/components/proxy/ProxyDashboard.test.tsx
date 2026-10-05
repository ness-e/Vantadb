// DESKTOP-38: helpers puros del ProxyDashboard — TTL legible + persistencia
// de URL del proxy (localStorage inyectado por jsdom).
// DESKTOP-44 (FIND-155): auth del dashboard — storage de la user key,
// header `x-vanta-user-key` en GET /snapshot y hint accionable de 401.
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import ProxyDashboard, {
  PROXY_URL_EVENT,
  proxyUrl,
  proxyUserKey,
  ttlLabel,
} from "./ProxyDashboard";

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

// ── FIND-155: auth del dashboard ───────────────────────────────────────────

const emptySnapshot = {
  turns: [],
  sessions: [],
  writeback: { pending_labels: [], pending_count: 0 },
  rate_limit: { limit_per_minute: 60, hits_total: 0, degraded: false },
};

function okResponse(body: unknown = emptySnapshot) {
  return { ok: true, status: 200, json: async () => body };
}

describe("proxyUserKey (FIND-155)", () => {
  beforeEach(() => {
    localStorage.removeItem("vanta.proxy.url");
    localStorage.removeItem("vanta.proxy.userKey");
  });

  it("defaults to empty and reads the stored key", () => {
    expect(proxyUserKey()).toBe("");
    localStorage.setItem("vanta.proxy.userKey", "sk-test");
    expect(proxyUserKey()).toBe("sk-test");
  });

  it("setup form stores the URL and the trimmed key", async () => {
    vi.stubGlobal("fetch", vi.fn().mockResolvedValue(okResponse()));
    render(<ProxyDashboard />);
    fireEvent.change(screen.getByLabelText("URL base del proxy"), {
      target: { value: "http://127.0.0.1:8096/" },
    });
    fireEvent.change(screen.getByLabelText("User key (header x-vanta-user-key)"), {
      target: { value: "  sk-test  " },
    });
    fireEvent.submit(screen.getByText("CONECTAR").closest("form")!);
    await waitFor(() => expect(proxyUrl()).toBe("http://127.0.0.1:8096"));
    expect(proxyUserKey()).toBe("sk-test");
  });
});

describe("ProxyDashboard snapshot auth (FIND-155)", () => {
  beforeEach(() => {
    localStorage.setItem("vanta.proxy.url", "http://127.0.0.1:8096");
    localStorage.setItem("vanta.proxy.userKey", "sk-test");
  });

  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
    localStorage.removeItem("vanta.proxy.url");
    localStorage.removeItem("vanta.proxy.userKey");
  });

  it("sends x-vanta-user-key from storage on the snapshot poll", async () => {
    const fetchMock = vi.fn().mockResolvedValue(okResponse());
    vi.stubGlobal("fetch", fetchMock);
    render(<ProxyDashboard />);
    await waitFor(() => expect(fetchMock).toHaveBeenCalled());
    expect(fetchMock).toHaveBeenCalledWith(
      "http://127.0.0.1:8096/snapshot",
      expect.objectContaining({
        headers: { "x-vanta-user-key": "sk-test" },
      }),
    );
  });

  it("shows an actionable hint when the snapshot returns 401", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({ ok: false, status: 401, json: async () => ({}) }),
    );
    render(<ProxyDashboard />);
    await waitFor(() => expect(screen.getByText(/HTTP 401/)).toBeTruthy());
    expect(screen.getByText("401 — la user key falta o no es válida")).toBeTruthy();
  });
});
