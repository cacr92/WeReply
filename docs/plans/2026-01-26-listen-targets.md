# Listen Targets Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add per-chat listen targets (manual + recent list) so agents only monitor selected chats and group messages are included.

**Architecture:** Orchestrator owns persistent listen_targets in Config. It pushes targets to agents on start/resume and on updates. Agents create independent chat windows per target, emit message.new with accurate is_group, and expose recent chat list via IPC. Frontend provides manual add + recent list picker with persistence.

**Tech Stack:** Rust (Tauri + specta), Python (wxauto), Swift (Accessibility), React + TS + Ant Design.

---

### Task 1: Types and validation (@tdd-workflow, @ipc-communication)

**Files:**
- Modify: src-tauri/src/types.rs
- Modify: src-tauri/src/ipc.rs
- Create: src-tauri/src/listen_targets.rs

**Step 1: Write the failing tests**

Add in `src-tauri/src/listen_targets.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_dedupes_targets() {
        let input = vec![
            ListenTarget { name: "  Team A ".into(), kind: ChatKind::Unknown },
            ListenTarget { name: "Team A".into(), kind: ChatKind::Unknown },
            ListenTarget { name: "".into(), kind: ChatKind::Unknown },
        ];
        let out = normalize_listen_targets(input, 50).unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "Team A");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p wereply-lib listen_targets` (or `cargo test normalize_listen_targets`)
Expected: FAIL (missing function / module).

**Step 3: Write minimal implementation**

```rust
pub fn normalize_listen_targets(mut targets: Vec<ListenTarget>, max: usize) -> Result<Vec<ListenTarget>> {
    // trim, drop empty, dedupe by name (case-sensitive), cap size
    Ok(result)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test normalize_listen_targets`
Expected: PASS.

**Step 5: Commit**

Run:
```bash
git add src-tauri/src/types.rs src-tauri/src/ipc.rs src-tauri/src/listen_targets.rs
git commit -m "feat: add listen target types and validation"
```

---

### Task 2: Orchestrator commands and IPC requests (@tauri-development, @ipc-communication)

**Files:**
- Modify: src-tauri/src/lib.rs
- Modify: src-tauri/src/state.rs
- Modify: src-tauri/src/agent.rs

**Step 1: Write the failing tests**

Add unit tests for IPC payload serialization in `src-tauri/src/ipc.rs`:

```rust
#[test]
fn listen_control_payload_with_targets_serializes() {
    let payload = ListenControlPayload {
        poll_interval_ms: Some(800),
        targets: Some(vec![ListenTarget { name: "Team A".into(), kind: ChatKind::Group }]),
    };
    let value = serde_json::to_value(payload).unwrap();
    assert!(value.get("targets").is_some());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test listen_control_payload_with_targets_serializes`
Expected: FAIL (missing fields).

**Step 3: Write minimal implementation**

- Add new commands: `get_listen_targets`, `set_listen_targets`, `list_recent_chats`.
- Update `send_listen_control` to include targets on start/resume.
- On `set_listen_targets`, validate + save config and push `listen.targets` if listening.
- In `agent.rs`, handle `chats.list.result`: update state cache and fulfill pending oneshot.

**Step 4: Run test to verify it passes**

Run: `cargo test listen_control_payload_with_targets_serializes`
Expected: PASS.

**Step 5: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/state.rs src-tauri/src/agent.rs src-tauri/src/ipc.rs
git commit -m "feat: add listen target commands and IPC flow"
```

---

### Task 3: Windows agent independent listening (@python-agent-development, @wechat-automation)

**Files:**
- Modify: platform_agents/windows/wxauto_agent.py

**Step 1: Write the failing test (lightweight)**

Add a small unit test in `platform_agents/windows/tests/test_listen_targets.py` to validate target normalization in agent helper (if adding helper). If no test harness, add a minimal inline self-test block guarded by `if __name__ == "__main__":` and keep it disabled by default.

**Step 2: Run test to verify it fails**

Run: `python -m unittest platform_agents/windows/tests/test_listen_targets.py`
Expected: FAIL (missing helper).

**Step 3: Write minimal implementation**

- Track `listen_targets` and `listen_chats` in state.
- On `listen.start/resume`, update targets from payload and call `AddListenChat` per target.
- On `listen.targets`, update targets (add/remove) without toggling listening state.
- Use callback to push incoming messages into a thread-safe queue.
- Dequeue in main loop, dedupe per chat, emit `message.new` with `is_group` from `chat.ChatInfo()`.
- Implement `chats.list` -> `chats.list.result` using `wx.GetSession()`.

**Step 4: Run test to verify it passes**

Run: `python -m unittest platform_agents/windows/tests/test_listen_targets.py`
Expected: PASS.

**Step 5: Commit**

```bash
git add platform_agents/windows/wxauto_agent.py platform_agents/windows/tests/test_listen_targets.py
git commit -m "feat: windows agent listen targets and chat list"
```

---

### Task 4: macOS agent multi-window monitoring (@macos-agent-development, @wechat-automation)

**Files:**
- Modify: platform_agents/macos/wechat_agent.swift

**Step 1: Write the failing test**

Add a small unit test for target matching / normalization (if extracting helpers). If no test target, add a simple deterministic helper and test via a minimal Swift test file if present; otherwise note the limitation and proceed with guarded runtime checks.

**Step 2: Run test to verify it fails**

Run: `swift test` (if Package.swift exists)
Expected: FAIL (missing helper). If no tests are configured, record that and proceed.

**Step 3: Write minimal implementation**

- Add `listenTargets: [ListenTarget]` to agent state.
- Replace `frontmostWeChatWindow` usage with `allWeChatWindows()`.
- For each window whose title matches a target, collect latest message and emit `message.new` with `is_group` from target or title heuristic.
- Implement `chats.list` by scanning AXTable/AXOutline/AXList rows for session titles.
- Handle `listen.targets` command to update targets without activating WeChat.

**Step 4: Run test to verify it passes**

Run: `swift test` (if available)
Expected: PASS or document missing test harness.

**Step 5: Commit**

```bash
git add platform_agents/macos/wechat_agent.swift
git commit -m "feat: macos agent listen targets and chat list"
```

---

### Task 5: Frontend targets UI (@react-typescript-development)

**Files:**
- Modify: src/App.tsx
- Modify: src/App.css
- Create: src/utils/listenTargets.ts
- Create: src/utils/listenTargets.test.ts

**Step 1: Write the failing tests**

```ts
describe("normalizeListenTargets", () => {
  it("trims and dedupes", () => {
    expect(normalizeListenTargets(["  A ", "A", ""]).map(t => t.name)).toEqual(["A"]);
  });
});
```

**Step 2: Run test to verify it fails**

Run: `npm test -- listenTargets`
Expected: FAIL (missing module).

**Step 3: Write minimal implementation**

- Add UI section in Settings: manual input + add button, recent list + add button, selected targets list with remove.
- Call `commands.getListenTargets` on boot, `commands.setListenTargets` on save.
- Call `commands.listRecentChats` to refresh list.
- Block `startListening` if no targets selected.

**Step 4: Run test to verify it passes**

Run: `npm test -- listenTargets`
Expected: PASS.

**Step 5: Commit**

```bash
git add src/App.tsx src/App.css src/utils/listenTargets.ts src/utils/listenTargets.test.ts
git commit -m "feat: add listen targets UI"
```

---

### Task 6: Bindings and verification (@tauri-development)

**Files:**
- Modify: src-tauri/src/bindings.rs
- Modify: src/bindings.ts

**Step 1: Generate bindings**

Run: `cargo run --bin generate_bindings` (from `src-tauri/`)
Expected: `src/bindings.ts` updated with new types + commands.

**Step 2: Verify**

Run: `cargo test` and `npm test` (note any failures or if not run).

**Step 3: Commit**

```bash
git add src-tauri/src/bindings.rs src/bindings.ts
git commit -m "chore: update bindings for listen targets"
```
