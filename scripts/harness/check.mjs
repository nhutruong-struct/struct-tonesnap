// Quality gates for ToneSnap. Exit code 0 means every gate that ran passed.
//
//   node scripts/harness/check.mjs                 fast: invariants, rustfmt, tsc, clippy, unit tests
//   node scripts/harness/check.mjs --full          + production frontend build, Win32 integration tests
//   node scripts/harness/check.mjs --only=input    key-press tests; only while nobody uses the PC
//   node scripts/harness/check.mjs --changed       only gates for groups changed since the last pass
//   node scripts/harness/check.mjs --only=tsc,test pick gates by id
//   --fix                                          run cargo fmt, eslint --fix, prettier --write and the whitespace
//                                                  fixes first (what most style failures need)
//   PR_TITLE="…" node scripts/harness/check.mjs --only=pr-title    check a pull request title (CI does this)
//   PR_BASE=master PR_TITLE="…" PR_BODY="…" … --only=release-pr    check a release pull request (CI does this)
//
// Results are recorded in .claude/.state/harness.json for the hooks.

import { spawnSync } from "node:child_process";
import { appendFileSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { GATES_FOR_GROUP, MANIFEST, ROOT, toolEnv } from "./config.mjs";
import { fixWhitespace, runInvariants, scannedFiles } from "./invariants.mjs";
import { titleProblems } from "./pr-title.mjs";
import { releaseProblems } from "./release-pr.mjs";
import { changedGroups, fingerprints, readState, writeState } from "./state.mjs";

const PRETTIER = join(ROOT, "node_modules/prettier/bin/prettier.cjs");
const ESLINT = join(ROOT, "node_modules/eslint/bin/eslint.js");
// pnpm through the corepack that ships with Node, so terminals without npm's folder on PATH
// (those hosted by MSIX apps) still find it. Windows keeps node_modules next to node.exe; macOS and
// Linux keep it in lib/ beside bin/.
const PNPM = join(
  dirname(process.execPath),
  process.platform === "win32" ? "" : "../lib",
  "node_modules/corepack/dist/pnpm.js",
);

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
      /error|warning|\[warn\]|FAILED|\bFAIL\b|×|✖|panicked|Diff in|failures:|test result|TS\d{4}/i.test(l) ||
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

const ESLINT_TARGETS = ["src", "scripts", "eslint.config.js"];

/**
 * The tests of the scripts (the harness and the release scripts), listed one by one because `node --test`
 * does not expand globs on every Node version.
 */
function harnessTests() {
  return ["harness", "release"].flatMap((folder) => {
    const dir = join(ROOT, "scripts", folder);
    return readdirSync(dir)
      .filter((f) => f.endsWith(".test.mjs"))
      .map((f) => join(dir, f));
  });
}

const GATES = [
  {
    id: "invariants",
    title: "project rules (contracts, code rules, language, privacy, secrets, whitespace)",
    hint: "Each line is file:line and the rule that failed; AGENTS.md → Invariants explains why. Whitespace problems fix themselves with --fix; the rest need a code change.",
    fixable: true,
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
    hint: "Run node scripts/harness/check.mjs --fix, then commit the formatted files.",
    fixable: true,
    run: tool("cargo", ["fmt", "--manifest-path", MANIFEST, "--check"]),
  },
  {
    id: "prettier",
    title: "prettier --check (TS, CSS, JSON, YAML, HTML, scripts)",
    hint: "Run node scripts/harness/check.mjs --fix, then commit the formatted files.",
    fixable: true,
    run: tool(node, [PRETTIER, "--check", "--log-level", "warn", "."]),
  },
  {
    id: "tsc",
    title: "TypeScript (tsc)",
    hint: "A type error: fix the code (no any, @ts-ignore or casts that hide it).",
    run: tool(node, [join(ROOT, "node_modules/typescript/bin/tsc"), "-p", ROOT]),
  },
  {
    id: "eslint",
    title: "ESLint (type-aware, React hooks; Node rules for scripts/)",
    hint: "node scripts/harness/check.mjs --fix applies the automatic fixes; the rest need a code change. The rule name is at the end of each line.",
    fixable: true,
    run: tool(node, [ESLINT, "--max-warnings", "0", ...ESLINT_TARGETS]),
  },
  {
    id: "clippy",
    title: "cargo clippy -D warnings",
    hint: "Read the lint's help text; cargo clippy --fix can apply some suggestions. Do not silence a lint without a reason (.claude/rules/rust.md).",
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
    hint: "A Rust test fails: run cargo test --lib <name> in src-tauri for the full output.",
    run: tool("cargo", ["test", "--manifest-path", MANIFEST, "--lib"], { requireTests: true }),
  },
  {
    id: "vitest",
    title: "frontend tests (vitest)",
    hint: "A UI test fails: pnpm test <file> shows the full output.",
    run: tool(node, [join(ROOT, "node_modules/vitest/vitest.mjs"), "run"]),
  },
  {
    id: "harness",
    title: "harness tests (project rules, PR title and release PR rules, hook paths)",
    hint: "A test of scripts/harness/ fails: node --test scripts/harness shows the full output.",
    run: tool(node, ["--test", ...harnessTests()]),
  },
  {
    // Needs the title from the pull request event, so it runs only when asked for by name (CI does).
    id: "pr-title",
    title: "pull request title (becomes the squashed commit message)",
    hint: 'Edit the title on GitHub: English, imperative, under 72 characters, e.g. "Restore the clipboard after the OCR fallback".',
    explicit: true,
    run: () => {
      const title = process.env.PR_TITLE;
      // Dependabot titles are generated ("Bump x from 1 to 2 in /src-tauri") and often longer.
      if (title === undefined || process.env.PR_AUTHOR === "dependabot[bot]") return { ok: true };
      const problems = titleProblems(title);
      return { ok: problems.length === 0, details: problems.map((p) => `"${title}": ${p}`) };
    },
  },
  {
    // Merging into master publishes a release, so its PR must show the README, website, issues, project board and
    // announcement were handled. Needs the pull request event, so it runs only when asked for by name (CI does).
    id: "release-pr",
    title: "release pull request (develop → master) is complete",
    hint: "Edit the PR on GitHub: use .github/PULL_REQUEST_TEMPLATE/release.md, do each step and tick its box (docs/MAINTAINING.md → Releasing).",
    explicit: true,
    run: () => {
      if (process.env.PR_BASE !== "master") return { ok: true };
      const { version } = JSON.parse(readFileSync(join(ROOT, "src-tauri/tauri.conf.json"), "utf8"));
      const problems = releaseProblems({ title: process.env.PR_TITLE, body: process.env.PR_BODY, version });
      return { ok: problems.length === 0, details: problems };
    },
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

/** Rewrites what can be fixed mechanically: Rust formatting, ESLint autofixes, Prettier, whitespace. */
function fix() {
  const quiet = { cwd: ROOT, env, stdio: "ignore" };
  const cargo = spawnSync("cargo", ["fmt", "--manifest-path", MANIFEST], quiet);
  if (cargo.error) console.log("  (cargo not found: skipped cargo fmt; install Rust from https://rustup.rs)");
  spawnSync(node, [ESLINT, "--fix", ...ESLINT_TARGETS], quiet);
  let whitespace = 0;
  for (const file of scannedFiles()) {
    const path = join(ROOT, file);
    const text = readFileSync(path, "utf8");
    const fixed = fixWhitespace(file, text);
    if (fixed !== text) {
      writeFileSync(path, fixed);
      whitespace++;
    }
  }
  spawnSync(node, [PRETTIER, "--write", "--log-level", "warn", "."], quiet);
  console.log(`Fixed formatting (rustfmt, ESLint, Prettier) and whitespace in ${whitespace} file(s).\n`);
}

/** GitHub Actions turns `::error` lines into annotations on the pull request, so a newcomer sees them in place. */
function annotate(gate, details) {
  if (process.env.GITHUB_ACTIONS !== "true") return;
  const encode = (text) => text.replaceAll("%", "%25").replaceAll("\r", "%0D").replaceAll("\n", "%0A");
  let located = 0;
  for (const line of details) {
    const m = /^\[(\w+)\] (\S+?):(\d+) (.*)$/.exec(line);
    if (!m) continue;
    located++;
    console.log(`::error file=${m[2]},line=${m[3]},title=${encode(`Project rule: ${m[1]}`)}::${encode(m[4])}`);
  }
  if (located < details.length || details.length === 0) {
    console.log(`::error title=${encode(`${gate.id} failed`)}::${encode(gate.hint ?? gate.title)}`);
  }
}

/** The closing advice, also written to the job summary on GitHub. */
function advice(failedGates) {
  const lines = failedGates.map((g) => `- **${g.id}**: ${g.hint ?? g.title}`);
  const closing = [];
  if (failedGates.some((g) => g.fixable)) {
    closing.push("Most style failures fix themselves: node scripts/harness/check.mjs --fix, then commit.");
  }
  closing.push("See CONTRIBUTING.md → Style and formatting.");
  if (process.env.GITHUB_STEP_SUMMARY) {
    appendFileSync(process.env.GITHUB_STEP_SUMMARY, `### Checks failed\n\n${[...lines, "", ...closing].join("\n")}\n`);
  }
  return closing;
}

export function check() {
  if (args.has("--fix")) fix();

  const gates = selectGates();
  const mode = only ? `only ${only}` : args.has("--changed") ? "changed" : args.has("--full") ? "full" : "fast";
  console.log(`ToneSnap check (${mode})`);
  const failed = [];
  const failedGates = [];
  const before = fingerprints();
  for (const gate of gates) {
    const started = Date.now();
    const { ok, details = [], warnings = [] } = gate.run();
    const secs = ((Date.now() - started) / 1000).toFixed(1).padStart(5);
    console.log(`  ${ok ? "PASS" : "FAIL"} ${gate.id.padEnd(11)}${secs}s  ${gate.title}`);
    for (const line of details) console.log(`         ${line}`);
    for (const line of warnings) console.log(`         (warning) ${line}`);
    if (!ok) {
      if (gate.hint) console.log(`         → ${gate.hint}`);
      annotate(gate, details);
      failed.push(gate.id);
      failedGates.push(gate);
    }
  }
  console.log(failed.length ? `\nFAILED: ${failed.join(", ")}` : "\nAll gates passed.");
  for (const line of failed.length ? advice(failedGates) : []) console.log(line);

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
