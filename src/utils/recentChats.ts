import type { ListenTargetKind } from "./listenTargets";

export type RecentChat = {
  chat_id: string;
  chat_title: string;
  kind: ListenTargetKind;
};

export const filterRecentChats = (
  chats: RecentChat[],
  query: string,
): RecentChat[] => {
  const normalized = query.trim().toLowerCase();
  if (!normalized) {
    return chats;
  }
  return chats.filter((chat) => {
    const title = chat.chat_title?.trim() ?? "";
    const id = chat.chat_id?.trim() ?? "";
    return (
      title.toLowerCase().includes(normalized) ||
      id.toLowerCase().includes(normalized)
    );
  });
};
