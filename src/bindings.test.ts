import { describe, expect, it, vi } from "vitest";

const invokeMock = vi.hoisted(() =>
  vi.fn().mockResolvedValue({ success: true, message: "", data: null }),
);

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import { commands } from "./bindings";

describe("tauri command bindings", () => {
  it("passes apiKey when saving api key", async () => {
    await commands.saveApiKey("sk-test");

    expect(invokeMock).toHaveBeenCalledWith("save_api_key", {
      apiKey: "sk-test",
    });
  });

  it("passes apiKey when diagnosing with provided key", async () => {
    await commands.diagnoseDeepseek("sk-diagnose");

    expect(invokeMock).toHaveBeenCalledWith("diagnose_deepseek", {
      apiKey: "sk-diagnose",
    });
  });

  it("omits args when diagnosing without a key", async () => {
    await commands.diagnoseDeepseek();

    expect(invokeMock).toHaveBeenCalledWith("diagnose_deepseek", {});
  });
});
