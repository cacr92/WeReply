# Connection Diagnostics Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an in-app DeepSeek connection diagnostics button that returns detailed status for chat and models endpoints.

**Architecture:** Frontend triggers a new Tauri command that probes `/chat/completions` and `/models` with the provided or stored API key. Backend returns a typed diagnostics payload; frontend formats it into a short summary and detailed lines in Settings.

**Tech Stack:** Tauri (Rust), React + TypeScript, Ant Design `message`.

---

### Task 1: Add diagnostics formatter (frontend, TDD)

**Files:**
- Create: `src/utils/diagnostics.ts`
- Test: `src/utils/diagnostics.test.ts`

**Step 1: Write the failing test**

```ts
import { describe, expect, it } from "vitest";
import { summarizeDiagnostics } from "./diagnostics";

describe("summarize diagnostics", () => {
  it("returns ok summary when both endpoints are ok", () => {
    const result = summarizeDiagnostics({
      base_url: "https://api.deepseek.com",
      model: "deepseek-chat",
      chat: { ok: true, status: 200, message: "ok" },
      models: { ok: true, status: 200, message: "ok" },
    });
    expect(result.ok).toBe(true);
    expect(result.message).toBe("连接诊断通过");
    expect(result.lines[0]).toContain("聊天接口");
  });
});
```

**Step 2: Run test to verify it fails**

Run: `npm test -- src/utils/diagnostics.test.ts`  
Expected: FAIL (module not found)

**Step 3: Write minimal implementation**

```ts
export const summarizeDiagnostics = (...) => ({ ... });
```

**Step 4: Run test to verify it passes**

Run: `npm test -- src/utils/diagnostics.test.ts`  
Expected: PASS

**Step 5: Commit**

```bash
git add src/utils/diagnostics.ts src/utils/diagnostics.test.ts
git commit -m "feat: add diagnostics formatter"
```

---

### Task 2: Add backend diagnostics command (Tauri)

**Files:**
- Modify: `src-tauri/src/types.rs`
- Modify: `src-tauri/src/deepseek.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/bindings.ts`

**Step 1: Write failing test (if applicable)**

Add a small unit test for diagnostics message formatting or status normalization if introducing helpers.

**Step 2: Implement command and types**

Add `DeepseekDiagnostics` and `DeepseekEndpointStatus` types, plus a `diagnose_deepseek` command that probes chat and models with a timeout guard.

**Step 3: Run backend tests**

Run: `cargo test`  
Expected: PASS

**Step 4: Commit**

```bash
git add src-tauri/src/types.rs src-tauri/src/deepseek.rs src-tauri/src/lib.rs src/bindings.ts
git commit -m "feat: add deepseek diagnostics command"
```

---

### Task 3: Wire diagnostics button in UI

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/App.css`

**Step 1: Add UI and state**

Add a “连接诊断” button in Settings → API 密钥 panel, plus a compact diagnostics display block.

**Step 2: Run frontend tests**

Run: `npm test`  
Expected: PASS

**Step 3: Commit**

```bash
git add src/App.tsx src/App.css
git commit -m "feat: add deepseek diagnostics button"
```

---

### Task 4: Update changelog + verify

**Files:**
- Modify: `CHANGELOG.md`

**Step 1: Update changelog**

Add an Unreleased bullet describing the diagnostics button.

**Step 2: Full verification**

Run:
```bash
npm test
npm run lint
cargo test
cargo clippy
```

**Step 3: Commit**

```bash
git add CHANGELOG.md
git commit -m "chore: update changelog for diagnostics"
```
