import { describe, expect, it } from "vitest";
import { getStateLabel, getStyleLabel } from "./labels";

describe("labels", () => {
  it("maps runtime states to labels", () => {
    expect(getStateLabel("idle")).toBe("空闲");
    expect(getStateLabel("listening")).toBe("监听中");
    expect(getStateLabel("generating")).toBe("生成中");
    expect(getStateLabel("paused")).toBe("已暂停");
    expect(getStateLabel("error")).toBe("异常");
  });

  it("maps suggestion styles to labels", () => {
    expect(getStyleLabel("formal")).toBe("正式");
    expect(getStyleLabel("neutral")).toBe("中性");
    expect(getStyleLabel("casual")).toBe("轻松");
  });
});
