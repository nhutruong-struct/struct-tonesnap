// Rules for a release pull request (`develop` → `master`). Merging it publishes the release, so everything the
// release touches outside the app (README, website, issues, the project board, Discussions) is settled first and
// ticked in the PR body, from .github/PULL_REQUEST_TEMPLATE/release.md (docs/MAINTAINING.md → Releasing).
//
//   PR_BASE=master PR_TITLE="Release v1.2.0" PR_BODY="…" node scripts/harness/check.mjs --only=release-pr

/** Headings of the release template; their presence shows the template was used. */
export const RELEASE_SECTIONS = ["## Version", "## README", "## Website", "## Issues", "## Project", "## Announcement"];

/** Placeholders of the template that must be replaced: the version and issue or PR numbers. */
const PLACEHOLDER = /vX\.Y\.Z|#…/;

/**
 * Problems with the version headings and links of CHANGELOG.md. A version is written as its tag (`v1.2.0`), the
 * same in the release PR title, the GitHub release and the announcement; release.yml finds the notes by it.
 */
export function changelogProblems(text) {
  const problems = [];
  (text ?? "").split(/\r?\n/).forEach((line, i) => {
    const heading = /^## \[([^\]]+)\]/.exec(line);
    if (heading && heading[1] !== "Unreleased" && !/^v\d+\.\d+\.\d+$/.test(heading[1]))
      problems.push(`CHANGELOG.md:${i + 1}: write the version as its tag, "## [v${heading[1].replace(/^v/, "")}]"`);
    const link = /^\[([^\]]+)\]: /.exec(line);
    if (link && link[1] !== "Unreleased" && /^\d/.test(link[1]))
      problems.push(`CHANGELOG.md:${i + 1}: name the link "[v${link[1]}]"`);
  });
  return problems;
}

/** `text` without its HTML comments; an unclosed one hides the rest, as GitHub renders it. */
function withoutComments(text) {
  let kept = "";
  let from = 0;
  for (;;) {
    const start = text.indexOf("<!--", from);
    if (start < 0) return kept + text.slice(from);
    kept += text.slice(from, start);
    const end = text.indexOf("-->", start + 4);
    if (end < 0) return kept;
    from = end + 3;
  }
}

/** Problems with a release PR, as sentences the maintainer can act on; empty when it is ready to merge. */
export function releaseProblems({ title, body, version, changelog }) {
  const problems = [];
  if (changelog !== undefined) {
    problems.push(...changelogProblems(changelog));
    if (!changelog.split(/\r?\n/).some((line) => line.startsWith(`## [v${version}] - `)))
      problems.push(`CHANGELOG.md has no "## [v${version}] - date" section`);
  }
  const expected = `Release v${version}`;
  if ((title ?? "").trim() !== expected) problems.push(`title it "${expected}" (the version in tauri.conf.json)`);
  // The template's comments are guidance, not content.
  const lines = withoutComments((body ?? "").replace(/\r\n/g, "\n")).split("\n");
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
