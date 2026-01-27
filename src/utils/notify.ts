import { message } from "antd";

type NotifyOptions = {
  detail?: unknown;
  fallback?: string;
  key?: string;
  duration?: number;
};

const DEFAULT_DURATIONS = {
  success: 2,
  info: 2,
  warning: 3,
  error: 4,
  loading: 0,
} as const;

const DEFAULT_ERROR_FALLBACK = "请稍后重试";

export const resolveNotifyDetail = (input: unknown): string | null => {
  if (typeof input === "string") {
    const trimmed = input.trim();
    return trimmed.length > 0 ? trimmed : null;
  }
  if (input instanceof Error) {
    const trimmed = input.message.trim();
    return trimmed.length > 0 ? trimmed : null;
  }
  return null;
};

export const formatNotifyMessage = (
  action: string,
  detail?: unknown,
  fallback?: string,
): string => {
  const actionText = resolveNotifyDetail(action) ?? "";
  const detailText = resolveNotifyDetail(detail) ?? resolveNotifyDetail(fallback);
  if (!detailText || detailText === actionText) {
    return actionText || detailText || "";
  }
  if (!actionText) {
    return detailText;
  }
  return `${actionText}：${detailText}`;
};

const toDuration = (value: number | undefined, fallback: number): number =>
  typeof value === "number" ? value : fallback;

export const notify = {
  success(action: string, options: NotifyOptions = {}): void {
    message.success({
      content: formatNotifyMessage(action, options.detail),
      duration: toDuration(options.duration, DEFAULT_DURATIONS.success),
      key: options.key,
    });
  },
  info(action: string, options: NotifyOptions = {}): void {
    message.info({
      content: formatNotifyMessage(action, options.detail),
      duration: toDuration(options.duration, DEFAULT_DURATIONS.info),
      key: options.key,
    });
  },
  warning(action: string, options: NotifyOptions = {}): void {
    message.warning({
      content: formatNotifyMessage(action, options.detail),
      duration: toDuration(options.duration, DEFAULT_DURATIONS.warning),
      key: options.key,
    });
  },
  error(action: string, options: NotifyOptions = {}): void {
    const fallback =
      options.fallback === undefined ? DEFAULT_ERROR_FALLBACK : options.fallback;
    message.error({
      content: formatNotifyMessage(action, options.detail, fallback),
      duration: toDuration(options.duration, DEFAULT_DURATIONS.error),
      key: options.key,
    });
  },
  loading(action: string, options: NotifyOptions = {}): void {
    message.loading({
      content: formatNotifyMessage(action, options.detail),
      duration: toDuration(options.duration, DEFAULT_DURATIONS.loading),
      key: options.key,
    });
  },
};
