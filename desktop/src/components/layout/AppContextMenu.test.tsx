// FIND-21 Step 1 (RED): menú contextual propio in-app — items con atajo
// visible, click dispara la acción, Esc lo cierra.
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppContextMenu, type ContextMenuItem } from "./AppContextMenu";

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

const items = (): ContextMenuItem[] => [
  { id: "palette", label: "Paleta de comandos", hint: "Ctrl+K", onSelect: vi.fn() },
  { id: "help", label: "Guía rápida", hint: "?", onSelect: vi.fn() },
  { id: "theme", label: "Cambiar tema", hint: "Alt+T", onSelect: vi.fn() },
  { id: "settings", label: "Ir a Ajustes", hint: "Ctrl+,", onSelect: vi.fn() },
];

describe("AppContextMenu", () => {
  it("renderiza los items con su atajo visible", () => {
    render(<AppContextMenu x={100} y={80} items={items()} onClose={() => {}} />);
    expect(screen.getByRole("menu")).toBeTruthy();
    expect(screen.getByText("Paleta de comandos")).toBeTruthy();
    expect(screen.getByText("Ctrl+K")).toBeTruthy();
    expect(screen.getByText("Alt+T")).toBeTruthy();
  });

  it("click en un item dispara su acción y cierra", () => {
    const list = items();
    const onClose = vi.fn();
    render(<AppContextMenu x={10} y={10} items={list} onClose={onClose} />);
    fireEvent.click(screen.getByText("Guía rápida"));
    expect(list[1].onSelect).toHaveBeenCalledTimes(1);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("Esc cierra el menú", () => {
    const onClose = vi.fn();
    render(<AppContextMenu x={10} y={10} items={items()} onClose={onClose} />);
    fireEvent.keyDown(screen.getByRole("menu"), { key: "Escape" });
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
