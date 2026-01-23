# WeReply Complete Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 完成监听与输入写入功能（Windows/macOS），简化 UI，仅保留 API Key + 主题切换，并在保存 API Key 后立即验证 DeepSeek 连接。

**Architecture:** Tauri Rust 作为中枢，Agent 负责平台监听/写入。前端只展示状态、建议与 API Key 输入。Rust 负责 IPC 控制与 DeepSeek 调用，保存 API Key 后进行最小请求验证。

**Tech Stack:** Tauri 2.x, Rust + tokio + reqwest, React + TypeScript + Ant Design, wxauto (Windows), Swift Accessibility (macOS)

---

### Task 1: UI 简化与 API Key 状态

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/App.css`
- Modify: `src/bindings.ts`
- Test: `src/utils/labels.test.ts`

**Step 1: Write the failing test**

```typescript
import { describe, expect, it } from "vitest";
import { getStateLabel } from "./labels";

describe("labels", () => {
  it("returns fallback for unknown state", () => {
    expect(getStateLabel("error")).toBe("异常");
  });
});
```

**Step 2: Run test to verify it fails**

Run: `npm test -- src/utils/labels.test.ts`
Expected: FAIL if missing coverage for new UI states or labels.

**Step 3: Write minimal implementation**

- Remove config panel UI and related state.
- Keep theme toggle and API Key panel.
- Add API Key validation status display.

**Step 4: Run test to verify it passes**

Run: `npm test -- src/utils/labels.test.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/App.tsx src/App.css src/bindings.ts src/utils/labels.test.ts
git commit -m "feat: simplify ui and api key flow"
```

---

### Task 2: DeepSeek API Key 验证逻辑

**Files:**
- Modify: `src-tauri/src/deepseek.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/types.rs`
- Test: `src-tauri/src/deepseek.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn build_request_for_validation() {
    let req = build_request("ping", 1, "deepseek-chat", 0.0, 1.0);
    assert_eq!(req["n"], 1);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p wereply_lib deepseek::tests::build_request_for_validation`
Expected: FAIL (test not present / function not updated).

**Step 3: Write minimal implementation**

- Add `validate_api_key` function in `deepseek.rs` that performs a minimal request.
- Update `save_api_key` command to call validation; on failure delete key and return error.

**Step 4: Run test to verify it passes**

Run: `cargo test -p wereply_lib deepseek::tests::build_request_for_validation`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/deepseek.rs src-tauri/src/lib.rs src-tauri/src/types.rs
git commit -m "feat: validate api key on save"
```

---

### Task 3: IPC 监听控制与 Rust 协调

**Files:**
- Modify: `src-tauri/src/ipc.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/agent.rs`
- Test: `src-tauri/src/ipc.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn envelope_rejects_empty_type() {
    let mut env = IpcEnvelope::new("", serde_json::json!({}));
    env.r#type = "".to_string();
    assert!(parse_envelope(&serde_json::to_string(&env).unwrap()).is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p wereply_lib ipc::tests::envelope_rejects_empty_type`
Expected: FAIL (test not present).

**Step 3: Write minimal implementation**

- Add listen control payloads and `listen.start/pause/resume/stop` message handling.
- Rust `start_listening/pause_listening/resume_listening/stop_listening` must send IPC control messages to Agent.
- Agent read loop should update `agent_connected` and `agent.status` events.

**Step 4: Run test to verify it passes**

Run: `cargo test -p wereply_lib ipc::tests::envelope_rejects_empty_type`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/ipc.rs src-tauri/src/lib.rs src-tauri/src/agent.rs
git commit -m "feat: add listen control ipc"
```

---

### Task 4: Windows Agent 监听与写入

**Files:**
- Modify: `platform_agents/windows/wxauto_agent.py`
- Modify: `platform_agents/windows/requirements.txt`

**Step 1: Write the failing test**

N/A (agent is runtime script; test manually by running agent and observing JSON output).

**Step 2: Run to verify it fails**

Run: `python platform_agents/windows/wxauto_agent.py`
Expected: Currently only emits ready; no message/new nor listen controls.

**Step 3: Write minimal implementation**

- Implement listen loop using wxauto.
- Handle `listen.*` and `input.write`.
- Implement dedupe by message id/text+timestamp.

**Step 4: Run to verify it works**

Run: `python platform_agents/windows/wxauto_agent.py`
Expected: Emits `agent.ready`, reacts to `listen.start` and `input.write`.

**Step 5: Commit**

```bash
git add platform_agents/windows/wxauto_agent.py platform_agents/windows/requirements.txt
git commit -m "feat: implement windows agent"
```

---

### Task 5: macOS Agent 监听与写入

**Files:**
- Modify: `platform_agents/macos/wechat_agent.swift`
- Modify: `platform_agents/macos/scripts.applescript`

**Step 1: Write the failing test**

N/A (agent is runtime script; test manually by running agent and observing JSON output).

**Step 2: Run to verify it fails**

Run: `swift platform_agents/macos/wechat_agent.swift`
Expected: Only emits permission error; no listen controls.

**Step 3: Write minimal implementation**

- Implement Accessibility permission check.
- Add polling for frontmost WeChat window and latest message text.
- Handle `listen.*` and `input.write` with clipboard paste.

**Step 4: Run to verify it works**

Run: `swift platform_agents/macos/wechat_agent.swift`
Expected: Emits `agent.ready` and handles control messages.

**Step 5: Commit**

```bash
git add platform_agents/macos/wechat_agent.swift platform_agents/macos/scripts.applescript
git commit -m "feat: implement macos agent"
```

---

### Task 6: End-to-end verification

**Files:**
- Modify: `CHANGELOG.md`

**Step 1: Security review**

Check for hardcoded secrets and sensitive logging.

**Step 2: Run lint & clippy**

Run: `cargo clippy` and `npm run lint`

**Step 3: Run tests**

Run: `cargo test` and `npm test`

**Step 4: Update changelog**

Add entry in `CHANGELOG.md`.

**Step 5: Commit**

```bash
git add CHANGELOG.md
git commit -m "chore: update changelog"
```
