import { describe, expect, it, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { UpdateWindow } from "./UpdateWindow";
import { tauriClient } from "../../services/tauriClient";
import type { UpdateReleaseInfo } from "../../types/domain";

vi.mock("../../services/tauriClient", () => ({
  tauriClient: {
    getAppVersion: vi.fn(),
    getPendingUpdate: vi.fn(),
    onUpdateAvailable: vi.fn(),
    onUpdateProgress: vi.fn(),
    installUpdate: vi.fn(),
    dismissUpdatePrompt: vi.fn(),
    skipUpdateVersion: vi.fn(),
  },
}));

const SAMPLE_RELEASE: UpdateReleaseInfo = {
  version: "0.2.0",
  tag_name: "v0.2.0",
  release_notes: "Correção de bugs e melhorias no auto-update",
  published_at: "2026-11-16T12:00:00Z",
  download_url: "https://github.com/Sarpa-m/Synapse/releases/download/v0.2.0/Synapse-setup.exe",
  asset_name: "Synapse-setup.exe",
  asset_size_bytes: 5 * 1024 * 1024,
};

describe("UpdateWindow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(tauriClient.getAppVersion).mockResolvedValue("0.1.0");
    vi.mocked(tauriClient.getPendingUpdate).mockResolvedValue(SAMPLE_RELEASE);
    vi.mocked(tauriClient.onUpdateAvailable).mockResolvedValue(() => undefined);
    vi.mocked(tauriClient.onUpdateProgress).mockResolvedValue(() => undefined);
    vi.mocked(tauriClient.installUpdate).mockResolvedValue(undefined);
    vi.mocked(tauriClient.dismissUpdatePrompt).mockResolvedValue(undefined);
    vi.mocked(tauriClient.skipUpdateVersion).mockResolvedValue(undefined);
  });

  it("exibe comparativo de versões, notas de lançamento e solicita permissão ao usuário", async () => {
    render(<UpdateWindow />);

    await waitFor(() => {
      expect(
        screen.getByText("Nova versão do Synapse disponível")
      ).toBeInTheDocument();
    });

    expect(screen.getByText("v0.1.0")).toBeInTheDocument();
    expect(screen.getByText("v0.2.0")).toBeInTheDocument();
    expect(
      screen.getByText("Correção de bugs e melhorias no auto-update")
    ).toBeInTheDocument();
  });

  it("inicia instalação apenas quando o usuário clica em Permitir e Atualizar", async () => {
    const user = userEvent.setup();
    render(<UpdateWindow />);

    const btn = await screen.findByRole("button", {
      name: /Permitir e Atualizar/i,
    });
    await user.click(btn);

    expect(tauriClient.installUpdate).toHaveBeenCalledWith(SAMPLE_RELEASE);
  });

  it("permite adiar ('Mais tarde') ou ignorar a versão sem instalar nada", async () => {
    const user = userEvent.setup();
    render(<UpdateWindow />);

    const laterBtn = await screen.findByRole("button", { name: /Mais tarde/i });
    await user.click(laterBtn);
    expect(tauriClient.dismissUpdatePrompt).toHaveBeenCalledTimes(1);
    expect(tauriClient.installUpdate).not.toHaveBeenCalled();

    const skipBtn = screen.getByRole("button", {
      name: /Ignorar esta versão/i,
    });
    await user.click(skipBtn);
    expect(tauriClient.skipUpdateVersion).toHaveBeenCalledWith("0.2.0");
  });
});
