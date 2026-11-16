import React, { useEffect, useState } from "react";
import type { AppConfig, SelectionDeliveryMode } from "../../types/domain";
import { tauriClient } from "../../services/tauriClient";
import { ShortcutRecorder } from "./ShortcutRecorder";
import { ToggleSwitch } from "../../components/ui/ToggleSwitch";
import { StatusBanner } from "../../components/ui/StatusBanner";

interface GeneralTabProps {
  config: AppConfig;
  onChange: (updated: AppConfig) => void;
  onSave: () => Promise<void> | void;
}

const DELIVERY_DESCRIPTIONS: Record<SelectionDeliveryMode, string> = {
  clipboard:
    "Copia o resultado para a área de transferência para você colar manualmente com Ctrl+V.",
  paste_and_keep:
    "Restaura o foco, cola automaticamente no app de origem e mantém a nova versão no Ctrl+V.",
  paste_and_restore:
    "Restaura o foco, cola automaticamente e devolve o seu clipboard anterior logo em seguida.",
};

export const GeneralTab: React.FC<GeneralTabProps> = ({
  config,
  onChange,
  onSave,
}) => {
  const [status, setStatus] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [appVersion, setAppVersion] = useState<string>("0.1.0");
  const [isCheckingUpdate, setIsCheckingUpdate] = useState(false);

  useEffect(() => {
    void tauriClient.getAppVersion().then(setAppVersion);
  }, []);

  const handleManualUpdateCheck = async () => {
    setIsCheckingUpdate(true);
    setStatus(null);
    try {
      const res = await tauriClient.checkForUpdates(true);
      if (res.status === "update_available") {
        setStatus({
          type: "success",
          text: `Nova versão v${res.release.version} encontrada! A janela de atualização foi aberta.`,
        });
      } else {
        setStatus({
          type: "success",
          text: `Você já está na versão mais recente (v${res.current_version}).`,
        });
      }
    } catch (err: unknown) {
      const msg =
        err instanceof Error
          ? err.message
          : typeof err === "object" && err !== null && "message" in err
            ? String((err as { message: unknown }).message)
            : String(err);
      setStatus({ type: "error", text: msg });
    } finally {
      setIsCheckingUpdate(false);
    }
  };

  const handleSave = async () => {
    if (config.global_shortcut === config.compose_shortcut) {
      setStatus({
        type: "error",
        text: "Os atalhos Principal e de Composição não podem ser iguais.",
      });
      return;
    }

    setIsSaving(true);
    setStatus(null);
    try {
      await onSave();
      setStatus({ type: "success", text: "Configurações salvas com sucesso." });
    } catch (err: unknown) {
      const msg =
        err instanceof Error
          ? err.message
          : typeof err === "object" && err !== null && "message" in err
            ? String((err as { message: unknown }).message)
            : String(err);
      setStatus({ type: "error", text: msg });
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Seção 1: Atalhos Globais */}
      <section className="rounded-xl border border-white/[0.08] bg-[#12151B] p-5 space-y-4">
        <div>
          <h3 className="text-sm font-semibold text-white">Atalhos Globais</h3>
          <p className="text-xs text-zinc-400 mt-0.5">
            Disparam o Synapse instantaneamente a partir de qualquer aplicativo.
          </p>
        </div>

        <div className="space-y-4">
          <ShortcutRecorder
            label="Atalho Global Principal (Popup Contextual)"
            value={config.global_shortcut}
            onChange={(global_shortcut) =>
              onChange({ ...config, global_shortcut })
            }
          />

          <ShortcutRecorder
            label="Atalho de Composição (Digitar → Perfil → Ctrl+V)"
            value={config.compose_shortcut}
            onChange={(compose_shortcut) =>
              onChange({ ...config, compose_shortcut })
            }
          />

          <ShortcutRecorder
            label="Atalho de Undo Rápido"
            value={config.undo_shortcut}
            onChange={(undo_shortcut) =>
              onChange({ ...config, undo_shortcut })
            }
          />
        </div>
      </section>

      {/* Seção 2: Entrega e Sistema */}
      <section className="rounded-xl border border-white/[0.08] bg-[#12151B] p-5 space-y-5">
        <div>
          <h3 className="text-sm font-semibold text-white">
            Entrega e Comportamento
          </h3>
          <p className="text-xs text-zinc-400 mt-0.5">
            Controle como o texto gerado retorna ao aplicativo e a retenção local.
          </p>
        </div>

        <div className="space-y-1.5">
          <label className="block text-xs font-medium text-zinc-300">
            Ao processar texto selecionado
          </label>
          <select
            aria-label="Ao processar texto selecionado"
            value={config.selection_delivery_mode}
            onChange={(e) =>
              onChange({
                ...config,
                selection_delivery_mode: e.target.value as SelectionDeliveryMode,
              })
            }
            className="w-full px-3 py-2 rounded-lg bg-[#0B0D10] border border-white/10 text-sm text-white synapse-interactive focus:outline-none focus:border-sky-400/60"
          >
            <option value="clipboard">
              Salvar nova versão na área de transferência (Ctrl+V)
            </option>
            <option value="paste_and_keep">
              Colar automaticamente e manter nova versão no Ctrl+V
            </option>
            <option value="paste_and_restore">
              Colar automaticamente e restaurar clipboard antigo
            </option>
          </select>
          <p className="text-xs text-zinc-400 leading-relaxed">
            {DELIVERY_DESCRIPTIONS[config.selection_delivery_mode]}
          </p>
        </div>

        <div className="flex items-center justify-between gap-4 pt-2 border-t border-white/[0.06]">
          <div>
            <label className="block text-sm font-medium text-zinc-200">
              Retenção do Histórico (dias)
            </label>
            <span className="block text-xs text-zinc-400 mt-0.5">
              Registros mais antigos são removidos automaticamente no boot.
            </span>
          </div>
          <input
            type="number"
            min={1}
            max={365}
            aria-label="Retenção do Histórico"
            value={config.retention_days}
            onChange={(e) =>
              onChange({ ...config, retention_days: Number(e.target.value) })
            }
            className="w-24 px-3 py-1.5 rounded-lg bg-[#0B0D10] border border-white/10 text-sm font-mono text-white text-center synapse-interactive focus:outline-none focus:border-sky-400/60"
          />
        </div>

        <div className="pt-2 border-t border-white/[0.06]">
          <ToggleSwitch
            label="Iniciar Synapse automaticamente com o Windows"
            description="Mantém o utilitário silencioso na bandeja do sistema após o login."
            checked={config.launch_on_startup}
            onChange={(launch_on_startup) =>
              onChange({ ...config, launch_on_startup })
            }
          />
        </div>

        <div className="pt-3 border-t border-white/[0.06] space-y-3">
          <ToggleSwitch
            label="Verificar atualizações automaticamente ao iniciar"
            description="Consulta o GitHub Releases na inicialização e solicita sua permissão antes de atualizar."
            checked={config.auto_check_updates ?? true}
            onChange={(auto_check_updates) =>
              onChange({ ...config, auto_check_updates })
            }
          />

          <div className="flex items-center justify-between gap-4 pt-1">
            <span className="text-xs text-zinc-400 font-mono">
              Versão instalada: v{appVersion}
            </span>
            <button
              type="button"
              disabled={isCheckingUpdate}
              onClick={handleManualUpdateCheck}
              className="px-3.5 py-1.5 rounded-lg bg-[#0B0D10] hover:bg-white/[0.06] border border-white/10 text-xs font-medium text-zinc-200 disabled:opacity-50 synapse-interactive"
            >
              {isCheckingUpdate ? "Verificando..." : "Verificar agora"}
            </button>
          </div>
        </div>
      </section>

      {status && <StatusBanner type={status.type} message={status.text} />}

      <div className="flex justify-end">
        <button
          type="button"
          disabled={isSaving}
          onClick={handleSave}
          className="px-5 py-2.5 rounded-lg bg-sky-500 hover:bg-sky-400 disabled:opacity-50 text-xs font-semibold text-slate-950 synapse-pressable"
        >
          {isSaving ? "Salvando..." : "Salvar Configurações Gerais"}
        </button>
      </div>
    </div>
  );
};
