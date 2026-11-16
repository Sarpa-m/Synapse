import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { GeneralTab } from "./GeneralTab";
import type { AppConfig } from "../../types/domain";

const BASE_CONFIG: AppConfig = {
  global_shortcut: "Alt+Space",
  compose_shortcut: "Ctrl+Alt+Space",
  undo_shortcut: "Ctrl+Alt+Z",
  base_url: "https://api.deepseek.com/v1",
  model: "deepseek-chat",
  timeout_secs: 10,
  retention_days: 30,
  launch_on_startup: false,
  selection_delivery_mode: "clipboard",
};

describe("GeneralTab — Atalhos, Entrega e Sistema", () => {
  it("bloqueia salvamento se atalho principal e de composição forem iguais", async () => {
    const onSave = vi.fn();
    render(
      <GeneralTab
        config={{ ...BASE_CONFIG, compose_shortcut: "Alt+Space" }}
        onChange={vi.fn()}
        onSave={onSave}
      />
    );

    fireEvent.click(
      screen.getByRole("button", { name: "Salvar Configurações Gerais" })
    );

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent(
      "Os atalhos Principal e de Composição não podem ser iguais."
    );
    expect(onSave).not.toHaveBeenCalled();
  });

  it("atualiza modo de entrega e toggle de inicialização com o Windows", async () => {
    const onChange = vi.fn();
    const onSave = vi.fn().mockResolvedValue(undefined);

    render(
      <GeneralTab config={BASE_CONFIG} onChange={onChange} onSave={onSave} />
    );

    fireEvent.change(screen.getByLabelText("Ao processar texto selecionado"), {
      target: { value: "paste_and_restore" },
    });
    expect(onChange).toHaveBeenCalledWith({
      ...BASE_CONFIG,
      selection_delivery_mode: "paste_and_restore",
    });

    fireEvent.click(
      screen.getByRole("switch", {
        name: "Iniciar Synapse automaticamente com o Windows",
      })
    );
    expect(onChange).toHaveBeenCalledWith({
      ...BASE_CONFIG,
      launch_on_startup: true,
    });

    fireEvent.click(
      screen.getByRole("button", { name: "Salvar Configurações Gerais" })
    );

    await waitFor(() => {
      expect(onSave).toHaveBeenCalledTimes(1);
    });
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Configurações salvas com sucesso."
    );
  });
});
