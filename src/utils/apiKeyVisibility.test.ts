import { describe, expect, it } from "vitest";
import { getApiKeyInputType, getApiKeyToggleLabel } from "./apiKeyVisibility";

describe("api key visibility helpers", () => {
  it("returns password type when hidden", () => {
    expect(getApiKeyInputType(false)).toBe("password");
  });

  it("returns text type when visible", () => {
    expect(getApiKeyInputType(true)).toBe("text");
  });

  it("returns toggle label based on visibility", () => {
    expect(getApiKeyToggleLabel(false)).toBe("显示");
    expect(getApiKeyToggleLabel(true)).toBe("隐藏");
  });
});
