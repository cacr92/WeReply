import { invoke } from "@tauri-apps/api/core";

export type RuntimeState = "idle" | "listening" | "generating" | "paused" | "error";
export type Platform = "windows" | "macos" | "unknown";
export type SuggestionStyle = "formal" | "neutral" | "casual";

export interface Suggestion {
  id: string;
  style: SuggestionStyle;
  text: string;
}

export interface Status {
  state: RuntimeState;
  platform: Platform;
  agent_connected: boolean;
  last_error: string;
}

export interface Config {
  deepseek_model: string;
  suggestion_count: number;
  context_max_messages: number;
  context_max_chars: number;
  poll_interval_ms: number;
  temperature: number;
  top_p: number;
  base_url: string;
  timeout_ms: number;
  max_retries: number;
  log_level: string;
  log_to_file: boolean;
}

export interface SuggestionsUpdated {
  chat_id: string;
  suggestions: Suggestion[];
}

export interface ErrorPayload {
  code: string;
  message: string;
  recoverable: boolean;
}

export interface ApiResponse<T> {
  success: boolean;
  message: string;
  data: T | null;
}

export const commands = {
  getConfig: (): Promise<ApiResponse<Config>> => invoke("get_config"),
  setConfig: (config: Config): Promise<ApiResponse<null>> =>
    invoke("set_config", { config }),
  getStatus: (): Promise<ApiResponse<Status>> => invoke("get_status"),
  startListening: (): Promise<ApiResponse<null>> => invoke("start_listening"),
  stopListening: (): Promise<ApiResponse<null>> => invoke("stop_listening"),
  pauseListening: (): Promise<ApiResponse<null>> => invoke("pause_listening"),
  resumeListening: (): Promise<ApiResponse<null>> => invoke("resume_listening"),
  writeSuggestion: (chatId: string, text: string): Promise<ApiResponse<null>> =>
    invoke("write_suggestion", { chat_id: chatId, text }),
  saveApiKey: (apiKey: string): Promise<ApiResponse<null>> =>
    invoke("save_api_key", { api_key: apiKey }),
  getApiKeyStatus: (): Promise<ApiResponse<boolean>> =>
    invoke("get_api_key_status"),
  deleteApiKey: (): Promise<ApiResponse<null>> => invoke("delete_api_key"),
};
