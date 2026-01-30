import { describe, expect, it } from "vitest";

import type { UiPathsStatus } from "../bindings";
import { formatEpochSeconds, formatUiPathsStatus } from "./uiPathsStatus";

describe("ui paths status formatting", () => {
  it("formats epoch seconds in UTC when timezone provided", () => {
    const formatted = formatEpochSeconds(0, "UTC");
    expect(formatted).toBe("1970-01-01 00:00");
  });

  it("returns 未保存 when status missing", () => {
    expect(formatUiPathsStatus(null)).toBe("未保存");
  });

  it("returns 已保存 with formatted timestamp", () => {
    const status: UiPathsStatus = {
      saved: true,
      saved_at: 0,
      version: 1,
      paths_file: null,
      tree_file: null,
    };
    expect(formatUiPathsStatus(status, "UTC")).toBe("已保存 1970-01-01 00:00");
  });
});
