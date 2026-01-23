# Auto-Install Windows Agent Dependencies Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Automatically install Windows Python agent dependencies (wxauto, pyautogui, pyperclip) on first run for dev and release builds before starting the agent.

**Architecture:** Add a Rust-side preflight step in the agent startup path that checks for required Python modules, runs `python -m pip install -r platform_agents/windows/requirements.txt` when missing, and re-checks before spawning the agent. Guard with a per-process cache and a mutex to avoid redundant installs.

**Tech Stack:** Rust (tokio::process, tokio::time), Tauri AppHandle path resolution, Windows Python runtime.

---

### Task 1: Add unit tests for command construction helpers

**Files:**
- Modify: `src-tauri/src/agent.rs`
- Test: `src-tauri/src/agent.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn python_check_args_include_required_modules() {
    let args = python_check_args(&["wxauto", "pyautogui", "pyperclip"]);
    assert_eq!(args[0], "-c");
    assert!(args[1].contains("import wxauto"));
    assert!(args[1].contains("import pyautogui"));
    assert!(args[1].contains("import pyperclip"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p wereply_lib agent::tests::python_check_args_include_required_modules`
Expected: FAIL with "cannot find function `python_check_args`"

**Step 3: Write minimal implementation**

```rust
fn python_check_args(modules: &[&str]) -> Vec<String> {
    let mut script = String::new();
    for module in modules {
        script.push_str("import ");
        script.push_str(module);
        script.push('\n');
    }
    vec!["-c".to_string(), script]
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p wereply_lib agent::tests::python_check_args_include_required_modules`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/agent.rs
git commit -m "test: add python check args helper test"
```

### Task 2: Add unit tests for pip install args and requirements path builder

**Files:**
- Modify: `src-tauri/src/agent.rs`
- Test: `src-tauri/src/agent.rs`

**Step 1: Write the failing tests**

```rust
#[test]
fn pip_install_args_include_requirements_flag() {
    let args = pip_install_args("C:/path/requirements.txt");
    assert_eq!(args[0], "-m");
    assert_eq!(args[1], "pip");
    assert!(args.iter().any(|arg| arg == "-r"));
}

#[test]
fn windows_requirements_path_is_under_platform_agents() {
    let base = std::path::Path::new("C:/app");
    let path = windows_requirements_path(base);
    assert!(path.ends_with("platform_agents/windows/requirements.txt"));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p wereply_lib agent::tests::pip_install_args_include_requirements_flag`
Expected: FAIL with "cannot find function `pip_install_args`"

Run: `cargo test -p wereply_lib agent::tests::windows_requirements_path_is_under_platform_agents`
Expected: FAIL with "cannot find function `windows_requirements_path`"

**Step 3: Write minimal implementation**

```rust
fn pip_install_args(requirements: &str) -> Vec<String> {
    vec![
        "-m".to_string(),
        "pip".to_string(),
        "install".to_string(),
        "--disable-pip-version-check".to_string(),
        "--no-input".to_string(),
        "-r".to_string(),
        requirements.to_string(),
    ]
}

fn windows_requirements_path(base: &std::path::Path) -> std::path::PathBuf {
    base.join("platform_agents").join("windows").join("requirements.txt")
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p wereply_lib agent::tests::pip_install_args_include_requirements_flag`
Expected: PASS

Run: `cargo test -p wereply_lib agent::tests::windows_requirements_path_is_under_platform_agents`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/agent.rs
git commit -m "test: add pip args and requirements path tests"
```

### Task 3: Implement Windows dependency preflight in agent startup

**Files:**
- Modify: `src-tauri/src/agent.rs`
- Test: `src-tauri/src/agent.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn python_check_args_are_stable_for_three_modules() {
    let args = python_check_args(&["wxauto", "pyautogui", "pyperclip"]);
    assert_eq!(args.len(), 2);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p wereply_lib agent::tests::python_check_args_are_stable_for_three_modules`
Expected: FAIL if args not length 2

**Step 3: Write minimal implementation**

- Add async helper `ensure_windows_agent_dependencies` called by `start_agent` on Windows before spawning.
- Use `tokio::process::Command` to run `python` with `python_check_args` and `pip_install_args`.
- Add timeout for pip install (60s) and re-check.
- Use `tracing::info/warn` for logging.
- Add per-process cache with `AtomicBool` and a `tokio::sync::Mutex` guard.

**Step 4: Run test to verify it passes**

Run: `cargo test -p wereply_lib agent::tests::python_check_args_are_stable_for_three_modules`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/agent.rs
git commit -m "feat: auto-install windows agent dependencies"
```

### Task 4: Full verification

**Files:**
- Verify: `src-tauri/src/agent.rs`

**Step 1: Run Rust tests**

Run: `cargo test`
Expected: PASS

**Step 2: Run lint checks**

Run: `cargo clippy`
Expected: PASS

Run: `npm run lint`
Expected: PASS

**Step 3: Update changelog**

- Add entry in `CHANGELOG.md` for auto-install Windows agent dependencies.

**Step 4: Commit**

```bash
git add CHANGELOG.md

git commit -m "docs: update changelog for windows agent deps auto-install"
```
