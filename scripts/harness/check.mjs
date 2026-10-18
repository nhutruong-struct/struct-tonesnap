// Quality gates for ToneSnap. Exit code 0 means every gate that ran passed.
//
//   node scripts/harness/check.mjs                 fast: invariants, rustfmt, tsc, clippy, unit tests
//   node scripts/harness/check.mjs --full          + production frontend build, Win32 integration tests
//   node scripts/harness/check.mjs --only=input    key-press tests; only while nobody uses the PC
//   node scripts/harness/check.mjs --changed       only gates for groups changed since the last pass
//   node scripts/harness/check.mjs --only=tsc,test pick gates by id
//   --fix                                          run cargo fmt, eslint --fix and prettier --write first
//
// Results are recorded in .claude/.state/harness.json for the hooks.

import { spawnSync } from "node:child_process";
import { dirname, join } from "node:path";
import { GATES_FOR_GROUP, MANIFEST, ROOT, toolEnv } from "./config.mjs";
import { runInvariants } from "./invariants.mjs";
import { changedGroups, fingerprints, readState, writeState } from "./state.mjs";

const PRETTIER = join(ROOT, "node_modules/prettier/bin/prettier.cjs");
const ESLINT = join(ROOT, "node_modules/eslint/bin/eslint.js");
// pnpm through the corepack that ships with Node, so terminals without npm's folder on PATH
// (those hosted by MSIX apps) still find it.
const PNPM = join(dirname(process.execPath), "node_modules/corepack/dist/pnpm.js");

const args = new Set(process.argv.slice(2).filter((a) => !a.startsWith("--only=")));
const only = process.argv
  .find((a) => a.startsWith("--only="))
  ?.slice(7)
  .split(",");
const env = toolEnv();
const node = process.execPath;

/** Lines worth showing from a failed tool: diagnostics first, then the tail. */
function digest(output, limit = 25) {
  // Node prints a warning on every start when NODE_EXTRA_CA_CERTS (set by some corporate TLS-inspection tools) cannot be read.
  const lines = output.split(/\r?\n/).filter((l) => l.trim() && !/Ignoring extra certs/.test(l));
  // rustc's location lines ("  --> src/lib.rs:3:5") are matched without a regex: CodeQL reads a
  // `-->` in a regex as an HTML comment filter (js/bad-tag-filter).
  const important = lines.filter(
    (l) =>
      /error|warning|FAILED|\bFAIL\b|×|panicked|Diff in|failures:|test result|TS\d{4}/i.test(l) ||
      l.trimStart().startsWith("--> "),
  );
  return (important.length ? important : lines.slice(-limit)).slice(0, limit);
}

/** Long enough for a cold Rust build; a hung test must not stall the Stop hook forever. */
const TOOL_TIMEOUT_MS = 20 * 60 * 1000;

/** `requireTests`: fail when a test filter matched nothing, instead of passing with 0 tests. */
function tool(command, commandArgs, { requireTests = false } = {}) {
  return () => {
    const r = spawnSync(command, commandArgs, {
      cwd: ROOT,
      env,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      timeout: TOOL_TIMEOUT_MS,
    });
    if (r.error?.code === "ETIMEDOUT") return { ok: false, details: [`${command} timed out after 20 minutes`] };
    if (r.error) return { ok: false, details: [`could not start ${command}: ${r.error.message}`] };
    const output = `${r.stdout}\n${r.stderr}`;
    if (r.status === 0 && requireTests && /running 0 tests/.test(output)) {
      return { ok: false, details: ["no test matched the filter; were the test modules renamed?"] };
    }
    return { ok: r.status === 0, details: r.status === 0 ? [] : digest(output) };
  };
}

const GATES = [
  {
    id: "invariants",
    title: "project rules (contracts, code rules, language, privacy, secrets)",
    run: () => {
      const results = runInvariants();
      const errors = results.flatMap((r) => r.errors.map((e) => `[${r.name}] ${e}`));
      const warnings = results.flatMap((r) => r.warnings.map((w) => `[${r.name}] ${w}`));
      return { ok: errors.length === 0, details: errors, warnings };
    },
  },
  {
    id: "rustfmt",
    title: "cargo fmt --check",
    run: tool("cargo", ["fmt", "--manifest-path", MANIFEST, "--check"]),
  },
  {
    id: "prettier",
    title: "prettier --check (TS, CSS, JSON, scripts)",
    run: tool(node, [PRETTIER, "--check", "--log-level", "warn", "."]),
  },
  {
    id: "tsc",
    title: "TypeScript (tsc)",
    run: tool(node, [join(ROOT, "node_modules/typescript/bin/tsc"), "-p", ROOT]),
  },
  {
    id: "eslint",
    title: "ESLint (type-aware, React hooks)",
    run: tool(node, [ESLINT, "--max-warnings", "0", "src"]),
  },
  {
    id: "clippy",
    title: "cargo clippy -D warnings",
    run: tool("cargo", [
      "clippy",
      "--manifest-path",
      MANIFEST,
      "--all-targets",
      "--message-format",
      "short",
      "--",
      "-D",
      "warnings",
    ]),
  },
  {
    id: "test",
    title: "cargo test --lib",
    run: tool("cargo", ["test", "--manifest-path", MANIFEST, "--lib"], { requireTests: true }),
  },
  {
    id: "vitest",
    title: "frontend tests (vitest)",
    run: tool(node, [join(ROOT, "node_modules/vitest/vitest.mjs"), "run"]),
  },
  {
    id: "build",
    title: "vite build",
    full: true,
    run: tool(node, [join(ROOT, "node_modules/vite/bin/vite.js"), "build"]),
  },
  {
    id: "win32",
    title: "Win32 integration tests (UIA, OCR, clipboard, windows)",
    full: true,
    run: tool(
      "cargo",
      ["test", "--manifest-path", MANIFEST, "--lib", "--", "--ignored", "--test-threads=1", "integration_tests"],
      {
        requireTests: true,
      },
    ),
  },
  {
    id: "deps",
    title: "dependency audit: cargo deny (advisories, licenses, sources), pnpm audit; needs network",
    full: true,
    run: all(
      tool("cargo", ["deny", "--manifest-path", MANIFEST, "check"]),
      tool(node, [PNPM, "audit", "--audit-level", "moderate"]),
    ),
  },
  {
    // Sends real key presses and takes focus, so it never runs unless asked for by name.
    id: "input",
    title: "input tests (Ctrl+C / Ctrl+V into a test window)",
    explicit: true,
    run: tool(
      "cargo",
      ["test", "--manifest-path", MANIFEST, "--lib", "--", "--ignored", "--test-threads=1", "input_tests"],
      {
        requireTests: true,
      },
    ),
  },
];

/** Runs several tools as one gate; it passes only if all of them pass. */
function all(...runs) {
  return () => {
    const results = runs.map((run) => run());
    return { ok: results.every((r) => r.ok), details: results.flatMap((r) => r.details ?? []) };
  };
}

function selectGates() {
  if (only) return GATES.filter((g) => only.includes(g.id));
  if (args.has("--changed")) {
    const groups = readState().dirty ?? [];
    const ids = new Set(["invariants", ...groups.flatMap((g) => GATES_FOR_GROUP[g] ?? [])]);
    return GATES.filter((g) => ids.has(g.id));
  }
  return GATES.filter((g) => !g.explicit && (!g.full || args.has("--full")));
}

export function check() {
  if (args.has("--fix")) {
    const quiet = { cwd: ROOT, env, stdio: "ignore" };
    spawnSync("cargo", ["fmt", "--manifest-path", MANIFEST], quiet);
    spawnSync(node, [ESLINT, "--fix", "src"], quiet);
    spawnSync(node, [PRETTIER, "--write", "--log-level", "warn", "."], quiet);
  }

  const gates = selectGates();
  const mode = only ? `only ${only}` : args.has("--changed") ? "changed" : args.has("--full") ? "full" : "fast";
  console.log(`ToneSnap check (${mode})`);
  const failed = [];
  const before = fingerprints();
  for (const gate of gates) {
    const started = Date.now();
    const { ok, details = [], warnings = [] } = gate.run();
    const secs = ((Date.now() - started) / 1000).toFixed(1).padStart(5);
    console.log(`  ${ok ? "PASS" : "FAIL"} ${gate.id.padEnd(11)}${secs}s  ${gate.title}`);
    for (const line of details) console.log(`         ${line}`);
    for (const line of warnings) console.log(`         (warning) ${line}`);
    if (!ok) failed.push(gate.id);
  }
  console.log(failed.length ? `\nFAILED: ${failed.join(", ")}` : "\nAll gates passed.");

  // A pass covers the code as it was when the check started; anything edited meanwhile stays
  // dirty. `--only` runs a subset, so it never clears dirty groups.
  const state = readState();
  state.lastCheck = { at: new Date().toISOString(), mode, ok: failed.length === 0, failed };
  if (failed.length === 0 && !only) {
    state.dirty = changedGroups(before);
    state.stopBlocks = 0;
  }
  state.snapshot = fingerprints();
  writeState(state);
  return failed.length === 0;
}

process.exit(check() ? 0 : 1);
