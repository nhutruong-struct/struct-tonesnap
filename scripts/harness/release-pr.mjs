// Rules for a release pull request (`develop` → `master`). Merging it publishes the release, so everything the
// release touches outside the app (README, website, issues, the project board, Discussions) is settled first and
// ticked in the PR body, from .github/PULL_REQUEST_TEMPLATE/release.md (docs/MAINTAINING.md → Releasing).
//
//   PR_BASE=master PR_TITLE="Release v1.2.0" PR_BODY="…" node scripts/harness/check.mjs --only=release-pr

/** Headings of the release template; their presence shows the template was used. */
export const RELEASE_SECTIONS = ["## Version", "## README", "## Website", "## Issues", "## Project", "## Announcement"];

/** Placeholders of the template that must be replaced: the version and issue or PR numbers. */
const PLACEHOLDER = /vX\.Y\.Z|#…/;

/** Problems with a release PR, as sentences the maintainer can act on; empty when it is ready to merge. */
export function releaseProblems({ title, body, version }) {
  const problems = [];
  const expected = `Release v${version}`;
  if ((title ?? "").trim() !== expected) problems.push(`title it "${expected}" (the version in tauri.conf.json)`);
  // The template's comments are guidance, not content.
  const lines = (body ?? "")
    .replace(/\r\n/g, "\n")
    .replace(/<!--[\s\S]*?-->/g, "")
    .split("\n");
  const missing = RELEASE_SECTIONS.filter((heading) => !lines.some((line) => line.trim() === heading));
  if (missing.length > 0)
    problems.push(
      `the body lacks ${missing.join(", ")}: open the PR with the release template (gh pr create --template release.md)`,
    );
  for (const line of lines) {
    const box = /^\s*[-*] \[ \] (.*)$/.exec(line);
    if (box) problems.push(`unticked: ${box[1].trim()}`);
  }
  if (lines.some((line) => PLACEHOLDER.test(line)))
    problems.push("replace the template's placeholders (vX.Y.Z, #…) with the version and the issue or PR numbers");
  return problems;
}
