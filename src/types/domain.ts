export interface Profile {
  id: string;
  name: string;
  system_prompt: string;
  temperature: number;
  shortcut: string | null;
  is_default: boolean;
}

export interface HistoryEntry {
  id: number;
  profile_name: string;
  original_text: string;
  generated_text: string;
  created_at: string;
}

export type SelectionDeliveryMode =
  | "clipboard"
  | "paste_and_keep"
  | "paste_and_restore";

export interface AppConfig {
  global_shortcut: string;
  compose_shortcut: string;
  undo_shortcut: string;
  launch_on_startup: boolean;
  base_url: string;
  model: string;
  timeout_secs: number;
  retention_days: number;
  selection_delivery_mode: SelectionDeliveryMode;
}

export interface AppConfigView extends AppConfig {
  has_api_key: boolean;
}

export type PopupMode = "selection" | "compose";

export interface PopupSessionView {
  selected_text: string;
  char_count: number;
  mode: PopupMode;
}

export interface SynapseErrorPayload {
  code: string;
  message?: string;
}
