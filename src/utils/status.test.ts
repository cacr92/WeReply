import { describe, expect, it } from "vitest";
import type { Status } from "../bindings";
import { createStatusState, statusReducer } from "./status";

const idleStatus: Status = {
  state: "idle",
  platform: "unknown",
  agent_connected: false,
  last_error: "",
};

const listeningStatus: Status = {
  state: "listening",
  platform: "windows",
  agent_connected: true,
  last_error: "",
};

describe("status reducer", () => {
  it("applies bootstrap status before any live updates", () => {
    const initial = createStatusState(idleStatus);
    const next = statusReducer(initial, { type: "bootstrap", status: listeningStatus });

    expect(next.status.state).toBe("listening");
    expect(next.hasLiveUpdate).toBe(false);
  });

  it("ignores bootstrap updates after live status arrives", () => {
    const initial = createStatusState(idleStatus);
    const afterEvent = statusReducer(initial, { type: "event", status: listeningStatus });
    const afterBootstrap = statusReducer(afterEvent, { type: "bootstrap", status: idleStatus });

    expect(afterBootstrap.status.state).toBe("listening");
    expect(afterBootstrap.hasLiveUpdate).toBe(true);
  });
});
