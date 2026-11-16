import React, { useState } from "react";
import type { AppConfig, SelectionDeliveryMode } from "../../types/domain";
import { ShortcutRecorder } from "./ShortcutRecorder";

interface GeneralTabProps {
  config: AppConfig;
  onChange: (updated: AppConfig) => void;
  onSave: () => Promise<void> | void;
}

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
    <div className="space-y-4">
      <ShortcutRecorder
        label="Atalho Global Principal (Popup Contextual)"
        value={config.global_shortcut}
        onChange={(global_shortcut) => onChange({ ...config, global_shortcut })}
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
        onChange={(undo_shortcut) => onChange({ ...config, undo_shortcut })}
      />

      <div>
        <label className="block text-xs font-medium text-slate-300 mb-1">
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
          className="w-full px-3 py-2 rounded bg-slate-800 border border-slate-700 text-sm text-white"
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
      </div>

      <div>
        <label className="block text-xs font-medium text-slate-300 mb-1">
          Retenção do Histórico (dias)
        </label>
        <input
          type="number"
          min={1}
          max={365}
          aria-label="Retenção do Histórico"
          value={config.retention_days}
          onChange={(e) =>
            onChange({ ...config, retention_days: Number(e.target.value) })
          }
          className="w-40 px-3 py-2 rounded bg-slate-800 border border-slate-700 text-sm text-white"
        />
      </div>

      <label className="flex items-center gap-2 text-sm text-slate-200 cursor-pointer">
        <input
          type="checkbox"
          checked={config.launch_on_startup}
          onChange={(e) =>
            onChange({ ...config, launch_on_startup: e.target.checked })
          }
        />
        Iniciar Synapse automaticamente com o Windows
      </label>

      {status && (
        <div
          role={status.type === "error" ? "alert" : "status"}
          className={`px-3 py-2 rounded text-xs ${
            status.type === "error"
              ? "bg-red-950/60 border border-red-800 text-red-200"
              : "bg-emerald-950/60 border border-emerald-800 text-emerald-200"
          }`}
        >
          {status.text}
        </div>
      )}

      <button
        type="button"
        disabled={isSaving}
        onClick={handleSave}
        className="px-4 py-2 rounded bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-sm font-medium text-white"
      >
        {isSaving ? "Salvando..." : "Salvar Configurações Gerais"}
      </button>
    </div>
  );
};
