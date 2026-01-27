import type { ApiResponse } from "../bindings";

export type ApiKeyStatus = "idle" | "connecting" | "connected" | "failed";

const STATUS_LABEL: Record<ApiKeyStatus, string> = {
  idle: "未设置",
  connecting: "连接中",
  connected: "已连接",
  failed: "连接失败",
};

export const getApiKeyStatusLabel = (status: ApiKeyStatus): string =>
  STATUS_LABEL[status] ?? "未知";

export const resolveApiKeySaveOutcome = (
  result: ApiResponse<null> | null,
  error?: unknown,
): { status: ApiKeyStatus; apiKeySet: boolean; clearInput: boolean; message: string } => {
  if (error) {
    const message =
      error instanceof Error
        ? error.message
        : typeof error === "string"
          ? error
          : "连接失败";
    return { status: "failed", apiKeySet: false, clearInput: false, message };
  }
  if (result?.success) {
    return {
      status: "connected",
      apiKeySet: true,
      clearInput: false,
      message: "API 密钥已保存并连接成功",
    };
  }
  return {
    status: "failed",
    apiKeySet: false,
    clearInput: false,
    message: result?.message || "连接失败",
  };
};
