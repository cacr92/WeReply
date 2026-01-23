import { describe, expect, it } from "vitest";
import { summarizeDiagnostics } from "./diagnostics";

describe("summarize diagnostics", () => {
  it("returns ok summary when both endpoints are ok", () => {
    const result = summarizeDiagnostics({
      base_url: "https://api.deepseek.com",
      model: "deepseek-chat",
      chat: { ok: true, status: 200, message: "ok" },
      models: { ok: true, status: 200, message: "ok" },
    });

    expect(result.ok).toBe(true);
    expect(result.message).toBe("连接诊断通过");
    expect(result.lines[0]).toContain("聊天接口");
  });

  it("returns failure summary with status and message", () => {
    const result = summarizeDiagnostics({
      base_url: "https://api.deepseek.com",
      model: "deepseek-chat",
      chat: { ok: false, status: 401, message: "Authentication Fails" },
      models: { ok: true, status: 200, message: "ok" },
    });

    expect(result.ok).toBe(false);
    expect(result.message).toContain("聊天接口");
    expect(result.lines[0]).toContain("401");
    expect(result.lines[0]).toContain("Authentication Fails");
  });
});
