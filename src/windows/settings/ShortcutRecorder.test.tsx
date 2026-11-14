import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { ShortcutRecorder } from "./ShortcutRecorder";

describe("ShortcutRecorder — Captura de Atalhos", () => {
  it("captura combinação com modificadores e ignora tecla sem modificador", () => {
    const onChange = vi.fn();

    render(
      <ShortcutRecorder
        label="Atalho Principal"
        value="Alt+Space"
        onChange={onChange}
      />
    );

    const recorderBtn = screen.getByRole("button", { name: "Atalho Principal" });
    fireEvent.click(recorderBtn);

    // Tecla sem modificador deve mostrar dica e não chamar onChange
    fireEvent.keyDown(recorderBtn, { key: "k", code: "KeyK" });
    expect(
      screen.getByText("Use ao menos um modificador (Ctrl, Alt ou Shift).")
    ).toBeInTheDocument();
    expect(onChange).not.toHaveBeenCalled();

    // Combinação Ctrl+Shift+K deve atualizar o atalho
    fireEvent.keyDown(recorderBtn, {
      key: "K",
      code: "KeyK",
      ctrlKey: true,
      shiftKey: true,
    });
    expect(onChange).toHaveBeenCalledWith("Ctrl+Shift+K");
  });
});
