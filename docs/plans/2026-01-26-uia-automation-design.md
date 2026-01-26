# WeChat UI Automation Rewrite Design

**Goal:** Replace the legacy Python/wxauto agent with native UI Automation implementations to support WeChat 4.1.7 (Windows) and latest WeChat for macOS.

**Architecture:** Embed platform-specific automation inside the Tauri backend. Provide a unified Rust trait to list recent chats, start/stop listening, and write input. Windows uses UIA; macOS uses Accessibility API (AX). Internal events replace stdin/stdout IPC and feed existing status/suggestions flows.

**Tech Stack:** Rust (Tauri backend), Windows UIA (Windows API), macOS Accessibility API (Swift + bridge), tokio, tracing.

---

## Scope
- Recent chat list (scroll to load all)
- Start/stop listening with UIA/AX event subscriptions
- Write suggestion into chat input (UIA first, clipboard fallback)
- Error recovery and privacy-safe logging

## High-Level Components
- `src-tauri/src/ui_automation/mod.rs`: `WeChatAutomation` trait and factory
- `src-tauri/src/ui_automation/windows/*`: UIA window discovery, session list scanning, message watcher, input writer
- `src-tauri/src/ui_automation/macos/*`: AX discovery, session list scanning, message watcher, input writer
- `src-tauri/src/automation.rs` (or refactor of `agent.rs`): orchestration glue
- `src-tauri/src/lib.rs`: command handlers call automation module

## Data Flow
1. App boot: create automation instance and cache key UI elements.
2. Recent chats: front-end calls `list_recent_chats` -> automation scans sidebar, scrolls, dedupes.
3. Start listening: subscribe to UIA/AX events -> parse incoming messages -> emit `suggestions.updated`.
4. Write suggestion: UIA sets input value -> fallback to clipboard paste -> restore clipboard.

## Error Handling
- Window not found: emit `WECHAT_NOT_FOUND`, retry with backoff.
- Control tree changed: rebuild UI cache and retry.
- Event subscription failure: fallback to polling (500ms).
- Input write failures: fallback to clipboard then report recoverable error.
- Logs avoid content; only metadata and hashes.

## Testing & Acceptance
- Unit tests for control tree parsing, selectors, and dedupe.
- Integration smoke tests for recent chat listing and input write (mock UI tree).
- Acceptance: recent chat list loads in WeChat 4.1.7; start/stop listening works; suggestions can be written.
