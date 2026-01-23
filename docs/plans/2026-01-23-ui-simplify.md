# WeReply UI Simplify & Reliability Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 简化界面为“按钮 + 回复区域 + 建议列表”，修复 API key 连接卡住问题，并补全日志链路。

**Architecture:** 前端减少展示信息，仅保留控制按钮/回复区域/建议列表；API key 入口移至设置弹窗；后台补充关键路径 tracing 日志并增加 API key 校验超时保护。UI 行为通过小型工具函数统一处理并带测试。

**Tech Stack:** React + TypeScript (Vite, Vitest), Tauri (Rust, tracing).

---

### Task 1: API key 保存流程防卡死 + 可测试逻辑

**Files:**
- Modify: `src/utils/apiKey.ts`
- Modify: `src/utils/apiKey.test.ts`
- Modify: `src/App.tsx`

**Step 1: Write the failing test**

```ts
import { describe, expect, it } from "vitest";
import { resolveApiKeySaveOutcome } from "./apiKey";

describe("api key save outcome", () => {
  it("returns failed when invoke throws", () => {
    const result = resolveApiKeySaveOutcome(null, new Error("invoke failed"));
    expect(result.status).toBe("failed");
    expect(result.message).toBe("连接失败");
    expect(result.apiKeySet).toBe(false);
    expect(result.clearInput).toBe(false);
  });
});
```

**Step 2: Run test to verify it fails**

Run: `npm test src/utils/apiKey.test.ts`
Expected: FAIL with "resolveApiKeySaveOutcome is not defined".

**Step 3: Write minimal implementation**

```ts
export const resolveApiKeySaveOutcome = (
  result: ApiResponse<null> | null,
  error?: unknown,
): { status: ApiKeyStatus; apiKeySet: boolean; clearInput: boolean; message: string } => {
  if (error) {
    return { status: "failed", apiKeySet: false, clearInput: false, message: "连接失败" };
  }
  if (result?.success) {
    return { status: "connected", apiKeySet: true, clearInput: true, message: "API 密钥已保存并连接成功" };
  }
  return { status: "failed", apiKeySet: false, clearInput: false, message: result?.message || "连接失败" };
};
```

**Step 4: Run test to verify it passes**

Run: `npm test src/utils/apiKey.test.ts`
Expected: PASS.

**Step 5: Update UI to use helper with try/catch**

```ts
try {
  const res = await commands.saveApiKey(apiKeyInput.trim());
  const outcome = resolveApiKeySaveOutcome(res);
  setApiKeyStatus(outcome.status);
  setApiKeySet(outcome.apiKeySet);
  if (outcome.clearInput) setApiKeyInput("");
  message[outcome.status === "connected" ? "success" : "error"](outcome.message);
} catch (err) {
  const outcome = resolveApiKeySaveOutcome(null, err);
  setApiKeyStatus(outcome.status);
  setApiKeySet(outcome.apiKeySet);
  message.error(outcome.message);
}
```

**Step 6: Run tests**

Run: `npm test`
Expected: PASS.

**Step 7: Commit**

```bash
git add src/utils/apiKey.ts src/utils/apiKey.test.ts src/App.tsx

git commit -m "fix: handle api key save failures"
```

---

### Task 2: 回复输入校验工具函数 + 单元测试

**Files:**
- Create: `src/utils/reply.ts`
- Create: `src/utils/reply.test.ts`
- Modify: `src/App.tsx`

**Step 1: Write the failing test**

```ts
import { describe, expect, it } from "vitest";
import { normalizeReplyText } from "./reply";

describe("reply normalization", () => {
  it("rejects empty", () => {
    expect(normalizeReplyText("  ")).toEqual({ ok: false, reason: "回复内容不能为空", text: "" });
  });

  it("rejects overlength", () => {
    const longText = "a".repeat(2001);
    expect(normalizeReplyText(longText)).toEqual({ ok: false, reason: "回复内容过长", text: "" });
  });

  it("accepts trimmed", () => {
    expect(normalizeReplyText(" hi ")).toEqual({ ok: true, text: "hi" });
  });
});
```

**Step 2: Run test to verify it fails**

Run: `npm test src/utils/reply.test.ts`
Expected: FAIL with "normalizeReplyText is not defined".

**Step 3: Write minimal implementation**

```ts
export const normalizeReplyText = (input: string):
  | { ok: true; text: string }
  | { ok: false; text: ""; reason: string } => {
  const trimmed = input.trim();
  if (!trimmed) {
    return { ok: false, text: "", reason: "回复内容不能为空" };
  }
  if (trimmed.length > 2000) {
    return { ok: false, text: "", reason: "回复内容过长" };
  }
  return { ok: true, text: trimmed };
};
```

**Step 4: Run test to verify it passes**

Run: `npm test src/utils/reply.test.ts`
Expected: PASS.

**Step 5: Use helper in App.tsx**

```ts
const normalized = normalizeReplyText(draftText);
if (!normalized.ok) {
  message.warning(normalized.reason);
  return;
}
const res = await commands.writeSuggestion(lastChatId, normalized.text);
```

**Step 6: Run tests**

Run: `npm test`
Expected: PASS.

**Step 7: Commit**

```bash
git add src/utils/reply.ts src/utils/reply.test.ts src/App.tsx

git commit -m "feat: add reply validation helper"
```

---

### Task 3: UI 精简 + 建议点击直接写入

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/App.css`

**Step 1: Update layout**

- 删除状态卡片与编辑区，仅保留：顶部品牌 + 设置按钮；控制按钮（开始/停止）；建议列表；回复输入框 + 回复按钮。
- API key 入口移入设置弹窗（Ant Design Modal）。
- 建议点击即调用 `writeSuggestion` 写入微信输入框，不发送；成功后清空回复输入框。

**Step 2: Run UI smoke test**

Run: `npm run build`
Expected: PASS.

**Step 3: Commit**

```bash
git add src/App.tsx src/App.css

git commit -m "feat: simplify main ui and quick write suggestions"
```

---

### Task 4: 后端日志补全 + API key 校验超时保护

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/agent.rs`
- Modify: `src-tauri/src/deepseek.rs`

**Step 1: Write failing test**

```rs
#[test]
fn normalize_timeout_caps() {
    assert_eq!(cap_timeout_ms(12_000), 8_000);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test cap_timeout_ms -p wereply`
Expected: FAIL with "cap_timeout_ms not found".

**Step 3: Implement helper + timeout wrapper**

```rs
fn cap_timeout_ms(timeout_ms: u64) -> u64 {
    timeout_ms.min(8_000).max(2_000)
}

pub async fn validate_api_key(...) -> Result<()> {
    let timeout_ms = cap_timeout_ms(config.timeout_ms);
    // build client with timeout_ms
    let response = tokio::time::timeout(Duration::from_millis(timeout_ms),
        client.post(url).bearer_auth(api_key).json(&request).send()
    ).await.context("DeepSeek 连接超时")??;
}
```

**Step 4: Add tracing logs**

- `save_api_key`: 开始/成功/失败（不记录 key）。
- `start_listening/stop_listening/write_suggestion`: 参数校验失败与成功路径。
- `agent`: ready/status/error/message.new 关键路径、建议生成完成。

**Step 5: Run tests**

Run: `cargo test`
Expected: PASS.

**Step 6: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/agent.rs src-tauri/src/deepseek.rs

git commit -m "feat: add tracing logs and api key timeout guard"
```

---

### Task 5: 全量校验与整理

**Step 1: Lint + Tests**

Run: `npm run lint` and `npm test`
Run: `cargo clippy` and `cargo test`

**Step 2: Update changelog**

Append entry in `CHANGELOG.md`.

**Step 3: Final commit**

```bash
git add CHANGELOG.md

git commit -m "chore: update changelog"
```
