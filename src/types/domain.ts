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
  auto_check_updates?: boolean;
  skipped_update_version?: string | null;
}

export interface AppConfigView extends AppConfig {
  has_api_key: boolean;
}

export interface UpdateReleaseInfo {
  version: string;
  tag_name: string;
  release_notes: string;
  published_at: string;
  download_url: string;
  asset_name: string;
  asset_size_bytes: number;
}

export type UpdateCheckResult =
  | {
      status: "up_to_date";
      current_version: string;
    }
  | {
      status: "update_available";
      current_version: string;
      release: UpdateReleaseInfo;
    }
  | {
      status: "skipped";
      current_version: string;
      skipped_version: string;
    };

export interface UpdateProgress {
  downloaded_bytes: number;
  total_bytes: number;
  percentage: number;
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
