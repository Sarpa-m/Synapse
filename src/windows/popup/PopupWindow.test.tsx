import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { PopupWindow } from "./PopupWindow";
import { tauriClient } from "../../services/tauriClient";

describe("PopupWindow — Navegação por Teclado e Resiliência", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("navega com ArrowDown e executa o 2º perfil ao pressionar Enter", async () => {
    const execSpy = vi
      .spyOn(tauriClient, "executeProfile")
      .mockResolvedValue("Texto reescrito");
    const closeSpy = vi.spyOn(tauriClient, "closePopup").mockResolvedValue();

    render(<PopupWindow />);

    const input = await screen.findByLabelText("Prompt ou busca de perfil");
    await screen.findByText("Correção Silenciosa");

    fireEvent.keyDown(input, { key: "ArrowDown" });
    fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() => {
      expect(execSpy).toHaveBeenCalledWith("default-reescrita-profissional");
      expect(closeSpy).toHaveBeenCalled();
    });
  });

  it("fecha imediatamente ao pressionar Escape", async () => {
    const closeSpy = vi.spyOn(tauriClient, "closePopup").mockResolvedValue();

    render(<PopupWindow />);
    const input = await screen.findByLabelText("Prompt ou busca de perfil");

    fireEvent.keyDown(input, { key: "Escape" });

    await waitFor(() => {
      expect(closeSpy).toHaveBeenCalledTimes(1);
    });
  });

  it("executa prompt avulso quando o texto digitado não corresponde a nenhum perfil", async () => {
    const customSpy = vi
      .spyOn(tauriClient, "executeCustomPrompt")
      .mockResolvedValue("Bonjour le monde");

    render(<PopupWindow />);
    const input = await screen.findByLabelText("Prompt ou busca de perfil");
    await screen.findByText("Correção Silenciosa");

    fireEvent.change(input, { target: { value: "traduza para frances" } });
    fireEvent.keyDown(input, { key: "Enter" });

    await waitFor(() => {
      expect(customSpy).toHaveBeenCalledWith("traduza para frances");
    });
  });

  it("exibe banner vermelho de erro e mantém o popup aberto se a LLM falhar", async () => {
    vi.spyOn(tauriClient, "executeProfile").mockRejectedValue(
      new Error("O provedor de IA excedeu o tempo limite de resposta (10s)")
    );
    const closeSpy = vi.spyOn(tauriClient, "closePopup").mockResolvedValue();

    render(<PopupWindow />);
    const input = await screen.findByLabelText("Prompt ou busca de perfil");
    await screen.findByText("Correção Silenciosa");

    fireEvent.keyDown(input, { key: "Enter" });

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent(
      "O provedor de IA excedeu o tempo limite de resposta (10s)"
    );
    expect(closeSpy).not.toHaveBeenCalled();
  });

  it("no modo composição envia texto digitado com Ctrl+Enter para copiar ao clipboard", async () => {
    vi.spyOn(tauriClient, "getPopupSession").mockResolvedValue({
      selected_text: "",
      char_count: 0,
      mode: "compose",
    });
    const composeSpy = vi
      .spyOn(tauriClient, "executeComposeProfile")
      .mockResolvedValue("Texto corrigido no clipboard");
    const closeSpy = vi.spyOn(tauriClient, "closePopup").mockResolvedValue();

    render(<PopupWindow />);

    const textarea = await screen.findByLabelText("Texto para processar");
    await screen.findByText("Correção Silenciosa");

    fireEvent.change(textarea, { target: { value: "ola mundo com erro" } });
    fireEvent.keyDown(textarea, { key: "Enter", ctrlKey: true });

    await waitFor(() => {
      expect(composeSpy).toHaveBeenCalledWith(
        "default-correcao-silenciosa",
        "ola mundo com erro"
      );
      expect(closeSpy).toHaveBeenCalled();
    });
  });
});
