import { describe, expect, it } from "vitest";
import { DEFAULT_MODELS, normalizeModels, resolveModelSelection } from "./models";

describe("models", () => {
  it("normalizeModels falls back to defaults when list is empty", () => {
    expect(normalizeModels([])).toEqual(DEFAULT_MODELS);
  });

  it("normalizeModels filters and preserves allowed order", () => {
    expect(normalizeModels(["other", "deepseek-reasoner", "deepseek-chat"])).toEqual([
      "deepseek-chat",
      "deepseek-reasoner",
    ]);
  });

  it("resolveModelSelection keeps selection when available", () => {
    const result = resolveModelSelection(["deepseek-chat"], "deepseek-chat");
    expect(result.selected).toBe("deepseek-chat");
    expect(result.changed).toBe(false);
  });

  it("resolveModelSelection falls back to first model", () => {
    const result = resolveModelSelection(["deepseek-chat"], "deepseek-reasoner");
    expect(result.selected).toBe("deepseek-chat");
    expect(result.changed).toBe(true);
  });
});
