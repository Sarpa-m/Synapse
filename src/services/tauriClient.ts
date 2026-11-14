import type {
  AppConfig,
  AppConfigView,
  HistoryEntry,
  PopupSessionView,
  Profile,
} from "../types/domain";

const DEFAULT_SEED_PROFILES: Profile[] = [
  {
    id: "default-correcao-silenciosa",
    name: "Correção Silenciosa",
    system_prompt:
      "Corrija erros gramaticais, ortográficos e de pontuação do texto a seguir, mantendo estritamente o tom, o idioma e a intenção original. Retorne APENAS o texto corrigido, sem explicações ou aspas.",
    temperature: 0.1,
    shortcut: "1",
    is_default: true,
  },
  {
    id: "default-reescrita-profissional",
    name: "Reescrita Profissional",
    system_prompt:
      "Reescreva o texto a seguir tornando-o claro, polido, cordial e adequado para comunicação profissional de trabalho (como e-mails ou mensagens corporativas). Retorne APENAS o texto reescrito, sem comentários adicionais.",
    temperature: 0.4,
    shortcut: "2",
    is_default: true,
  },
  {
    id: "default-traducao-en-pt",
    name: "Tradução EN-PT",
    system_prompt:
      "Se o texto a seguir estiver em inglês (ou outro idioma), traduza-o naturalmente para o português brasileiro. Se já estiver em português, traduza-o para o inglês. Retorne APENAS a tradução.",
    temperature: 0.2,
    shortcut: "3",
    is_default: true,
  },
];

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function safeInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
  fallback?: () => T | Promise<T>
): Promise<T> {
  if (isTauriRuntime()) {
    const { invoke } = await import("@tauri-apps/api/core");
    return invoke<T>(cmd, args);
  }
  if (fallback) {
    return fallback();
  }
  throw new Error(`Tauri IPC command '${cmd}' indisponível fora do runtime Tauri`);
}

export const SESSION_STARTED_EVENT = "synapse://session-started";

export const tauriClient = {
  /** Assina o evento emitido pelo backend quando o atalho global captura um novo texto. */
  async onSessionStarted(
    handler: (session: PopupSessionView) => void
  ): Promise<() => void> {
    if (!isTauriRuntime()) return () => undefined;
    const { listen } = await import("@tauri-apps/api/event");
    return listen<PopupSessionView>(SESSION_STARTED_EVENT, (event) =>
      handler(event.payload)
    );
  },
  async listProfiles(): Promise<Profile[]> {
    return safeInvoke("list_profiles_cmd", undefined, () => DEFAULT_SEED_PROFILES);
  },

  async saveProfile(profile: Profile): Promise<Profile> {
    return safeInvoke("save_profile_cmd", { profile }, () => profile);
  },

  async deleteProfile(id: string): Promise<void> {
    return safeInvoke("delete_profile_cmd", { id }, () => undefined);
  },

  async restoreDefaultProfiles(): Promise<Profile[]> {
    return safeInvoke(
      "restore_default_profiles_cmd",
      undefined,
      () => DEFAULT_SEED_PROFILES
    );
  },

  async getPopupSession(): Promise<PopupSessionView | null> {
    return safeInvoke("get_current_popup_session", undefined, () => ({
      selected_text: "Prezados, nois vai mandar o relatorio ate o final do dia.",
      char_count: 58,
      mode: "selection" as const,
    }));
  },

  async executeProfile(profileId: string): Promise<string> {
    return safeInvoke(
      "execute_profile_cmd",
      { profileId },
      () => "Prezados, enviaremos o relatório até o final do dia."
    );
  },

  async executeCustomPrompt(instruction: string): Promise<string> {
    return safeInvoke(
      "execute_custom_prompt_cmd",
      { instruction },
      () => `[Processado: ${instruction}]`
    );
  },

  async executeComposeProfile(
    profileId: string,
    inputText: string
  ): Promise<string> {
    return safeInvoke(
      "execute_compose_profile_cmd",
      { profileId, inputText },
      () => `[Copiado para Ctrl+V: ${inputText}]`
    );
  },

  async closePopup(): Promise<void> {
    return safeInvoke("cancel_popup_session_cmd", undefined, () => undefined);
  },

  async getSettings(): Promise<AppConfigView> {
    return safeInvoke("get_settings_cmd", undefined, () => ({
      global_shortcut: "Alt+Space",
      compose_shortcut: "Ctrl+Alt+Space",
      undo_shortcut: "Ctrl+Alt+Z",
      launch_on_startup: false,
      base_url: "https://api.deepseek.com/v1",
      model: "deepseek-chat",
      timeout_secs: 10,
      retention_days: 30,
      has_api_key: true,
    }));
  },

  async saveSettings(config: AppConfig): Promise<AppConfigView> {
    return safeInvoke("save_settings_cmd", { config }, () => ({
      ...config,
      has_api_key: true,
    }));
  },

  async saveApiKey(apiKey: string): Promise<AppConfigView> {
    return safeInvoke("save_api_key_cmd", { apiKey }, async () => {
      const current = await tauriClient.getSettings();
      return { ...current, has_api_key: apiKey.trim().length > 0 };
    });
  },

  async clearApiKey(): Promise<AppConfigView> {
    return safeInvoke("clear_api_key_cmd", undefined, async () => {
      const current = await tauriClient.getSettings();
      return { ...current, has_api_key: false };
    });
  },

  async listHistory(limit = 100): Promise<HistoryEntry[]> {
    return safeInvoke("list_history_cmd", { limit }, () => []);
  },

  async clearHistory(): Promise<void> {
    return safeInvoke("clear_history_cmd", undefined, () => undefined);
  },
};
