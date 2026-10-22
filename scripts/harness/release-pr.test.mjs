// Tests for the release pull request rules: node --test scripts/harness/release-pr.test.mjs

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT } from "./config.mjs";
import { RELEASE_SECTIONS, changelogProblems, releaseProblems } from "./release-pr.mjs";

const TEMPLATE = readFileSync(join(ROOT, ".github/PULL_REQUEST_TEMPLATE/release.md"), "utf8");

/** The template as a maintainer leaves it when every step is done. */
const filled = TEMPLATE.replace(/- \[ \]/g, "- [x]")
  .replace(/vX\.Y\.Z/g, "v1.2.0")
  .replace("prepared in #…", "prepared in #77")
  .replace("Updated in #…", "Updated in #78")
  .replace("Shipped: #…", "Shipped: #22");

test("release pr: the filled-in template passes", () => {
  assert.deepEqual(releaseProblems({ title: "Release v1.2.0", body: filled, version: "1.2.0" }), []);
});

test("release pr: the template has every section the rule looks for", () => {
  for (const heading of RELEASE_SECTIONS) assert.ok(TEMPLATE.split("\n").includes(heading), heading);
});

test("release pr: the untouched template fails on boxes and placeholders", () => {
  const problems = releaseProblems({ title: "Release v1.2.0", body: TEMPLATE, version: "1.2.0" });
  assert.ok(problems.some((p) => /^unticked: .*README/.test(p)));
  assert.ok(problems.some((p) => /placeholders/.test(p)));
});

test("release pr: each rule is reported", () => {
  const has = (input, pattern) =>
    assert.ok(
      releaseProblems({ title: "Release v1.2.0", body: filled, version: "1.2.0", ...input }).some((p) =>
        pattern.test(p),
      ),
      JSON.stringify(input).slice(0, 80),
    );
  has({ title: "Release v1.1.4" }, /title it "Release v1\.2\.0"/);
  has({ title: "Release 1.2.0" }, /title/);
  has({ body: "Merge develop" }, /release template/);
  has({ body: filled.replace("## Project", "## Board") }, /## Project/);
  has({ body: filled.replace("- [x]", "- [ ]") }, /unticked/);
  has({ body: filled.replace("#78", "#…") }, /placeholders/);
});

test("release pr: comments and CRLF line ends are not content", () => {
  const body = filled.replace(/\n/g, "\r\n") + "\r\n<!-- - [ ] vX.Y.Z #… -->\r\n";
  assert.deepEqual(releaseProblems({ title: "Release v1.2.0", body, version: "1.2.0" }), []);
  const unclosed = filled + "\n<!-- - [ ] vX.Y.Z\n- [ ] hidden too\n";
  assert.deepEqual(releaseProblems({ title: "Release v1.2.0", body: unclosed, version: "1.2.0" }), []);
});

test("changelog: every version in CHANGELOG.md is written as its tag", () => {
  assert.deepEqual(changelogProblems(readFileSync(join(ROOT, "CHANGELOG.md"), "utf8")), []);
});

test("changelog: a version without its v is reported", () => {
  const text = "## [Unreleased]\n\n## [1.2.0] - 2026-10-22\n\n[Unreleased]: a\n[1.2.0]: b\n[v1.1.4]: c\n";
  assert.deepEqual(
    changelogProblems(text).map((p) => p.split(":")[1]),
    ["3", "6"],
  );
});

test("release pr: the changelog must have the release's section", () => {
  const input = { title: "Release v1.2.0", body: filled, version: "1.2.0" };
  assert.deepEqual(releaseProblems({ ...input, changelog: "## [v1.2.0] - 2026-10-22\n" }), []);
  assert.ok(
    releaseProblems({ ...input, changelog: "## [1.2.0] - 2026-10-22\n" }).some((p) => /no "## \[v1\.2\.0\]/.test(p)),
  );
});
