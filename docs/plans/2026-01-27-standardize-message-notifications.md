# Message Notification Standardization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Standardize all frontend notifications through a single `notify` wrapper with consistent templates and durations.

**Architecture:** Add `src/utils/notify.ts` to wrap AntD `message` with formatting rules, then replace all `message.*` usages in `src/App.tsx` with `notify.*` calls.

**Tech Stack:** React 19, TypeScript, Ant Design 5, Vitest.

---

### Task 1: Define notify formatting tests (TDD)

**Files:**
- Create: `src/utils/notify.test.ts`

**Step 1: Write the failing test**

```ts
import { describe, expect, it } from "vitest";
import { formatNotifyMessage, resolveNotifyDetail } from "./notify";

describe("notify formatting", () => {
  it("uses detail when provided", () => {
    expect(formatNotifyMessage("保存失败", "网络异常")).toBe("保存失败：网络异常");
  });

  it("falls back when detail is missing", () => {
    expect(formatNotifyMessage("保存失败", undefined, "请稍后重试")).toBe(
      "保存失败：请稍后重试",
    );
  });

  it("avoids duplicate detail", () => {
    expect(formatNotifyMessage("连接失败", "连接失败")).toBe("连接失败");
  });

  it("extracts error message", () => {
    expect(resolveNotifyDetail(new Error("出错了"))).toBe("出错了");
  });
});
```

**Step 2: Run test to verify it fails**

Run: `npm test`
Expected: FAIL with "Cannot find module './notify'" or missing export errors.

**Step 3: Write minimal implementation**

Create `src/utils/notify.ts` with `formatNotifyMessage/resolveNotifyDetail` and `notify` wrapper.

**Step 4: Run test to verify it passes**

Run: `npm test`
Expected: PASS with all notify tests green.

**Step 5: Commit**

```bash
git add src/utils/notify.ts src/utils/notify.test.ts
git commit -m "feat: add notify wrapper for message"
```

---

### Task 2: Replace App notifications with notify

**Files:**
- Modify: `src/App.tsx`

**Step 1: Replace message usage**

Update imports to use `notify`, and replace all `message.*` calls with `notify.*` using action/detail formatting.

**Step 2: Run test to verify it passes**

Run: `npm test`
Expected: PASS with all existing tests green.

**Step 3: Commit**

```bash
git add src/App.tsx
git commit -m "refactor: standardize app notifications"
```

---

### Task 3: Verification

**Files:**
- N/A

**Step 1: Run lint**

Run: `npm run lint`
Expected: exit 0, no TypeScript errors.

**Step 2: Run Rust checks**

Run: `cargo clippy`
Expected: exit 0, no warnings.

**Step 3: Run tests**

Run: `npm test` and `cargo test`
Expected: all tests pass.

**Step 4: Commit (if needed)**

```bash
git status -sb
```
Expected: clean or only known files.
