// UX-A11Y-01 Step 1 (RED→GREEN): ImportPaste sin window.confirm nativo —
// confirmación inline (patrón IngestForm confirming). El test espía
// window.confirm: si el modal lo llama, el test falla.
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import ImportPaste from "./ImportPaste";
import ImportDrop from "./ImportDrop";
import { EXAMPLE_CSV } from "./parseImport";

const mocks = vi.hoisted(() => ({
  ingestBatch: vi.fn(),
}));

vi.mock("../../vanta", () => ({
  ingestBatch: mocks.ingestBatch,
  vantaErrorMessage: (e: unknown) => String(e),
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
  vi.restoreAllMocks();
});

const noop = () => {};

describe("ImportPaste confirm inline (UX-A11Y-01)", () => {
  it("IMPORTAR pide confirmación inline sin window.confirm", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
    mocks.ingestBatch.mockResolvedValue(["nota-1", "nota-2", "nota-3"]);
    const onImported = vi.fn();
    render(
      <ImportPaste open onClose={noop} defaultNamespace="default" onImported={onImported} onError={noop} />,
    );

    fireEvent.click(screen.getByText(/EJEMPLO/));
    const importBtn = await screen.findByText(/IMPORTAR 3/);
    fireEvent.click(importBtn);

    // Confirmación inline visible (role=alert), sin dialog nativo.
    expect(confirmSpy).not.toHaveBeenCalled();
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toMatch(/importar 3 registros/i);

    // Confirmar ejecuta el import y reporta.
    fireEvent.click(screen.getByText(/CONFIRMAR IMPORT/i));
    await waitFor(() => expect(onImported).toHaveBeenCalledWith(3));
    expect(mocks.ingestBatch).toHaveBeenCalled();
  });

  it("cancelar la confirmación no importa nada", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(true);
    const onImported = vi.fn();
    render(
      <ImportPaste open onClose={noop} defaultNamespace="default" onImported={onImported} onError={noop} />,
    );

    fireEvent.click(screen.getByText(/EJEMPLO/));
    fireEvent.click(await screen.findByText(/IMPORTAR 3/));
    await screen.findByRole("alert");
    fireEvent.click(screen.getByText(/CANCELAR/));

    expect(onImported).not.toHaveBeenCalled();
    expect(mocks.ingestBatch).not.toHaveBeenCalled();
  });
});

describe("ImportDrop confirm inline (UX-A11Y-01)", () => {
  it("IMPORTAR pide confirmación inline sin window.confirm", async () => {
    const confirmSpy = vi.spyOn(window, "confirm").mockReturnValue(true);
    mocks.ingestBatch.mockResolvedValue(["nota-1", "nota-2", "nota-3"]);
    const onImported = vi.fn();
    const { container } = render(
      <ImportDrop open onClose={noop} defaultNamespace="default" onImported={onImported} onError={noop} />,
    );

    const input = container.querySelector<HTMLInputElement>("#vdb-file-input");
    expect(input).toBeTruthy();
    fireEvent.change(input!, {
      target: { files: [new File([EXAMPLE_CSV], "notas.csv", { type: "text/csv" })] },
    });
    fireEvent.click(await screen.findByText(/IMPORTAR 3/));

    expect(confirmSpy).not.toHaveBeenCalled();
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toMatch(/importar 3 registros/i);

    fireEvent.click(screen.getByText(/CONFIRMAR IMPORT/i));
    await waitFor(() => expect(onImported).toHaveBeenCalledWith(3));
    expect(mocks.ingestBatch).toHaveBeenCalled();
  });
});
