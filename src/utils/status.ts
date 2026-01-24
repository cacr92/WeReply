import type { Status } from "../bindings";

export type StatusAction =
  | { type: "bootstrap"; status: Status }
  | { type: "event"; status: Status }
  | { type: "optimistic"; state: Status["state"]; last_error?: string };

export type StatusState = {
  status: Status;
  hasLiveUpdate: boolean;
};

export function createStatusState(status: Status): StatusState {
  return { status, hasLiveUpdate: false };
}

export function statusReducer(state: StatusState, action: StatusAction): StatusState {
  switch (action.type) {
    case "bootstrap":
      if (state.hasLiveUpdate) {
        return state;
      }
      return { ...state, status: action.status };
    case "event":
      return { status: action.status, hasLiveUpdate: true };
    case "optimistic":
      return {
        status: {
          ...state.status,
          state: action.state,
          last_error: action.last_error ?? state.status.last_error,
        },
        hasLiveUpdate: true,
      };
    default:
      return state;
  }
}
