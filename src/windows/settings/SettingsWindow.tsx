import React, { useEffect, useState } from "react";
import type { AppConfigView, HistoryEntry, Profile } from "../../types/domain";
import { tauriClient } from "../../services/tauriClient";
import { GeneralTab } from "./GeneralTab";
import { ApiProviderTab } from "./ApiProviderTab";
import { ProfilesTab } from "./ProfilesTab";
import { HistoryTab } from "./HistoryTab";

type TabId = "general" | "api" | "profiles" | "history";

export const SettingsWindow: React.FC = () => {
  const [activeTab, setActiveTab] = useState<TabId>("general");
  const [config, setConfig] = useState<AppConfigView | null>(null);
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [history, setHistory] = useState<HistoryEntry[]>([]);

  useEffect(() => {
    Promise.all([
      tauriClient.getSettings(),
      tauriClient.listProfiles(),
      tauriClient.listHistory(),
    ]).then(([cfg, profs, hist]) => {
      setConfig(cfg);
      setProfiles(profs);
      setHistory(hist);
    });
  }, []);

  if (!config) {
    return <div className="p-6 text-sm text-slate-400">Carregando configurações...</div>;
  }

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 p-6">
      <header className="mb-6 border-b border-slate-800 pb-4">
        <h1 className="text-lg font-bold text-white">Synapse Desktop — Configurações</h1>
        <p className="text-xs text-slate-400">
          Assistência Textual de IA em Background para Windows
        </p>
      </header>

      <nav className="flex gap-2 mb-6 border-b border-slate-800 pb-2">
        {(
          [
            ["general", "Geral"],
            ["api", "Provedores e API"],
            ["profiles", "Perfis"],
            ["history", "Histórico (Cofre)"],
          ] as const
        ).map(([id, label]) => (
          <button
            key={id}
            type="button"
            onClick={() => setActiveTab(id)}
            className={`px-3 py-1.5 rounded text-xs font-medium ${
              activeTab === id
                ? "bg-indigo-600 text-white"
                : "text-slate-400 hover:bg-slate-800"
            }`}
          >
            {label}
          </button>
        ))}
      </nav>

      <main>
        {activeTab === "general" && (
          <GeneralTab
            config={config}
            onChange={(updated) =>
              setConfig({ ...updated, has_api_key: config.has_api_key })
            }
            onSave={async () => {
              const saved = await tauriClient.saveSettings(config);
              setConfig(saved);
            }}
          />
        )}

        {activeTab === "api" && (
          <ApiProviderTab
            config={config}
            onChangeConfig={setConfig}
            onSaveConfig={async () => {
              const saved = await tauriClient.saveSettings(config);
              setConfig(saved);
            }}
            onSaveApiKey={async (key) => {
              const saved = await tauriClient.saveApiKey(key);
              setConfig(saved);
            }}
          />
        )}

        {activeTab === "profiles" && (
          <ProfilesTab
            profiles={profiles}
            onSaveProfile={async (profile) => {
              await tauriClient.saveProfile(profile);
              setProfiles(await tauriClient.listProfiles());
            }}
            onDeleteProfile={async (id) => {
              await tauriClient.deleteProfile(id);
              setProfiles(await tauriClient.listProfiles());
            }}
            onRestoreDefaults={async () => {
              const restored = await tauriClient.restoreDefaultProfiles();
              setProfiles(restored);
            }}
          />
        )}

        {activeTab === "history" && (
          <HistoryTab
            entries={history}
            onClearHistory={async () => {
              await tauriClient.clearHistory();
              setHistory([]);
            }}
          />
        )}
      </main>
    </div>
  );
};
