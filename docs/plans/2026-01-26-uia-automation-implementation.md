# WeChat UIA Automation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the legacy Python/wxauto agent with native UI Automation implementations for Windows (WeChat 4.1.7) and macOS (latest), supporting recent chats, listening, and input writing.

**Architecture:** Embed automation in the Tauri backend. Implement a platform-agnostic `WeChatAutomation` trait with Windows UIA and macOS Accessibility implementations. Orchestrator calls this trait directly instead of stdin/stdout IPC. Provide robust fallback and error reporting.

**Tech Stack:** Rust (Tauri), Windows UIA (windows crate), macOS Accessibility (objc/accessibility-sys), tokio, tracing, vitest.

---

### Task 1: Add automation trait and manager (@rust-optimization)

**Files:**
- Create: `src-tauri/src/ui_automation/mod.rs`
- Create: `src-tauri/src/ui_automation/types.rs`
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/ui_automation/tests.rs`

**Step 1: Write failing tests**
```rust
#[tokio::test]
async fn automation_manager_rejects_when_not_ready() {
    let mgr = AutomationManager::new(None);
    let res = mgr.list_recent_chats().await;
    assert!(!res.success);
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test ui_automation::tests::automation_manager_rejects_when_not_ready -v`
Expected: FAIL (missing module/struct)

**Step 3: Write minimal implementation**
```rust
pub trait WeChatAutomation {
    fn platform(&self) -> Platform;
    fn list_recent_chats(&self) -> Result<Vec<ChatSummary>>;
    fn start_listening(&self, targets: Vec<ListenTarget>) -> Result<()>;
    fn stop_listening(&self) -> Result<()>;
    fn write_input(&self, chat_id: &str, text: &str) -> Result<()>;
}

pub struct AutomationManager {
    inner: Option<Arc<dyn WeChatAutomation + Send + Sync>>,
}
```
Wire into `state` and `list_recent_chats` command with a stub for now.

**Step 4: Run test to verify it passes**
Run: `cargo test ui_automation::tests::automation_manager_rejects_when_not_ready -v`
Expected: PASS

**Step 5: Commit**
```bash
git add src-tauri/src/ui_automation/mod.rs src-tauri/src/ui_automation/types.rs src-tauri/src/state.rs src-tauri/src/lib.rs src-tauri/src/ui_automation/tests.rs
git commit -m "feat: add automation trait and manager"
```

---

### Task 2: Windows UIA foundation (window discovery + element adapter) (@wechat-automation)

**Files:**
- Create: `src-tauri/src/ui_automation/windows/mod.rs`
- Create: `src-tauri/src/ui_automation/windows/uia.rs`
- Create: `src-tauri/src/ui_automation/windows/element.rs`
- Test: `src-tauri/src/ui_automation/windows/tests.rs`
- Modify: `src-tauri/Cargo.toml`

**Step 1: Write failing tests**
```rust
#[test]
fn uia_finds_wechat_main_window_by_process_name() {
    let mock = MockUia::with_window("Weixin.exe", "微信");
    let hwnd = find_wechat_hwnd(&mock).unwrap();
    assert_eq!(hwnd, 1001);
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test uia_finds_wechat_main_window_by_process_name -v`
Expected: FAIL (missing functions)

**Step 3: Write minimal implementation**
- Add windows crate features for UIA and Win32 window enumeration.
- Implement `UiaProvider` trait with a mockable adapter.
- Implement `find_wechat_hwnd` that matches `Weixin.exe` / `WeChatAppEx.exe`.

**Step 4: Run test to verify it passes**
Run: `cargo test uia_finds_wechat_main_window_by_process_name -v`
Expected: PASS

**Step 5: Commit**
```bash
git add src-tauri/src/ui_automation/windows/mod.rs src-tauri/src/ui_automation/windows/uia.rs src-tauri/src/ui_automation/windows/element.rs src-tauri/src/ui_automation/windows/tests.rs src-tauri/Cargo.toml
git commit -m "feat: add Windows UIA discovery scaffold"
```

---

### Task 3: Windows recent chat list scanning (scroll + dedupe) (@wechat-automation)

**Files:**
- Modify: `src-tauri/src/ui_automation/windows/mod.rs`
- Create: `src-tauri/src/ui_automation/windows/session_list.rs`
- Test: `src-tauri/src/ui_automation/windows/tests.rs`

**Step 1: Write failing tests**
```rust
#[test]
fn session_list_scrolls_and_dedupes() {
    let mock = MockTree::with_sessions(vec!["A", "B", "C", "B"]);
    let chats = collect_recent_chats(&mock).unwrap();
    assert_eq!(chats.len(), 3);
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test session_list_scrolls_and_dedupes -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- Locate session list container by name/type heuristics.
- Scroll (PageDown) until no new items.
- Collect titles and infer `ChatKind` by badge/label heuristics.

**Step 4: Run test to verify it passes**
Run: `cargo test session_list_scrolls_and_dedupes -v`
Expected: PASS

**Step 5: Commit**
```bash
git add src-tauri/src/ui_automation/windows/session_list.rs src-tauri/src/ui_automation/windows/mod.rs src-tauri/src/ui_automation/windows/tests.rs
git commit -m "feat: scan Windows recent chat list"
```

---

### Task 4: Windows message watcher (UIA events + polling fallback) (@wechat-automation)

**Files:**
- Create: `src-tauri/src/ui_automation/windows/message_watch.rs`
- Modify: `src-tauri/src/ui_automation/windows/mod.rs`
- Test: `src-tauri/src/ui_automation/windows/tests.rs`

**Step 1: Write failing tests**
```rust
#[test]
fn watcher_falls_back_to_polling_on_subscribe_failure() {
    let mock = MockWatcher::subscribe_fail();
    let mode = mock.start();
    assert_eq!(mode, WatchMode::Polling);
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test watcher_falls_back_to_polling_on_subscribe_failure -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- Try UIA event subscription; on failure start polling loop.
- Parse latest message rows into `MessageNewPayload`.

**Step 4: Run test to verify it passes**
Run: `cargo test watcher_falls_back_to_polling_on_subscribe_failure -v`
Expected: PASS

**Step 5: Commit**
```bash
git add src-tauri/src/ui_automation/windows/message_watch.rs src-tauri/src/ui_automation/windows/mod.rs src-tauri/src/ui_automation/windows/tests.rs
git commit -m "feat: Windows message watcher with fallback"
```

---

### Task 5: Windows input writer (UIA + clipboard fallback) (@wechat-automation)

**Files:**
- Create: `src-tauri/src/ui_automation/windows/input_box.rs`
- Modify: `src-tauri/src/ui_automation/windows/mod.rs`
- Test: `src-tauri/src/ui_automation/windows/tests.rs`

**Step 1: Write failing tests**
```rust
#[test]
fn input_writer_uses_clipboard_on_uia_failure() {
    let mock = MockInputWriter::uia_fail();
    let ok = mock.write("chat", "hello");
    assert!(ok);
    assert!(mock.used_clipboard());
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test input_writer_uses_clipboard_on_uia_failure -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- Try UIA set value + send Enter.
- On failure, set clipboard, paste, restore clipboard.

**Step 4: Run test to verify it passes**
Run: `cargo test input_writer_uses_clipboard_on_uia_failure -v`
Expected: PASS

**Step 5: Commit**
```bash
git add src-tauri/src/ui_automation/windows/input_box.rs src-tauri/src/ui_automation/windows/mod.rs src-tauri/src/ui_automation/windows/tests.rs
git commit -m "feat: Windows input writer with clipboard fallback"
```

---

### Task 6: macOS Accessibility foundation (@macos-agent-development)

**Files:**
- Create: `src-tauri/src/ui_automation/macos/mod.rs`
- Create: `src-tauri/src/ui_automation/macos/ax.rs`
- Create: `src-tauri/src/ui_automation/macos/element.rs`
- Test: `src-tauri/src/ui_automation/macos/tests.rs`
- Modify: `src-tauri/Cargo.toml`

**Step 1: Write failing tests**
```rust
#[test]
fn ax_finds_wechat_app() {
    let mock = MockAx::with_bundle("com.tencent.xinWeChat");
    assert!(find_wechat_app(&mock).is_some());
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test ax_finds_wechat_app -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- Add macOS AX dependencies via objc/accessibility-sys.
- Create adapter to query app and window tree.

**Step 4: Run test to verify it passes**
Run: `cargo test ax_finds_wechat_app -v`
Expected: PASS

**Step 5: Commit**
```bash
git add src-tauri/src/ui_automation/macos/mod.rs src-tauri/src/ui_automation/macos/ax.rs src-tauri/src/ui_automation/macos/element.rs src-tauri/src/ui_automation/macos/tests.rs src-tauri/Cargo.toml
git commit -m "feat: macOS AX discovery scaffold"
```

---

### Task 7: macOS recent chats + message watcher + input writer (@macos-agent-development)

**Files:**
- Create: `src-tauri/src/ui_automation/macos/session_list.rs`
- Create: `src-tauri/src/ui_automation/macos/message_watch.rs`
- Create: `src-tauri/src/ui_automation/macos/input_box.rs`
- Modify: `src-tauri/src/ui_automation/macos/mod.rs`
- Test: `src-tauri/src/ui_automation/macos/tests.rs`

**Step 1: Write failing tests**
```rust
#[test]
fn macos_session_list_dedupes() {
    let mock = MockAxTree::with_sessions(vec!["A", "A", "B"]);
    let chats = collect_recent_chats(&mock).unwrap();
    assert_eq!(chats.len(), 2);
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test macos_session_list_dedupes -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- AX tree scan for sidebar list, scroll and collect names.
- AX observer for message list change; fallback to polling.
- Input write via AX set value + optional clipboard fallback.

**Step 4: Run test to verify it passes**
Run: `cargo test macos_session_list_dedupes -v`
Expected: PASS

**Step 5: Commit**
```bash
git add src-tauri/src/ui_automation/macos/session_list.rs src-tauri/src/ui_automation/macos/message_watch.rs src-tauri/src/ui_automation/macos/input_box.rs src-tauri/src/ui_automation/macos/mod.rs src-tauri/src/ui_automation/macos/tests.rs
git commit -m "feat: macOS recent chats and message watcher"
```

---

### Task 8: Orchestrator integration (replace agent flow) (@tauri-development)

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/state.rs`
- Modify: `src-tauri/src/agent.rs` (deprecate or repurpose)
- Modify: `src-tauri/src/ipc.rs` (remove unused)
- Test: `src-tauri/src/lib.rs`

**Step 1: Write failing tests**
```rust
#[tokio::test]
async fn list_recent_chats_uses_automation() {
    let state = test_state_with_mock_automation();
    let res = list_recent_chats_inner(state).await.unwrap();
    assert!(res.success);
}
```

**Step 2: Run test to verify it fails**
Run: `cargo test list_recent_chats_uses_automation -v`
Expected: FAIL

**Step 3: Write minimal implementation**
- Replace IPC request with direct automation call.
- Wire message watcher output into suggestions flow.
- Remove pending list logic no longer used.

**Step 4: Run test to verify it passes**
Run: `cargo test list_recent_chats_uses_automation -v`
Expected: PASS

**Step 5: Commit**
```bash
git add src-tauri/src/lib.rs src-tauri/src/state.rs src-tauri/src/agent.rs src-tauri/src/ipc.rs
git commit -m "refactor: integrate UIA automation"
```

---

### Task 9: Frontend + bindings updates (@react-typescript-development)

**Files:**
- Modify: `src/bindings.ts`
- Modify: `src/App.tsx`
- Test: `src/bindings.test.ts`

**Step 1: Write failing tests**
```ts
it('listRecentChats returns ChatSummary', async () => {
  expect(commands.listRecentChats).toBeDefined();
});
```

**Step 2: Run test to verify it fails**
Run: `npm test -- bindings.test.ts -t listRecentChats`
Expected: FAIL if bindings changed

**Step 3: Write minimal implementation**
- Regenerate bindings if needed.
- Keep UI behavior unchanged.

**Step 4: Run test to verify it passes**
Run: `npm test -- bindings.test.ts -t listRecentChats`
Expected: PASS

**Step 5: Commit**
```bash
git add src/bindings.ts src/App.tsx src/bindings.test.ts
git commit -m "chore: refresh bindings for automation"
```

---

### Task 10: Verification (@verification-before-completion)

**Step 1: Run backend tests**
Run: `cargo test`
Expected: PASS

**Step 2: Run frontend tests**
Run: `npm test`
Expected: PASS

**Step 3: Manual smoke checks**
- WeChat 4.1.7 running, recent chat list loads and scrolls.
- Start/stop listening works.
- Write suggestion inserts text (UIA or clipboard fallback).

**Step 4: Commit (if needed)**
```bash
git status -sb
```

---
