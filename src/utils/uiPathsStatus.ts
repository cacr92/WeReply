import type { UiPathsStatus } from "../bindings";

const pad2 = (value: number): string => String(value).padStart(2, "0");

export const formatEpochSeconds = (
  epochSeconds: number,
  timeZone?: string,
): string => {
  const date = new Date(epochSeconds * 1000);
  if (Number.isNaN(date.getTime())) {
    return "";
  }
  const formatter = new Intl.DateTimeFormat("zh-CN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
    timeZone,
  });
  const parts = formatter.formatToParts(date);
  const lookup = (type: string): string =>
    parts.find((item) => item.type === type)?.value ?? "";
  const year = lookup("year");
  const month = lookup("month");
  const day = lookup("day");
  const hour = lookup("hour");
  const minute = lookup("minute");
  if (!year || !month || !day || !hour || !minute) {
    return "";
  }
  return `${year}-${month}-${day} ${pad2(Number(hour))}:${pad2(Number(minute))}`;
};

export const formatUiPathsStatus = (
  status: UiPathsStatus | null,
  timeZone?: string,
): string => {
  if (!status || !status.saved) {
    return "未保存";
  }
  if (status.saved_at === null || status.saved_at === undefined) {
    return "已保存";
  }
  const formatted = formatEpochSeconds(status.saved_at, timeZone);
  return formatted ? `已保存 ${formatted}` : "已保存";
};
