import React, { useState } from "react";
import { Eye, EyeOff, KeyRound, ShieldCheck } from "lucide-react";
import type { AppConfigView } from "../../types/domain";
import { StatusBanner } from "../../components/ui/StatusBanner";

interface ApiProviderTabProps {
  config: AppConfigView;
  onChangeConfig: (updated: AppConfigView) => void;
  onSaveConfig: () => Promise<void> | void;
  onSaveApiKey: (apiKey: string) => Promise<void> | void;
}

const PROVIDER_PRESETS = [
  {
    id: "deepseek",
    name: "DeepSeek",
    subtitle: "Rápido e econômico",
    baseUrl: "https://api.deepseek.com/v1",
    model: "deepseek-chat",
  },
  {
    id: "openai",
    name: "OpenAI",
    subtitle: "GPT-4o mini",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4o-mini",
  },
  {
    id: "ollama",
    name: "Ollama Local",
    subtitle: "100% offline",
    baseUrl: "http://localhost:11434/v1",
    model: "llama3.2",
  },
];

export const ApiProviderTab: React.FC<ApiProviderTabProps> = ({
  config,
  onChangeConfig,
  onSaveConfig,
  onSaveApiKey,
}) => {
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [showKey, setShowKey] = useState(false);
  const [status, setStatus] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  const applyPreset = (baseUrl: string, model: string) => {
    onChangeConfig({ ...config, base_url: baseUrl, model });
  };

  const handleSaveKey = async () => {
    if (!apiKeyInput.trim()) return;
    try {
      await onSaveApiKey(apiKeyInput);
      setApiKeyInput("");
      setStatus({
        type: "success",
        text: "API Key salva com segurança no Windows Credential Manager.",
      });
    } catch (err: unknown) {
      setStatus({
        type: "error",
        text: err instanceof Error ? err.message : String(err),
      });
    }
  };

  const handleSaveConfig = async () => {
    try {
      await onSaveConfig();
      setStatus({
        type: "success",
        text: "Configurações do provedor salvas.",
      });
    } catch (err: unknown) {
      setStatus({
        type: "error",
        text: err instanceof Error ? err.message : String(err),
      });
    }
  };

  return (
    <div className="space-y-6">
      {/* Presets rápidos */}
      <section className="rounded-xl border border-white/[0.08] bg-[#12151B] p-5 space-y-4">
        <div>
          <h3 className="text-sm font-semibold text-white">
            Provedor Compatível com OpenAI
          </h3>
          <p className="text-xs text-zinc-400 mt-0.5">
            Escolha um preset ou informe qualquer endpoint{" "}
            <code className="text-zinc-300">/v1/chat/completions</code>.
          </p>
        </div>

        <div className="grid grid-cols-3 gap-2.5">
          {PROVIDER_PRESETS.map((preset) => {
            const isSelected = config.base_url === preset.baseUrl;
            return (
              <button
                key={preset.id}
                type="button"
                onClick={() => applyPreset(preset.baseUrl, preset.model)}
                className={`p-3 rounded-lg border text-left synapse-interactive synapse-pressable ${
                  isSelected
                    ? "bg-sky-500/15 border-sky-400/50 text-white"
                    : "bg-[#0B0D10] border-white/10 text-zinc-300 hover:border-white/20"
                }`}
              >
                <div className="text-xs font-semibold">{preset.name}</div>
                <div className="text-[11px] text-zinc-400 mt-0.5">
                  {preset.subtitle}
                </div>
              </button>
            );
          })}
        </div>

        <div className="grid grid-cols-2 gap-3 pt-1">
          <div>
            <label className="block text-xs font-medium text-zinc-300 mb-1.5">
              Base URL (Compatível com OpenAI)
            </label>
            <input
              type="text"
              aria-label="Base URL"
              value={config.base_url}
              onChange={(e) =>
                onChangeConfig({ ...config, base_url: e.target.value })
              }
              className="w-full px-3 py-2 rounded-lg bg-[#0B0D10] border border-white/10 text-sm font-mono text-white synapse-interactive focus:outline-none focus:border-sky-400/60"
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-zinc-300 mb-1.5">
              Modelo
            </label>
            <input
              type="text"
              aria-label="Modelo"
              value={config.model}
              onChange={(e) =>
                onChangeConfig({ ...config, model: e.target.value })
              }
              className="w-full px-3 py-2 rounded-lg bg-[#0B0D10] border border-white/10 text-sm font-mono text-white synapse-interactive focus:outline-none focus:border-sky-400/60"
            />
          </div>
        </div>

        <div className="flex justify-end pt-1">
          <button
            type="button"
            onClick={handleSaveConfig}
            className="px-4 py-2 rounded-lg bg-sky-500 hover:bg-sky-400 text-xs font-semibold text-slate-950 synapse-pressable"
          >
            Salvar Provedor
          </button>
        </div>
      </section>

      {/* Cofre de Credenciais */}
      <section className="rounded-xl border border-white/[0.08] bg-[#12151B] p-5 space-y-3.5">
        <div className="flex items-start justify-between gap-3">
          <div>
            <h3 className="text-sm font-semibold text-white flex items-center gap-2">
              <KeyRound className="w-4 h-4 text-sky-400" />
              Cofre de Credenciais
            </h3>
            <p className="text-xs text-zinc-400 mt-0.5">
              Armazenada exclusivamente no Windows Credential Manager (nunca no SQLite).
            </p>
          </div>
          {config.has_api_key && (
            <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full bg-emerald-950/60 border border-emerald-500/30 text-[11px] font-medium text-emerald-300 shrink-0">
              <ShieldCheck className="w-3.5 h-3.5" />
              Configurada
            </span>
          )}
        </div>

        <div className="flex gap-2">
          <div className="relative flex-1">
            <input
              type={showKey ? "text" : "password"}
              aria-label="API Key"
              placeholder={
                config.has_api_key
                  ? "Digite uma nova chave para substituir (sk-••••)"
                  : "sk-..."
              }
              value={apiKeyInput}
              onChange={(e) => setApiKeyInput(e.target.value)}
              className="w-full pl-3 pr-9 py-2 rounded-lg bg-[#0B0D10] border border-white/10 text-sm font-mono text-white placeholder-zinc-500 synapse-interactive focus:outline-none focus:border-sky-400/60"
            />
            <button
              type="button"
              aria-label={showKey ? "Ocultar chave" : "Mostrar chave"}
              onClick={() => setShowKey((prev) => !prev)}
              className="absolute right-2.5 top-1/2 -translate-y-1/2 text-zinc-400 hover:text-zinc-200"
            >
              {showKey ? (
                <EyeOff className="w-4 h-4" />
              ) : (
                <Eye className="w-4 h-4" />
              )}
            </button>
          </div>

          <button
            type="button"
            disabled={!apiKeyInput.trim()}
            onClick={handleSaveKey}
            className="px-4 py-2 rounded-lg bg-emerald-500 hover:bg-emerald-400 disabled:opacity-40 text-xs font-semibold text-slate-950 synapse-pressable"
          >
            Salvar no Cofre
          </button>
        </div>
      </section>

      {status && <StatusBanner type={status.type} message={status.text} />}
    </div>
  );
};
