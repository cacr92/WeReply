import { describe, expect, it } from "vitest";
import { getApiKeyStatusLabel, resolveApiKeySaveOutcome } from "./apiKey";

describe("api key labels", () => {
  it("returns connected label", () => {
    expect(getApiKeyStatusLabel("connected")).toBe("已连接");
  });

  it("returns failed label", () => {
    expect(getApiKeyStatusLabel("failed")).toBe("连接失败");
  });
});

describe("api key save outcome", () => {
  it("returns failed when invoke throws", () => {
    const result = resolveApiKeySaveOutcome(null, new Error("invoke failed"));
    expect(result.status).toBe("failed");
    expect(result.message).toBe("invoke failed");
    expect(result.apiKeySet).toBe(false);
    expect(result.clearInput).toBe(false);
  });
});
