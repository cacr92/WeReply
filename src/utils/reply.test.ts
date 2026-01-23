import { describe, expect, it } from "vitest";
import { normalizeReplyText } from "./reply";

describe("reply normalization", () => {
  it("rejects empty", () => {
    expect(normalizeReplyText("  ")).toEqual({
      ok: false,
      reason: "回复内容不能为空",
      text: "",
    });
  });

  it("rejects overlength", () => {
    const longText = "a".repeat(2001);
    expect(normalizeReplyText(longText)).toEqual({
      ok: false,
      reason: "回复内容过长",
      text: "",
    });
  });

  it("accepts trimmed", () => {
    expect(normalizeReplyText(" hi ")).toEqual({ ok: true, text: "hi" });
  });
});
