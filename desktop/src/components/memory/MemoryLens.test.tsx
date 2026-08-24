// DESKTOP-37 Step 4: lente MEMORIA — mockea el bridge `../../vanta` y valida
// escenas con heat, persona diff, skills timeline por content-hash y genlog
// con filtro de capa + click→Inspector (onOpenRecord) para anchors.
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import MemoryLens, { lineDiff } from "./MemoryLens";

const mocks = vi.hoisted(() => ({
  get: vi.fn(),
  memorySceneList: vi.fn(),
  memorySceneRead: vi.fn(),
  memoryPersonaGet: vi.fn(),
  memorySkillList: vi.fn(),
  memoryGenlogQuery: vi.fn(),
}));

vi.mock("../../vanta", () => ({
  ...mocks,
  vantaErrorMessage: (e: unknown) => String(e),
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const noop = () => {};

describe("lineDiff", () => {
  it("marca líneas agregadas y removidas", () => {
    const d = lineDiff("a\nb\nc", "a\nB\nc\nd");
    expect(d.added).toEqual(["B", "d"]);
    expect(d.removed).toEqual(["b"]);
  });
});

describe("MemoryLens", () => {
  it("sin backend activo no consulta el bridge", () => {
    render(<MemoryLens active={false} onNotice={noop} onError={noop} onOpenRecord={noop} />);
    expect(screen.getByText(/sin backend activo/)).toBeTruthy();
    expect(mocks.memorySceneList).not.toHaveBeenCalled();
  });

  it("escenas listadas con heat y detalle inline al click", async () => {
    mocks.memorySceneList.mockResolvedValue([
      { filename: "s-low.md", summary: "fría", heat: 1.5, created: "c1", updated: "u1" },
      { filename: "s-hot.md", summary: "caliente", heat: 9.0, created: "c2", updated: "u2" },
    ]);
    mocks.memorySceneRead.mockImplementation((_s: string, name: string) =>
      name === "s-low.md"
        ? Promise.reject(new Error("404"))
        : Promise.resolve({
            scene_name: "s-hot.md",
            meta: { created: "c2", updated: "u2", summary: "caliente", heat: 9.0 },
            content: "contenido vivo",
            deleted: false,
          }),
    );
    render(<MemoryLens active sessionKey="user-1" onNotice={noop} onError={noop} onOpenRecord={noop} />);

    await waitFor(() => expect(screen.getByText("s-hot.md")).toBeTruthy());
    expect(screen.getByText("s-low.md")).toBeTruthy();
    expect(screen.getByText(/heat 9\.0/)).toBeTruthy();
    // Barra de la más caliente a ancho completo (100%).
    expect(document.querySelector<HTMLDivElement>('[style*="width: 100%"]')).toBeTruthy();

    fireEvent.click(screen.getByText("s-hot.md"));
    await waitFor(() => expect(screen.getByText("contenido vivo")).toBeTruthy());

    // Soft-deleted visible: read 404 → badge en vez de contenido.
    fireEvent.click(screen.getByText("s-low.md"));
    await waitFor(() =>
      expect(screen.getByText(/soft-deleted — bloque no accesible/)).toBeTruthy(),
    );
    expect(mocks.memorySceneRead).toHaveBeenCalledWith("user-1", "s-low.md");
  });

  it("persona: snapshot + diff contra la última vista (localStorage)", async () => {
    localStorage.setItem("vanta-persona-last:user-1", "gusta el jazz\nvive en CABA");
    mocks.memoryPersonaGet.mockResolvedValue({
      content: "gusta el jazz\nvive en Palermo\nusa Linux",
      mode: "reflect",
      generated_at_ms: 1,
      generated_at: "ayer",
    });
    render(
      <MemoryLens active sessionKey="user-1" onNotice={noop} onError={noop} onOpenRecord={noop} />,
    );
    fireEvent.click(screen.getByRole("tab", { name: /PERSONA/ }));

    await waitFor(() => expect(screen.getByText(/diff vs última snapshot vista/)).toBeTruthy());
    expect(screen.getByText("+ usa Linux")).toBeTruthy();
    expect(screen.getByText("+ vive en Palermo")).toBeTruthy();
    expect(screen.getByText("− vive en CABA")).toBeTruthy();
  });

  it("skills agrupadas con timeline por content-hash y visor de versión", async () => {
    mocks.memorySkillList.mockResolvedValue([
      { name: "sql-tuning", description: "d2", content: "SELECT hints v2", content_hash: 0xdeadbeef, updated_at_ms: 2000 },
      { name: "sql-tuning", description: "d1", content: "SELECT hints v1", content_hash: 0x00c0ffee, updated_at_ms: 1000 },
    ]);
    render(
      <MemoryLens active sessionKey="user-1" onNotice={noop} onError={noop} onOpenRecord={noop} />,
    );
    fireEvent.click(screen.getByRole("tab", { name: /SKILLS/ }));

    await waitFor(() => expect(screen.getByText("sql-tuning")).toBeTruthy());
    expect(screen.getByText("2 versiones")).toBeTruthy();
    // Hashes cortos hex visibles (timeline), una fila por versión.
    expect(screen.getByText("00c0ffee")).toBeTruthy(); // 0x00c0ffee padStart(8)
    expect(screen.getByText("deadbeef")).toBeTruthy();

    // Click en la versión vieja → muestra su contenido (no el último).
    fireEvent.click(screen.getByText("00c0ffee"));
    await waitFor(() => expect(screen.getByText("SELECT hints v1")).toBeTruthy());
  });

  it("genlog: filtro por capa re-consulta; anchor abre record en Inspector", async () => {
    mocks.memoryGenlogQuery.mockImplementation((_s, layer) =>
      Promise.resolve(
        layer === undefined
          ? [
              { layer: "l1", status: "succeeded", anchor_id: "rec-1", session_key: "user-1", ts_ms: 10 },
              { layer: "l3", status: "failed", error: "boom", session_key: "user-1", ts_ms: 20 },
            ]
          : [{ layer: "l1", status: "succeeded", anchor_id: "rec-1", session_key: "user-1", ts_ms: 10 }],
      ),
    );
    const onOpen = vi.fn();
    mocks.get.mockResolvedValue({ id: "rec-1", namespace: "ns", text: "hola", metadata: {} });
    render(
      <MemoryLens active sessionKey="user-1" onNotice={noop} onError={noop} onOpenRecord={onOpen} />,
    );
    fireEvent.click(screen.getByRole("tab", { name: /GENLOG/ }));
    await waitFor(() => expect(screen.getByText("rec-1")).toBeTruthy());

    fireEvent.click(screen.getByRole("button", { name: "L1" }));
    await waitFor(() =>
      expect(mocks.memoryGenlogQuery).toHaveBeenLastCalledWith("user-1", "l1", 200),
    );

    // Entry con anchor → Inspector con el record real.
    fireEvent.click(screen.getByText("rec-1"));
    await waitFor(() => expect(onOpen).toHaveBeenCalledWith(expect.objectContaining({ id: "rec-1" }), null));
  });

  it("genlog: entry sin anchor no navega al Inspector", async () => {
    mocks.memoryGenlogQuery.mockResolvedValue([
      { layer: "l2", status: "failed", error: "sin record", session_key: "user-1", ts_ms: 5 },
    ]);
    const onOpen = vi.fn();
    render(
      <MemoryLens active sessionKey="user-1" onNotice={noop} onError={noop} onOpenRecord={onOpen} />,
    );
    fireEvent.click(screen.getByRole("tab", { name: /GENLOG/ }));
    await waitFor(() => expect(screen.getByText(/sin record/)).toBeTruthy());
    fireEvent.click(screen.getByText(/sin record/));
    expect(onOpen).not.toHaveBeenCalled();
  });
});
