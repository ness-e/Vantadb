// DESKTOP-26 Step 3: lente representativo — HomeOverview (VS-04/VS-CORE-02).
// Mockea el bridge `../../vanta` y valida el render de cards + merge de
// namespace_stats + fallback cuando el backend falla.
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import HomeOverview from "./HomeOverview";

const mocks = vi.hoisted(() => ({
  list: vi.fn(),
  namespaceStats: vi.fn(),
}));

vi.mock("../../vanta", () => ({
  list: mocks.list,
  namespaceStats: mocks.namespaceStats,
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

function rec(id: string, over: Partial<Record<string, unknown>> = {}) {
  return {
    id,
    namespace: "ns-a",
    text: `text-${id}`,
    updated_at_ms: Date.now() - 1000,
    ...over,
  };
}

describe("HomeOverview", () => {
  it("inactivo → estado cargando, sin llamar al bridge", () => {
    render(<HomeOverview active={false} />);
    expect(screen.getByText("cargando…")).toBeTruthy();
    expect(mocks.list).not.toHaveBeenCalled();
  });

  it("renderiza cards con merge de namespace_stats sobre list()", async () => {
    mocks.list.mockResolvedValue([
      rec("k1", { metadata: { tag: "x" } }),
      rec("k2", { namespace: "ns-b", vector: [1, 2] }),
    ]);
    // Stats reales: total=5 (incluye expirados que list() oculta).
    mocks.namespaceStats.mockResolvedValue({
      "ns-a": { count: 4, expiring_soon: 0, expired: 1 },
      "ns-b": { count: 1, expiring_soon: 0, expired: 0 },
    });

    render(<HomeOverview active />);
    await waitFor(() => expect(screen.getByText("MEMORIA EN VISTA")).toBeTruthy());

    // Total viene de stats (5), no de list (2).
    expect(screen.getByText("5")).toBeTruthy();
    // Filas por namespace con conteo exacto.
    expect(screen.getByText("ns-a")).toBeTruthy();
    expect(screen.getByText("ns-b")).toBeTruthy();
    // Expirados no purgados visibles vía stats ("1" aparece en ns-b,
    // expirados y con-vector — las 3 cards).
    expect(screen.getAllByText("1")).toHaveLength(3);
    // Con vector: solo k2.
    expect(screen.getByText("Registros con vector")).toBeTruthy();
  });

  it("list() rechaza → mensaje de fallo, sin crashear", async () => {
    mocks.list.mockRejectedValue(new Error("backend down"));
    render(<HomeOverview active />);
    await waitFor(() =>
      expect(screen.getByText("no se pudo leer el backend")).toBeTruthy(),
    );
  });
});
