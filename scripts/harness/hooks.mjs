// Claude Code hook entry point; wired up in .claude/settings.json.
//
//   session-start  inject where the project stands (open PLAN items, last check)
//   pre-tool       deny edits to generated/locked files, content with secrets and new
//                  Vietnamese prose outside the i18n files; ask before manifest edits and
//                  destructive or dependency-changing shell commands
//   post-tool      format edited Rust files; report project-rule violations in the edited file;
//                  remember which source groups the agent changed
//   stop           re-run the gates for changed groups; block the stop while they fail
//
// Reads the hook payload as JSON on stdin and answers with JSON on stdout.

import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, realpathSync } from "node:fs";
import { basename, dirname, isAbsolute, join, relative } from "node:path";
import {
  BLOCKED_COMMANDS,
  MANIFESTS,
  PROTECTED_PATHS,
  ROOT,
  SECRET_PATTERNS,
  TEXT_EXTENSIONS,
  toolEnv,
} from "./config.mjs";
import { fileViolations } from "./invariants.mjs";
import { changedGroups, fingerprints, readState, writeState } from "./state.mjs";

const MAX_STOP_BLOCKS = 3;
const EDIT_TOOLS = new Set(["Edit", "Write", "MultiEdit", "NotebookEdit"]);

function readInput() {
  try {
    return JSON.parse(readFileSync(0, "utf8") || "{}");
  } catch {
    return {};
  }
}

const reply = (obj) => process.stdout.write(JSON.stringify(obj));

/**
 * Absolute path with Windows 8.3 short names (C:\Users\ABC~1) expanded, so paths from
 * different sources compare equal. Works for files that do not exist yet.
 */
function canonical(path) {
  try {
    return realpathSync.native(path);
  } catch {
    const parent = dirname(path);
    return parent === path ? path : join(canonical(parent), basename(path));
  }
}

/** Project-relative forward-slash path, or null when the file is outside the project. */
function projectPath(file) {
  if (!file) return null;
  const abs = canonical(isAbsolute(file) ? file : join(ROOT, file));
  const rel = relative(canonical(ROOT), abs).replaceAll("\\", "/");
  return rel.startsWith("..") || isAbsolute(rel) ? null : rel;
}

/** Text an edit tool is about to write. */
function newContent(input) {
  const t = input.tool_input ?? {};
  return [t.content, t.new_string, t.new_source, ...(t.edits ?? []).map((e) => e.new_string)]
    .filter((x) => typeof x === "string")
    .join("\n");
}

const readProjectFile = (path) => (existsSync(join(ROOT, path)) ? readFileSync(join(ROOT, path), "utf8") : "");

/** The whole file as it will be after the edit, so rules see the edit in context. */
function contentAfterEdit(input, before) {
  const t = input.tool_input ?? {};
  if (typeof t.content === "string") return t.content;
  const edits = Array.isArray(t.edits) ? t.edits : [t];
  return edits.reduce((text, e) => {
    if (typeof e.old_string !== "string" || typeof e.new_string !== "string") return text;
    return e.replace_all ? text.split(e.old_string).join(e.new_string) : text.replace(e.old_string, () => e.new_string);
  }, before);
}

/** Violations present after the edit that were not there before (matched by rule and source line). */
function introducedViolations(path, before, after, rules) {
  const key = (v) => `${v.rule}|${v.snippet}`;
  const existing = new Set(fileViolations(path, before, rules).map(key));
  return fileViolations(path, after, rules).filter((v) => v.severity === "error" && !existing.has(key(v)));
}

// ---------------------------------------------------------------------------------------------

function sessionStart() {
  const state = readState();
  // Changes made before this session are not the agent's to fix unprompted.
  state.snapshot = fingerprints();
  state.dirty = [];
  state.stopBlocks = 0;
  writeState(state);

  const lines = ["ToneSnap harness: project status (from scripts/harness/hooks.mjs)."];
  try {
    const plan = readFileSync(join(ROOT, "docs/PLAN.md"), "utf8");
    const section = /## ([^\n]*progress[^\n]*)\n([\s\S]*?)(?=\n## |$)/i.exec(plan);
    if (section) {
      const done = (section[2].match(/- \[x\]/gi) ?? []).length;
      const open = [...section[2].matchAll(/- \[ \] (.+)/g)].map((m) => m[1]);
      lines.push(`- ${section[1]}: ${done}/${done + open.length} done. Open items:`);
      for (const item of open) lines.push(`  - ${item}`);
    }
  } catch {
    lines.push("- docs/PLAN.md not found.");
  }
  const last = state.lastCheck;
  if (last) {
    lines.push(
      `- Last check (${last.mode}, ${last.at}): ${last.ok ? "PASS" : `FAIL in ${last.failed.join(", ")}`}.` +
        (last.ok ? "" : " Fix these before new work if the user agrees."),
    );
  } else {
    lines.push("- No check has run yet: `node scripts/harness/check.mjs`.");
  }
  lines.push(
    "- The Stop hook re-runs the gates for code you change and blocks the turn while they fail. Follow CLAUDE.md (workflow, definition of done).",
  );
  reply({ hookSpecificOutput: { hookEventName: "SessionStart", additionalContext: lines.join("\n") } });
}

function preTool(input) {
  const name = input.tool_name;
  const deny = (reason) =>
    reply({
      hookSpecificOutput: { hookEventName: "PreToolUse", permissionDecision: "deny", permissionDecisionReason: reason },
    });

  if (EDIT_TOOLS.has(name)) {
    const path = projectPath(input.tool_input?.file_path ?? input.tool_input?.notebook_path);
    if (path) {
      const rule = PROTECTED_PATHS.find((p) => p.pattern.test(path));
      if (rule) return deny(`Harness: ${path} is protected (${rule.reason}).`);
    }
    const content = newContent(input);
    const secret = SECRET_PATTERNS.find((s) => s.pattern.test(content));
    if (secret) {
      return deny(
        `Harness: the new content contains what looks like a real ${secret.name}. Keys belong in the OS credential store (secrets.rs), never in files.`,
      );
    }
    if (path && TEXT_EXTENSIONS.test(path)) {
      const before = readProjectFile(path);
      const added = introducedViolations(path, before, contentAfterEdit(input, before), ["language"]);
      if (added.length) {
        return deny(
          `Harness: the repository is written in English; Vietnamese belongs only in src/lib/i18n.ts, src-tauri/src/i18n.rs and Vietnamese test data. ` +
            `New Vietnamese text in ${path}:\n` +
            added.map((v) => `  line ${v.line}: ${v.snippet.slice(0, 80)}`).join("\n"),
        );
      }
    }
    if (path && MANIFESTS.some((m) => m.test(path))) {
      return reply({
        hookSpecificOutput: {
          hookEventName: "PreToolUse",
          permissionDecision: "ask",
          permissionDecisionReason: `Harness: ${path} controls dependencies and build settings; the user approves every change to it.`,
        },
      });
    }
    return;
  }

  if (name === "Bash" || name === "PowerShell") {
    const command = input.tool_input?.command ?? "";
    const rule = BLOCKED_COMMANDS.find((c) => c.pattern.test(command));
    if (rule) {
      reply({
        hookSpecificOutput: {
          hookEventName: "PreToolUse",
          permissionDecision: "ask",
          permissionDecisionReason: `Harness: risky command (${rule.reason}). Confirm only if you meant it.`,
        },
      });
    }
  }
}

function postTool(input) {
  const notes = [];
  if (EDIT_TOOLS.has(input.tool_name)) {
    const path = projectPath(input.tool_input?.file_path);
    if (path?.startsWith("src-tauri/") && path.endsWith(".rs")) {
      const r = spawnSync("rustfmt", ["--edition", "2021", join(ROOT, path)], { env: toolEnv(), encoding: "utf8" });
      if (r.status !== 0 && !r.error) {
        notes.push(
          `rustfmt could not format ${path}, which usually means a syntax error:\n${(r.stderr ?? "").slice(0, 1500)}`,
        );
      }
    }
    // Prettier skips what .prettierignore lists (Markdown, docs, src-tauri…).
    if (path && /\.(tsx?|jsx?|mjs|css|json|html)$/.test(path)) {
      const prettier = join(ROOT, "node_modules/prettier/bin/prettier.cjs");
      const r = spawnSync(
        process.execPath,
        [prettier, "--write", "--ignore-unknown", "--log-level", "warn", join(ROOT, path)],
        { cwd: ROOT, env: toolEnv(), encoding: "utf8" },
      );
      if (r.status !== 0 && !r.error) {
        notes.push(
          `Prettier could not format ${path}, which usually means a syntax error:\n${(r.stderr ?? "").slice(0, 1500)}`,
        );
      }
    }
    // Immediate feedback on the code rules, instead of waiting for the Stop hook.
    if (path && TEXT_EXTENSIONS.test(path) && existsSync(join(ROOT, path))) {
      const found = fileViolations(path, readProjectFile(path));
      if (found.length) {
        notes.push(
          `Project rules flagged in ${path} (errors block the end of the turn; see CLAUDE.md "Code rules"):\n` +
            found
              .map((v) => `  [${v.rule}${v.severity === "warning" ? ", warning" : ""}] line ${v.line}: ${v.message}`)
              .join("\n"),
        );
      }
    }
  }

  const state = readState();
  const now = fingerprints();
  const changed = state.snapshot ? changedGroups(state.snapshot, now) : [];
  state.dirty = [...new Set([...(state.dirty ?? []), ...changed])];
  state.snapshot = now;
  writeState(state);

  if (notes.length) {
    reply({ hookSpecificOutput: { hookEventName: "PostToolUse", additionalContext: notes.join("\n\n") } });
  }
}

function stop() {
  const state = readState();
  if (!state.dirty?.length) return;

  const started = Date.now();
  const r = spawnSync(process.execPath, [join(ROOT, "scripts/harness/check.mjs"), "--changed"], {
    cwd: ROOT,
    env: toolEnv(),
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  const output = `${r.stdout ?? ""}${r.stderr ?? ""}`.trim();
  const secs = Math.round((Date.now() - started) / 1000);
  const after = readState(); // check.mjs updated dirty/lastCheck

  if (r.status === 0) {
    reply({ systemMessage: `Harness: all gates passed for ${state.dirty.join(" + ")} changes (${secs}s).` });
    return;
  }

  after.stopBlocks = (after.stopBlocks ?? 0) + 1;
  if (after.stopBlocks > MAX_STOP_BLOCKS) {
    after.stopBlocks = 0;
    writeState(after);
    reply({
      systemMessage: `Harness: checks still fail after ${MAX_STOP_BLOCKS} attempts; stopping anyway. Run \`node scripts/harness/check.mjs\` to see them.\n${output.slice(-1500)}`,
    });
    return;
  }
  writeState(after);
  reply({
    decision: "block",
    reason:
      `Harness checks failed (attempt ${after.stopBlocks}/${MAX_STOP_BLOCKS}). Fix the problems below, ` +
      `or explain to the user why they cannot be fixed now, before finishing.\n\n${output.slice(-4000)}`,
  });
}

const handlers = { "session-start": sessionStart, "pre-tool": preTool, "post-tool": postTool, stop };
const event = process.argv[2];
try {
  handlers[event]?.(readInput());
} catch (e) {
  // A broken hook must never block work; report it instead.
  reply({ systemMessage: `Harness hook "${event}" crashed: ${e.message}` });
}
