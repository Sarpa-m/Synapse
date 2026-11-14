import React, { useState } from "react";
import type { AppConfigView } from "../../types/domain";

interface ApiProviderTabProps {
  config: AppConfigView;
  onChangeConfig: (updated: AppConfigView) => void;
  onSaveConfig: () => void;
  onSaveApiKey: (apiKey: string) => void;
}

export const ApiProviderTab: React.FC<ApiProviderTabProps> = ({
  config,
  onChangeConfig,
  onSaveConfig,
  onSaveApiKey,
}) => {
  const [apiKeyInput, setApiKeyInput] = useState("");

  const applyPreset = (baseUrl: string, model: string) => {
    onChangeConfig({ ...config, base_url: baseUrl, model });
  };

  return (
    <div className="space-y-4">
      <div className="flex gap-2">
        <button
          type="button"
          onClick={() => applyPreset("https://api.deepseek.com/v1", "deepseek-chat")}
          className="px-2.5 py-1 text-xs rounded bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700"
        >
          Preset: DeepSeek
        </button>
        <button
          type="button"
          onClick={() => applyPreset("https://api.openai.com/v1", "gpt-4o-mini")}
          className="px-2.5 py-1 text-xs rounded bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700"
        >
          Preset: OpenAI
        </button>
        <button
          type="button"
          onClick={() => applyPreset("http://localhost:11434/v1", "llama3.2")}
          className="px-2.5 py-1 text-xs rounded bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700"
        >
          Preset: Ollama Local
        </button>
      </div>

      <div>
        <label className="block text-xs font-medium text-slate-300 mb-1">
          Base URL (Compatível com OpenAI)
        </label>
        <input
          type="text"
          aria-label="Base URL"
          value={config.base_url}
          onChange={(e) => onChangeConfig({ ...config, base_url: e.target.value })}
          className="w-full px-3 py-2 rounded bg-slate-800 border border-slate-700 text-sm text-white"
        />
      </div>

      <div>
        <label className="block text-xs font-medium text-slate-300 mb-1">
          Modelo
        </label>
        <input
          type="text"
          aria-label="Modelo"
          value={config.model}
          onChange={(e) => onChangeConfig({ ...config, model: e.target.value })}
          className="w-full px-3 py-2 rounded bg-slate-800 border border-slate-700 text-sm text-white"
        />
      </div>

      <div className="pt-2 border-t border-slate-800">
        <label className="block text-xs font-medium text-slate-300 mb-1">
          API Key (Armazenada exclusivamente no Windows Credential Manager)
        </label>
        <p className="text-xs text-emerald-400 mb-2">
          {config.has_api_key
            ? "✓ Chave configurada com segurança no cofre do Windows (sk-••••)"
            : "⚠ Nenhuma chave configurada no cofre"}
        </p>
        <div className="flex gap-2">
          <input
            type="password"
            aria-label="API Key"
            placeholder="sk-..."
            value={apiKeyInput}
            onChange={(e) => setApiKeyInput(e.target.value)}
            className="flex-1 px-3 py-2 rounded bg-slate-800 border border-slate-700 text-sm text-white"
          />
          <button
            type="button"
            onClick={() => {
              onSaveApiKey(apiKeyInput);
              setApiKeyInput("");
            }}
            className="px-4 py-2 rounded bg-emerald-600 hover:bg-emerald-500 text-sm font-medium text-white"
          >
            Salvar no Cofre
          </button>
        </div>
      </div>

      <button
        type="button"
        onClick={onSaveConfig}
        className="px-4 py-2 rounded bg-indigo-600 hover:bg-indigo-500 text-sm font-medium text-white"
      >
        Salvar Provedor
      </button>
    </div>
  );
};
