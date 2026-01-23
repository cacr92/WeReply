import type { RuntimeState, SuggestionStyle } from "../bindings";

const STATE_LABEL: Record<RuntimeState, string> = {
  idle: "空闲",
  listening: "监听中",
  generating: "生成中",
  paused: "已暂停",
  error: "异常",
};

const STYLE_LABEL: Record<SuggestionStyle, string> = {
  formal: "正式",
  neutral: "中性",
  casual: "轻松",
};

export const getStateLabel = (state: RuntimeState): string =>
  STATE_LABEL[state] ?? "未知";

export const getStyleLabel = (style: SuggestionStyle): string =>
  STYLE_LABEL[style] ?? "未知";
