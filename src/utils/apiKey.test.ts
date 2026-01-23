import { describe, expect, it } from "vitest";
import { getApiKeyStatusLabel } from "./apiKey";

describe("api key labels", () => {
  it("returns connected label", () => {
    expect(getApiKeyStatusLabel("connected")).toBe("已连接");
  });

  it("returns failed label", () => {
    expect(getApiKeyStatusLabel("failed")).toBe("连接失败");
  });
});
