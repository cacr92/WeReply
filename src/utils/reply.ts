export const normalizeReplyText = (
  input: string,
): { ok: true; text: string } | { ok: false; text: ""; reason: string } => {
  const trimmed = input.trim();
  if (!trimmed) {
    return { ok: false, text: "", reason: "回复内容不能为空" };
  }
  if (trimmed.length > 2000) {
    return { ok: false, text: "", reason: "回复内容过长" };
  }
  return { ok: true, text: trimmed };
};
