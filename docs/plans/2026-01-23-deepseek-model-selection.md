# DeepSeek Model Selection Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix DeepSeek connection failures by using official endpoints and minimal payloads, then add model listing and selection with persistence and a post-save fetch flow.

**Architecture:** Backend deepseek client builds endpoint URLs from base_url and keeps payloads minimal. A new list_models command fetches `/models`, filters to supported models, and the selected model is stored in config.json. Frontend fetches models only after a successful API key save, then persists selection via a new command.

**Tech Stack:** Tauri (Rust), React + TypeScript (Vite), reqwest, serde_json, Ant Design

---

### Task 1: Add frontend model utilities (tests first)

**Files:**
- Create: `src/utils/models.test.ts`
- Create: `src/utils/models.ts`

**Step 1: Write the failing test**

```typescript
import { DEFAULT_MODELS, normalizeModels, resolveModelSelection } from "./models";

test("normalizeModels falls back to defaults when list is empty", () => {
  expect(normalizeModels([])).toEqual(DEFAULT_MODELS);
});

test("normalizeModels filters and preserves allowed order", () => {
  expect(normalizeModels(["other", "deepseek-reasoner", "deepseek-chat"]))
    .toEqual(["deepseek-chat", "deepseek-reasoner"]);
});

test("resolveModelSelection keeps selection when available", () => {
  const result = resolveModelSelection(["deepseek-chat"], "deepseek-chat");
  expect(result.selected).toBe("deepseek-chat");
  expect(result.changed).toBe(false);
});

test("resolveModelSelection falls back to first model", () => {
  const result = resolveModelSelection(["deepseek-chat"], "deepseek-reasoner");
  expect(result.selected).toBe("deepseek-chat");
  expect(result.changed).toBe(true);
});
```

**Step 2: Run test to verify it fails**

Run: `npm test src/utils/models.test.ts`
Expected: FAIL (module not found)

**Step 3: Write minimal implementation**

```typescript
export const DEFAULT_MODELS = ["deepseek-chat", "deepseek-reasoner"] as const;

export const normalizeModels = (models: string[]): string[] => {
  const normalized = DEFAULT_MODELS.filter((model) => models.includes(model));
  return normalized.length > 0 ? normalized : [...DEFAULT_MODELS];
};

export const resolveModelSelection = (
  models: string[],
  selected: string,
): { selected: string; changed: boolean } => {
  if (models.includes(selected)) {
    return { selected, changed: false };
  }
  const next = models[0] ?? DEFAULT_MODELS[0];
  return { selected: next, changed: true };
};
```

**Step 4: Run test to verify it passes**

Run: `npm test src/utils/models.test.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/utils/models.ts src/utils/models.test.ts
git commit -m "test: add model selection utilities"
```

---

### Task 2: Add backend DeepSeek request tests (tests first)

**Files:**
- Modify: `src-tauri/src/deepseek.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn build_request_payload_is_minimal() {
    let req = build_request("hi", "deepseek-chat");
    assert!(req.get("temperature").is_none());
    assert!(req.get("top_p").is_none());
    assert!(req.get("n").is_none());
}

#[test]
fn build_chat_url_trims_slash() {
    let url = build_chat_url("https://api.deepseek.com/");
    assert_eq!(url, "https://api.deepseek.com/chat/completions");
}

#[test]
fn normalize_models_filters_and_fallbacks() {
    let list = normalize_models(vec!["x".to_string()]);
    assert_eq!(list, vec!["deepseek-chat", "deepseek-reasoner"]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test deepseek::tests::build_request_payload_is_minimal`
Expected: FAIL (function not found / assertions fail)

**Step 3: Write minimal implementation**

Add helper functions `build_chat_url` and `normalize_models`, and adjust `build_request` signature to remove extra parameters.

**Step 4: Run test to verify it passes**

Run: `cargo test deepseek::tests::build_request_payload_is_minimal`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/deepseek.rs
git commit -m "test: cover deepseek request defaults"
```

---

### Task 3: Implement backend DeepSeek fixes + model list command

**Files:**
- Modify: `src-tauri/src/deepseek.rs`
- Modify: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/types.rs` (if needed)

**Step 1: Write the failing test**

Add a test in `src-tauri/src/config.rs` to ensure invalid model is rejected by validation or to verify config load uses stored model.

**Step 2: Run test to verify it fails**

Run: `cargo test config::tests::<new_test_name>`
Expected: FAIL

**Step 3: Write minimal implementation**

- Change DeepSeek endpoints to `/chat/completions` and `/models`.
- Remove temperature/top_p/n from request payloads.
- Add `list_models` function and `is_supported_model` check.
- Persist only `deepseek_model` in `config.json` and load it at startup.
- Add Tauri commands: `list_models` and `set_deepseek_model`.

**Step 4: Run test to verify it passes**

Run: `cargo test config::tests::<new_test_name>`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/deepseek.rs src-tauri/src/config.rs src-tauri/src/lib.rs src-tauri/src/types.rs
git commit -m "feat: align deepseek endpoints and model selection"
```

---

### Task 4: Frontend wiring for model list + selection

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/App.css`
- Modify: `src/bindings.ts`

**Step 1: Write the failing test**

If needed, add a minimal test for selection resolution behavior to ensure UI logic uses the new helpers.

**Step 2: Run test to verify it fails**

Run: `npm test src/utils/models.test.ts`
Expected: FAIL (if new cases added)

**Step 3: Write minimal implementation**

- Fetch models after a successful API key save.
- Use `normalizeModels` and `resolveModelSelection` to set list and selected value.
- Add UI select for model choice and persist selection via `setDeepseekModel`.

**Step 4: Run test to verify it passes**

Run: `npm test src/utils/models.test.ts`
Expected: PASS

**Step 5: Commit**

```bash
git add src/App.tsx src/App.css src/bindings.ts
git commit -m "feat: add model selection in settings"
```

---

### Task 5: Full verification and cleanup

**Files:**
- Modify: `CHANGELOG.md`

**Step 1: Run full test suites**

Run: `npm test`
Expected: PASS

Run: `cargo test`
Expected: PASS

**Step 2: Run quality checks**

Run: `npm run lint`
Expected: PASS

Run: `cargo clippy`
Expected: PASS

**Step 3: Update CHANGELOG**

Add user-facing entries for fixed DeepSeek connection and model selection.

**Step 4: Commit**

```bash
git add CHANGELOG.md
git commit -m "chore: update changelog"
```
