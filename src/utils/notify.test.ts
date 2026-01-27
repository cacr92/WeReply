import { describe, expect, it } from "vitest";
import { formatNotifyMessage, resolveNotifyDetail } from "./notify";

describe("notify formatting", () => {
  it("uses detail when provided", () => {
    expect(formatNotifyMessage("保存失败", "网络异常")).toBe("保存失败：网络异常");
  });

  it("falls back when detail is missing", () => {
    expect(formatNotifyMessage("保存失败", undefined, "请稍后重试")).toBe(
      "保存失败：请稍后重试",
    );
  });

  it("avoids duplicate detail", () => {
    expect(formatNotifyMessage("连接失败", "连接失败")).toBe("连接失败");
  });

  it("extracts error message", () => {
    expect(resolveNotifyDetail(new Error("出错了"))).toBe("出错了");
  });
});
