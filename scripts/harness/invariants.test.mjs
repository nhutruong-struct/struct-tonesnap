// Tests for the per-file project rules and the pull request title rules.
// Run by the `harness` gate of check.mjs, or directly: node --test scripts/harness
//
// Samples that would break the rules themselves (comment markers, credentials, Vietnamese, trailing
// blanks) are assembled from pieces, so this file passes the checks it tests.

import assert from "node:assert/strict";
import { test } from "node:test";
import { fileViolations, fixWhitespace } from "./invariants.mjs";
import { titleProblems } from "./pr-title.mjs";

const SLASHES = "/" + "/";
const BLOCK = "/" + "*";
const HTML = "<" + "!--";
const HASH = "#";
const SPACE = " ";
// A Vietnamese greeting; the accented letters come from their code points.
const VIETNAMESE = `Xin ch${String.fromCharCode(0xe0)}o c${String.fromCharCode(0xe1)}c b${String.fromCharCode(0x1ea1)}n`;
const rules = (file, text, only) => fileViolations(file, text, only).map((v) => `${v.rule}:${v.line}:${v.message}`);

test("whitespace: a clean file passes", () => {
  assert.deepEqual(rules("docs/a.md", "# Title\n\ntext\n", ["whitespace"]), []);
});

test("whitespace: trailing blanks are reported with their line, except in Markdown", () => {
  const text = `first${SPACE}\nsecond\n`;
  assert.deepEqual(rules("scripts/a.mjs", text, ["whitespace"]), ["whitespace:1:trailing whitespace"]);
  assert.deepEqual(rules("docs/a.md", text, ["whitespace"]), []);
});

test("whitespace: CRLF, a missing final newline and extra final newlines", () => {
  assert.match(rules("a.toml", "a = 1\r\nb = 2\r\n", ["whitespace"])[0], /^whitespace:1:CR in a line ending/);
  assert.deepEqual(rules("a.json", "{}", ["whitespace"]), ["whitespace:1:no newline at the end of the file"]);
  assert.match(rules("a.json", "{}\n\n", ["whitespace"])[0], /more than one newline/);
});

test("whitespace: tab indentation is reported outside Rust", () => {
  assert.deepEqual(rules("a.ps1", "if ($x) {\n\tfoo\n}\n", ["whitespace"]), [
    "whitespace:2:tab indentation; indent with spaces",
  ]);
  assert.deepEqual(rules("src-tauri/src/a.rs", "fn a() {\n\tb();\n}\n", ["whitespace"]), []);
});

test("whitespace: lockfiles and recorded model output are exempt", () => {
  assert.deepEqual(rules("pnpm-lock.yaml", `a${SPACE}\r\n`, ["whitespace"]), []);
  assert.deepEqual(rules("docs/bench/run.md", `a${SPACE}\r\n`, ["whitespace"]), []);
});

test("fixWhitespace repairs what the whitespace rule reports", () => {
  const messy = `a${SPACE}\r\nb\t\r\n\r\n\r\n`;
  const fixed = fixWhitespace("scripts/a.mjs", messy);
  assert.equal(fixed, "a\nb\n");
  assert.deepEqual(rules("scripts/a.mjs", fixed, ["whitespace"]), []);
  assert.equal(fixWhitespace("a.json", "{}"), "{}\n");
  assert.equal(fixWhitespace("docs/a.md", `line${SPACE}${SPACE}\nnext\n`), `line${SPACE}${SPACE}\nnext\n`);
});

test("suppress: eslint-disable needs a reason", () => {
  const bare = `${SLASHES} eslint-disable-next-line no-console\nconsole.log(1);\n`;
  assert.equal(rules("src/a.ts", bare, ["suppress"]).length, 1);
  const block = `${BLOCK} eslint-disable no-console */\n`;
  assert.equal(rules("scripts/a.mjs", block, ["suppress"]).length, 1);
  const reasoned = `${SLASHES} eslint-disable-next-line no-console -- the bench prints its table\n`;
  assert.deepEqual(rules("src/a.ts", reasoned, ["suppress"]), []);
});

test("suppress: inline ESLint rule settings, prettier-ignore and rustfmt::skip are rejected", () => {
  assert.equal(rules("src/a.ts", `${BLOCK} eslint no-console: off */\n`, ["suppress"]).length, 1);
  assert.equal(rules("src/a.ts", `${SLASHES} prettier-ignore\nconst a = [1,2];\n`, ["suppress"]).length, 1);
  assert.equal(rules("site/a.html", `${HTML} prettier-ignore -->\n`, ["suppress"]).length, 1);
  assert.equal(rules(".github/a.yml", `${HASH} prettier-ignore\n`, ["suppress"]).length, 1);
  assert.equal(rules("src-tauri/src/a.rs", `#[rustfmt::skip]\nfn a() {}\n`, ["suppress"]).length, 1);
});

test("suppress: mentions in prose do not count", () => {
  assert.deepEqual(rules("docs/a.md", "Use `prettier-ignore` never.\n", ["suppress"]), []);
  assert.deepEqual(rules("src/a.ts", `const note = "eslint-disable is banned";\n`, ["suppress"]), []);
});

test("secrets: a real-looking key is rejected in any file, lockfiles aside", () => {
  const key = "sk-ant-" + "api03-" + "a".repeat(30);
  assert.equal(rules(".github/a.yml", `key: ${key}\n`, ["secrets"]).length, 1);
  assert.deepEqual(rules("pnpm-lock.yaml", `key: ${key}\n`, ["secrets"]), []);
});

test("code: `any` is rejected in code but not in the prose of a block comment", () => {
  const anyType = ": " + "any";
  assert.equal(rules("src/a.ts", `let x${anyType} = 1;\n`, ["code"]).length, 1);
  const doc = `${BLOCK}*\n * Accepts it${anyType} case works.\n */\nexport const x = 1;\n`;
  assert.deepEqual(rules("src/a.ts", doc, ["code"]), []);
});

test("language: Vietnamese prose is rejected in any file outside the allowlist", () => {
  assert.equal(rules("CONTRIBUTING.md", `${VIETNAMESE}\n`, ["language"]).length, 1);
  assert.equal(rules(".github/a.yml", `name: ${VIETNAMESE}\n`, ["language"]).length, 1);
  assert.deepEqual(rules("src/lib/i18n.ts", `a: "${VIETNAMESE}",\n`, ["language"]), []);
});

test("pr title: good titles pass", () => {
  for (const title of [
    "Add a Sponsor button to the website header",
    "Restore the clipboard after the OCR fallback",
    "Bump the rust-minor-and-patch group across 1 directory with 6 updates",
    "Fix `unwrap` in capture.rs",
  ]) {
    assert.deepEqual(titleProblems(title), [], title);
  }
});

test("pr title: each rule is reported", () => {
  const has = (title, pattern) =>
    assert.ok(
      titleProblems(title).some((p) => pattern.test(p)),
      title,
    );
  has("", /empty/);
  has("a".repeat(80), /characters/);
  has("fix the popup", /capital/);
  has("feat: add a button", /prefix/);
  has("Fix(popup): scroll", /prefix/);
  has("WIP Add a button", /WIP/);
  has("Add a button.", /punctuation/);
  has("Added a button", /imperative/);
  has("Fixes the popup", /imperative/);
  has(`Th${String.fromCharCode(0xea)}m n${String.fromCharCode(0xfa)}t`, /English/);
});

test("network: the updater endpoints in the Tauri config are checked too", () => {
  const config = (url) => JSON.stringify({ plugins: { updater: { endpoints: [url] } } });
  assert.deepEqual(
    rules("src-tauri/tauri.conf.json", config("https://github.com/o/r/releases/latest/download/latest.json"), [
      "network",
    ]),
    [],
  );
  assert.equal(
    rules("src-tauri/tauri.conf.json", config("https://updates.example.com/latest.json"), ["network"]).length,
    1,
  );
});
