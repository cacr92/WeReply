export const getApiKeyInputType = (visible: boolean): "text" | "password" =>
  visible ? "text" : "password";

export const getApiKeyToggleLabel = (visible: boolean): string =>
  visible ? "隐藏" : "显示";
