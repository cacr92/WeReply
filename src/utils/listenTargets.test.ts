import { describe, expect, it } from "vitest";
import { normalizeListenTargets } from "./listenTargets";

describe("listen targets", () => {
  it("trims and dedupes names", () => {
    const targets = normalizeListenTargets(["  A ", "A", ""]);
    expect(targets.map((item) => item.name)).toEqual(["A"]);
  });
});
