import React, { useEffect, useState } from "react";
import {
  History,
  KeyRound,
  ShieldAlert,
  ShieldCheck,
  SlidersHorizontal,
  Sparkles,
} from "lucide-react";
import type { AppConfigView, HistoryEntry, Profile } from "../../types/domain";
import { tauriClient } from "../../services/tauriClient";
import { GeneralTab } from "./GeneralTab";
import { ApiProviderTab } from "./ApiProviderTab";
import { ProfilesTab } from "./ProfilesTab";
import { HistoryTab } from "./HistoryTab";

type TabId = "general" | "api" | "profiles" | "history";

const NAV_ITEMS: Array<{
  id: TabId;
  label: string;
  description: string;
  icon: React.ComponentType<{ className?: string }>;
}> = [
  {
    id: "general",
    label: "Geral",
    description: "Atalhos, entrega e sistema",
    icon: SlidersHorizontal,
  },
  {
    id: "api",
    label: "Provedores e API",
    description: "Modelo e cofre de chaves",
    icon: KeyRound,
  },
  {
    id: "profiles",
    label: "Perfis",
    description: "Prompts e temperatura",
    icon: Sparkles,
  },
  {
    id: "history",
    label: "Histórico",
    description: "Cofre de textos gerados",
    icon: History,
  },
];

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
    return (
      <div className="min-h-screen bg-[#0B0D10] flex items-center justify-center text-sm text-zinc-400">
        Carregando configurações...
      </div>
    );
  }

  const activeNav = NAV_ITEMS.find((item) => item.id === activeTab)!;

  return (
    <div className="h-screen flex bg-[#0B0D10] text-zinc-100 overflow-hidden">
      {/* Sidebar fixa (220px) */}
      <aside className="w-[220px] shrink-0 flex flex-col justify-between bg-[#12151B] border-r border-white/[0.08] p-4">
        <div className="space-y-6">
          <div className="px-2 pt-1">
            <div className="flex items-center gap-2">
              <div className="w-7 h-7 rounded-lg bg-sky-500/15 border border-sky-400/30 flex items-center justify-center">
                <Sparkles className="w-4 h-4 text-sky-400" />
              </div>
              <div>
                <h1 className="text-sm font-semibold text-white tracking-tight">
                  Synapse
                </h1>
                <p className="text-[11px] text-zinc-400">Assistente Desktop</p>
              </div>
            </div>
          </div>

          <nav aria-label="Seções de configuração" className="space-y-1">
            {NAV_ITEMS.map((item) => {
              const Icon = item.icon;
              const isActive = activeTab === item.id;
              return (
                <button
                  key={item.id}
                  type="button"
                  onClick={() => setActiveTab(item.id)}
                  className={`w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-left text-xs font-medium synapse-interactive ${
                    isActive
                      ? "bg-sky-500/15 text-sky-200 ring-1 ring-inset ring-sky-400/30"
                      : "text-zinc-400 hover:text-zinc-100 hover:bg-white/[0.04]"
                  }`}
                >
                  <Icon
                    className={`w-4 h-4 shrink-0 ${
                      isActive ? "text-sky-400" : "text-zinc-500"
                    }`}
                  />
                  <span className="truncate">{item.label}</span>
                </button>
              );
            })}
          </nav>
        </div>

        {/* Status rápido do cofre */}
        <button
          type="button"
          onClick={() => setActiveTab("api")}
          className={`w-full flex items-center gap-2.5 px-3 py-2.5 rounded-lg border text-left synapse-interactive ${
            config.has_api_key
              ? "bg-emerald-950/35 border-emerald-500/25 text-emerald-300 hover:bg-emerald-950/50"
              : "bg-amber-950/35 border-amber-500/30 text-amber-300 hover:bg-amber-950/50"
          }`}
        >
          {config.has_api_key ? (
            <ShieldCheck className="w-4 h-4 shrink-0 text-emerald-400" />
          ) : (
            <ShieldAlert className="w-4 h-4 shrink-0 text-amber-400" />
          )}
          <div className="min-w-0">
            <div className="text-[11px] font-medium truncate">
              {config.has_api_key ? "API Key protegida" : "API Key pendente"}
            </div>
            <div className="text-[10px] opacity-75 truncate font-mono">
              {config.model}
            </div>
          </div>
        </button>
      </aside>

      {/* Canvas de conteúdo rolável */}
      <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
        <header className="px-7 py-5 border-b border-white/[0.08] bg-[#0B0D10]/80 shrink-0">
          <h2 className="text-base font-semibold text-white">
            {activeNav.label}
          </h2>
          <p className="text-xs text-zinc-400 mt-0.5">
            {activeNav.description}
          </p>
        </header>

        <main className="flex-1 overflow-y-auto synapse-scroll px-7 py-6">
          <div key={activeTab} className="max-w-2xl synapse-panel-enter">
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
          </div>
        </main>
      </div>
    </div>
  );
};
