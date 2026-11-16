import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { ProfilesTab } from "./ProfilesTab";

describe("ProfilesTab — Validação de Formulário", () => {
  it("bloqueia criação de perfil se a temperatura for maior que 2.0 ou nome vazio", async () => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    const onDelete = vi.fn().mockResolvedValue(undefined);

    render(
      <ProfilesTab
        profiles={[]}
        onSaveProfile={onSave}
        onDeleteProfile={onDelete}
      />
    );

    const submitBtn = screen.getByRole("button", { name: "Adicionar Perfil" });

    // 1. Nome vazio
    fireEvent.click(submitBtn);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "O nome do perfil deve ter entre 1 e 60 caracteres."
    );
    expect(onSave).not.toHaveBeenCalled();

    // 2. Temperatura > 2.0
    fireEvent.change(screen.getByLabelText("Nome do Perfil"), {
      target: { value: "Perfil Inválido" },
    });
    fireEvent.change(screen.getByLabelText("System Prompt"), {
      target: { value: "Corrija o texto" },
    });
    fireEvent.change(screen.getByLabelText("Temperatura"), {
      target: { value: "2.5" },
    });

    fireEvent.click(submitBtn);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "A temperatura deve estar entre 0.0 e 2.0."
    );
    expect(onSave).not.toHaveBeenCalled();
  });

  it("permite editar um perfil existente preservando id e is_default", async () => {
    const onSave = vi.fn().mockResolvedValue(undefined);
    const onDelete = vi.fn().mockResolvedValue(undefined);

    render(
      <ProfilesTab
        profiles={[
          {
            id: "default-correcao",
            name: "Correção Silenciosa",
            system_prompt: "Corrija o texto",
            temperature: 0.1,
            shortcut: "1",
            is_default: true,
          },
        ]}
        onSaveProfile={onSave}
        onDeleteProfile={onDelete}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Editar" }));
    expect(screen.getByLabelText("Nome do Perfil")).toHaveValue(
      "Correção Silenciosa"
    );

    fireEvent.change(screen.getByLabelText("Nome do Perfil"), {
      target: { value: "Correção Rápida" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Salvar Alterações" }));

    await waitFor(() => {
      expect(onSave).toHaveBeenCalledWith({
        id: "default-correcao",
        name: "Correção Rápida",
        system_prompt: "Corrija o texto",
        temperature: 0.1,
        shortcut: "1",
        is_default: true,
      });
    });
  });
});
