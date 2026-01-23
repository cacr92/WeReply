export type DiagnosticStatus = {
  ok: boolean;
  status?: number | null;
  message: string;
};

export type DeepseekDiagnostics = {
  base_url: string;
  model: string;
  chat: DiagnosticStatus;
  models: DiagnosticStatus;
};

const formatLine = (label: string, status: DiagnosticStatus): string => {
  const statusCode = status.status ?? null;
  if (status.ok) {
    return statusCode ? `${label}: OK (HTTP ${statusCode})` : `${label}: OK`;
  }
  const statusText = statusCode ? ` (HTTP ${statusCode})` : "";
  const detail = status.message ? ` ${status.message}` : "";
  return `${label}: 失败${statusText}${detail}`;
};

export const summarizeDiagnostics = (
  diagnostics: DeepseekDiagnostics,
): { ok: boolean; message: string; lines: string[] } => {
  const lines = [
    formatLine("聊天接口", diagnostics.chat),
    formatLine("模型接口", diagnostics.models),
  ];
  const ok = diagnostics.chat.ok && diagnostics.models.ok;
  const message = ok ? "连接诊断通过" : lines.filter((_, idx) => ![diagnostics.chat.ok, diagnostics.models.ok][idx]).join("；");
  return { ok, message, lines };
};
