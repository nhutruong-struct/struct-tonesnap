---
paths:
  - "src-tauri/**/*.rs"
---

# Rust code rules

★ = enforced by `invariants.mjs`; (clippy) = by the lints in `src-tauri/Cargo.toml`. The PostToolUse hook reports violations and runs rustfmt after each edit.

- ★ `unsafe` only in `src-tauri/src/platform/`. Release every handle on every path.
- (clippy) Every `unsafe` block is preceded by a `// SAFETY:` comment: which handle is valid, which buffer is large enough, which thread has COM.
- ★ No `.unwrap()` except directly after `.lock()`. Use `?`, a fallback such as `unwrap_or_default()`, or `.expect("why this cannot fail")`.
- ★ (clippy) No `todo!`, `unimplemented!`, `panic!`, `dbg!` or `println!` in shipped code. Commands return `Result` with an error code the UI translates (`CommandError`).
- ★ (clippy) `#[allow(...)]` only with a reason: `#[allow(lint, reason = "why")]`.
- Slow or blocking work (Win32, COM, file and network I/O) runs off Tauri's main thread. Synchronous `#[tauri::command]`s run on it, so a command that touches files, the credential store, the network or the clipboard is `async` or hands off to its own thread.
- Code outside `platform/` calls the platform-neutral functions in `platform/mod.rs` instead of adding `#[cfg(windows)]` branches.

## Tests

- New pure logic gets unit tests in the same file (`#[cfg(test)] mod tests`).
- Data Rust sends to the UI gets a test of its JSON shape.
- Provider code is tested against `llm/test_server.rs`.
- Win32 code is tested against real windows the test owns (`platform/win/test_window.rs`) in `mod integration_tests`, marked `#[ignore = "win32 integration: run by check.mjs --full"]`. Windows tests are written here; macOS tests on a Mac.
