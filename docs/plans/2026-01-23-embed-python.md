# Embed Windows Python Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bundle an embeddable Python 3.12 with the Windows app and prefer it at runtime for the Windows agent, including wxauto dependencies.

**Architecture:** Extend `build.rs` to download and unpack embeddable Python, install requirements into `resources/python/Lib/site-packages`, and ensure `_pth` enables site-packages. Update runtime agent launch to prefer embedded python and set env vars to use bundled site-packages.

**Tech Stack:** Rust build scripts, reqwest (blocking), zip, std::process, Tauri resource bundling.

---

### Task 1: Add tests for embedded python path/env selection

**Files:**
- Modify: `src-tauri/src/agent.rs`
- Test: `src-tauri/src/agent.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn embedded_python_paths_use_resource_layout() {
    let base = std::path::Path::new("C:/app/resources");
    let (python, site) = embedded_python_paths(base);
    assert!(python.ends_with("python/python.exe"));
    assert!(site.ends_with("python/Lib/site-packages"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test agent::tests::embedded_python_paths_use_resource_layout`
Expected: FAIL with "cannot find function `embedded_python_paths`"

**Step 3: Write minimal implementation**

```rust
fn embedded_python_paths(resource_dir: &Path) -> (PathBuf, PathBuf) {
    (
        resource_dir.join("python").join("python.exe"),
        resource_dir.join("python").join("Lib").join("site-packages"),
    )
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test agent::tests::embedded_python_paths_use_resource_layout`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/agent.rs
git commit -m "test: add embedded python path helper"
```

### Task 2: Add tests for python command env setup

**Files:**
- Modify: `src-tauri/src/agent.rs`
- Test: `src-tauri/src/agent.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn embedded_python_env_sets_pythonhome_and_pythonpath() {
    let base = std::path::Path::new("C:/app/resources");
    let env = embedded_python_env(base);
    assert!(env.iter().any(|(k, _)| k == "PYTHONHOME"));
    assert!(env.iter().any(|(k, _)| k == "PYTHONPATH"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test agent::tests::embedded_python_env_sets_pythonhome_and_pythonpath`
Expected: FAIL with "cannot find function `embedded_python_env`"

**Step 3: Write minimal implementation**

```rust
fn embedded_python_env(resource_dir: &Path) -> Vec<(String, String)> {
    let (python, site) = embedded_python_paths(resource_dir);
    vec![
        ("PYTHONHOME".to_string(), python.parent().unwrap().to_string_lossy().to_string()),
        ("PYTHONPATH".to_string(), site.to_string_lossy().to_string()),
        ("PYTHONNOUSERSITE".to_string(), "1".to_string()),
    ]
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test agent::tests::embedded_python_env_sets_pythonhome_and_pythonpath`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/agent.rs
git commit -m "test: add embedded python env helper"
```

### Task 3: Implement embedded python preference for Windows agent

**Files:**
- Modify: `src-tauri/src/agent.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn embedded_python_exists_flag_checks_exe_path() {
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path();
    std::fs::create_dir_all(base.join("python")).unwrap();
    std::fs::write(base.join("python").join("python.exe"), "").unwrap();
    assert!(embedded_python_exists(base));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test agent::tests::embedded_python_exists_flag_checks_exe_path`
Expected: FAIL with "cannot find function `embedded_python_exists`"

**Step 3: Write minimal implementation**

- Add helpers to resolve embedded python from resource dir
- Update `resolve_agent_command` to return python path + env overrides for Windows
- Update dependency preflight to use the resolved python path + env

**Step 4: Run test to verify it passes**

Run: `cargo test agent::tests::embedded_python_exists_flag_checks_exe_path`
Expected: PASS

**Step 5: Commit**

```bash
git add src-tauri/src/agent.rs

git commit -m "feat: prefer embedded python for windows agent"
```

### Task 4: Implement build.rs bundling of embedded python and deps

**Files:**
- Modify: `src-tauri/build.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/tauri.conf.json`

**Step 1: Write the failing test**

No direct unit tests; rely on build script logs and build success in release.

**Step 2: Implement build script**

- Add build-dependencies: `reqwest` (blocking), `zip`, `sha2` (optional), `tempfile` if needed
- Download embeddable Python zip into `target/python-cache`
- Extract into `src-tauri/resources/python/`
- Download `get-pip.py` and run to install pip
- Ensure `python312._pth` includes `Lib\\site-packages` and `import site`
- Install requirements into `resources/python/Lib/site-packages`
- Guard: run only on Windows + `PROFILE=release` or `WEREPLY_BUNDLE_PYTHON=1`

**Step 3: Update tauri bundle resources**

Add `"resources/**"` to `bundle.resources` in `tauri.conf.json`.

**Step 4: Verify**

Run: `cargo build` (debug)
Run: `cargo build --release` (should download once)

**Step 5: Commit**

```bash
git add src-tauri/build.rs src-tauri/Cargo.toml src-tauri/tauri.conf.json

git commit -m "feat: bundle embedded python for windows builds"
```

### Task 5: Verification and changelog

**Files:**
- Modify: `CHANGELOG.md`

**Step 1: Update changelog**

Add entry for bundling embedded Python and dependencies.

**Step 2: Run tests**

Run: `cargo test`
Run: `npm test`
Run: `cargo clippy`
Run: `npm run lint`

**Step 3: Commit**

```bash
git add CHANGELOG.md

git commit -m "docs: update changelog for embedded python bundling"
```
