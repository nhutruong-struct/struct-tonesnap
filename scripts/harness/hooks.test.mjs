// Tests for how the Claude Code hooks find files and checkouts, including Claude Code's git
// worktrees under .claude/worktrees/<name>/. Run by the `harness` gate of check.mjs, or directly:
// node --test scripts/harness
//
// The Vietnamese sample is assembled from code points, so this file passes the checks it tests.

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { join } from "node:path";
import { test } from "node:test";
import { MAIN_ROOT, ROOT, canonical, locate } from "./config.mjs";

const VIETNAMESE = `Xin ch${String.fromCharCode(0xe0)}o c${String.fromCharCode(0xe1)}c b${String.fromCharCode(0x1ea1)}n`;
// Nothing is created there: the hooks under test only read paths, and these files do not exist.
const WORKTREE = join(MAIN_ROOT, ".claude", "worktrees", "harness-test");
const main = canonical(MAIN_ROOT);
const worktree = canonical(WORKTREE);

/** Runs a hook as Claude Code does and returns its decision, or null when it lets the call through. */
function preTool(payload) {
  const r = spawnSync(process.execPath, [join(ROOT, "scripts/harness/hooks.mjs"), "pre-tool"], {
    input: JSON.stringify(payload),
    encoding: "utf8",
  });
  assert.equal(r.status, 0, r.stderr);
  return r.stdout ? JSON.parse(r.stdout).hookSpecificOutput : null;
}
const write = (cwd, file, content) => preTool({ cwd, tool_name: "Write", tool_input: { file_path: file, content } });

test("locate: a file in the main checkout keeps its path", () => {
  assert.deepEqual(locate(join(MAIN_ROOT, "src", "lib", "i18n.ts")), {
    root: main,
    path: "src/lib/i18n.ts",
    abs: join(main, "src", "lib", "i18n.ts"),
  });
});

test("locate: a file in a worktree gets its path inside the worktree", () => {
  const file = locate(join(WORKTREE, "src-tauri", "src", "i18n.rs"));
  assert.equal(file.root, worktree);
  assert.equal(file.path, "src-tauri/src/i18n.rs");
  assert.equal(locate(join(WORKTREE, "src", "popup")).root, worktree);
  assert.deepEqual(locate(WORKTREE), { root: worktree, path: "", abs: worktree });
});

test("locate: a relative path is taken from the base, a path outside the project is null", () => {
  assert.equal(locate("docs/PLAN.md", WORKTREE).abs, join(worktree, "docs", "PLAN.md"));
  assert.equal(locate("docs/PLAN.md", WORKTREE).root, worktree);
  assert.equal(locate(join(MAIN_ROOT, "..", "elsewhere", "a.md")), null);
  assert.equal(locate(undefined), null);
  assert.equal(locate(""), null);
});

test("pre-tool: Vietnamese is allowed in a worktree's i18n files and denied elsewhere in it", () => {
  assert.equal(write(WORKTREE, join(WORKTREE, "src", "lib", "i18n.ts"), `a: "${VIETNAMESE}",\n`), null);
  assert.equal(write(WORKTREE, join(WORKTREE, "src-tauri", "src", "i18n.rs"), `"${VIETNAMESE}"\n`), null);
  const denied = write(WORKTREE, join(WORKTREE, "docs", "notes.md"), `${VIETNAMESE}\n`);
  assert.equal(denied?.permissionDecision, "deny");
  assert.match(denied.permissionDecisionReason, / docs\/notes\.md:/);
});

test("pre-tool: protected paths are recognized inside a worktree", () => {
  const denied = write(WORKTREE, join(WORKTREE, "src-tauri", "gen", "schemas", "a.json"), "{}\n");
  assert.equal(denied?.permissionDecision, "deny");
  assert.match(denied.permissionDecisionReason, /src-tauri\/gen\/schemas\/a\.json is protected/);
});

test("pre-tool: a session in a worktree may still edit the main checkout's i18n files", () => {
  assert.equal(write(WORKTREE, join(MAIN_ROOT, "src", "lib", "i18n.ts"), `a: "${VIETNAMESE}",\n`), null);
});
