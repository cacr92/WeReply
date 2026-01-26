import { describe, expect, it } from "vitest";
import { filterRecentChats, type RecentChat } from "./recentChats";

describe("recent chats", () => {
  it("returns all chats when query is empty", () => {
    const chats: RecentChat[] = [
      { chat_id: "a", chat_title: "Alpha", kind: "unknown" },
      { chat_id: "b", chat_title: "Beta", kind: "direct" },
    ];

    expect(filterRecentChats(chats, "")).toEqual(chats);
  });

  it("filters by title or id with trimming", () => {
    const chats: RecentChat[] = [
      { chat_id: "team-01", chat_title: "项目群", kind: "group" },
      { chat_id: "alice", chat_title: "Alice", kind: "direct" },
      { chat_id: "report", chat_title: "", kind: "unknown" },
    ];

    expect(filterRecentChats(chats, " 项目 ")).toEqual([chats[0]]);
    expect(filterRecentChats(chats, "ali")).toEqual([chats[1]]);
    expect(filterRecentChats(chats, "report")).toEqual([chats[2]]);
  });
});
