export type ApiKeyStatus = "idle" | "connecting" | "connected" | "failed";

const STATUS_LABEL: Record<ApiKeyStatus, string> = {
  idle: "未设置",
  connecting: "连接中",
  connected: "已连接",
  failed: "连接失败",
};

export const getApiKeyStatusLabel = (status: ApiKeyStatus): string =>
  STATUS_LABEL[status] ?? "未知";
