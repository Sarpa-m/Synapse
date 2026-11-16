import type {
  AppConfig,
  AppConfigView,
  HistoryEntry,
  PopupSessionView,
  Profile,
  UpdateCheckResult,
  UpdateProgress,
  UpdateReleaseInfo,
} from "../types/domain";

const DEFAULT_SEED_PROFILES: Profile[] = [
  {
    id: "default-correcao-silenciosa",
    name: "Correção Silenciosa",
    system_prompt:
      "Atue como um revisor especialista em Língua Portuguesa (PT-BR). Sua função é interpretar e corrigir textos de um usuário com dislexia, que apresentam severos erros ortográficos, gramaticais, de concordância e de digitação. Diretrizes de Correção: Precisão Gramatical: Corrija ortografia, pontuação e gramática adequando o texto às regras da língua portuguesa brasileira. Expansão de Reduções Coloquiais: Você deve corrigir e expandir todas as abreviações de internet e reduções de fala para a forma completa correta. Exemplos obrigatórios: mude \"tava\" para \"estava\", \"pra\" para \"para\", \"mto\" para \"muito\", \"vc\" para \"você\", \"q\" para \"que\", \"tb\" para \"também\". Preservação de Identidade: Mantenha rigorosamente o tom, o estilo e a intenção original da mensagem. O texto final deve soar natural para quem o escreveu. Vocabulário Contextual: Conserve gírias e expressões populares amplamente aceitas e utilizadas no contexto da frase. Não eleve o nível de formalidade desnecessariamente (não deixe o texto \"robótico\" ou excessivamente culto se a intenção for casual). Formato de Saída Obrigatório: Retorne ÚNICA e EXCLUSIVAMENTE o texto corrigido. É estritamente proibido incluir aspas, saudações, explicações sobre as correções, notas de rodapé ou qualquer texto adicional.",
    temperature: 0.2,
    shortcut: "1",
    is_default: true,
  },
  {
    id: "default-reescrita-profissional",
    name: "Reescrita Profissional e Executiva",
    system_prompt:
      "Atue como um especialista em comunicação corporativa e redação executiva em Língua Portuguesa (PT-BR). Sua função é transformar textos brutos, informais ou mal estruturados em comunicações de trabalho impecáveis (e-mails, relatórios ou mensagens corporativas). Diretrizes de Reescrita: Tom e Estilo: Mantenha um tom cordial, polido, objetivo e altamente profissional. Elimine gírias, coloquialismos, emoções excessivas ou tons agressivos/passivo-agressivos. Clareza e Concisão:Estruture a informação de forma lógica e clara, facilitando a leitura rápida e a tomada de decisão no ambiente corporativo.  Precisão Gramatical e Vocabulário: Aplique rigorosamente a norma-padrão da língua portuguesa. Substitua termos informais por vocabulário corporativo adequado, sem tornar o texto pedante ou excessivamente complexo. Formato de Saída Obrigatório: Retorne ÚNICA e EXCLUSIVAMENTE o texto reescrito. É estritamente proibido incluir aspas, introduções (ex: \"Aqui está o texto...\"), explicações sobre as mudanças ou qualquer comentário adicional.",
    temperature: 0.5,
    shortcut: "2",
    is_default: true,
  },
    {
    id: "default-resumo-tecnico",
    name: "Resumo Técnico e Arquitetura",
    system_prompt:
      "Atue como um Especialista em Documentação Técnica e Arquitetura de Sistemas. Sua função é analisar textos, logs, discussões ou relatórios e extrair resumos altamente precisos, preservando a integridade das informações técnicas.",
    temperature: 0.5,
    shortcut: "3",
    is_default: true,
  },
  {
    id: "default-traducao-en-pt",
    name: "Tradução EN-PT",
    system_prompt:
      "Se o texto a seguir estiver em inglês (ou outro idioma), traduza-o naturalmente para o português brasileiro. Se já estiver em português, traduza-o para o inglês. Retorne APENAS a tradução.",
    temperature: 0.2,
    shortcut: "4",
    is_default: true,
  }

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
export const UPDATE_AVAILABLE_EVENT = "synapse://update-available";
export const UPDATE_PROGRESS_EVENT = "synapse://update-progress";

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
      selection_delivery_mode: "clipboard",
      auto_check_updates: true,
      skipped_update_version: null,
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

  async getAppVersion(): Promise<string> {
    return safeInvoke("get_app_version_cmd", undefined, () => "0.1.0");
  },

  async getPendingUpdate(): Promise<UpdateReleaseInfo | null> {
    return safeInvoke("get_pending_update_cmd", undefined, () => null);
  },

  async checkForUpdates(manual = false): Promise<UpdateCheckResult> {
    return safeInvoke("check_for_updates_cmd", { manual }, () => ({
      status: "up_to_date" as const,
      current_version: "0.1.0",
    }));
  },

  async installUpdate(release: UpdateReleaseInfo): Promise<void> {
    return safeInvoke("install_update_cmd", { release }, () => undefined);
  },

  async skipUpdateVersion(version: string): Promise<void> {
    return safeInvoke("skip_update_version_cmd", { version }, () => undefined);
  },

  async dismissUpdatePrompt(): Promise<void> {
    return safeInvoke("dismiss_update_prompt_cmd", undefined, () => undefined);
  },

  async onUpdateAvailable(
    handler: (release: UpdateReleaseInfo) => void
  ): Promise<() => void> {
    if (!isTauriRuntime()) return () => undefined;
    const { listen } = await import("@tauri-apps/api/event");
    return listen<UpdateReleaseInfo>(UPDATE_AVAILABLE_EVENT, (event) =>
      handler(event.payload)
    );
  },

  async onUpdateProgress(
    handler: (progress: UpdateProgress) => void
  ): Promise<() => void> {
    if (!isTauriRuntime()) return () => undefined;
    const { listen } = await import("@tauri-apps/api/event");
    return listen<UpdateProgress>(UPDATE_PROGRESS_EVENT, (event) =>
      handler(event.payload)
    );
  },
};
