// Rules for a pull request title. The PR is squash-merged and its title becomes the commit message
// on `develop` (CONTRIBUTING.md → Commit messages and PR titles), so it is checked before the merge.
//
//   PR_TITLE="Add a Sponsor button" node scripts/harness/check.mjs --only=pr-title

const MAX_LENGTH = 72;

/** Verb forms that are never imperative. "Changes" and nouns are left alone: they can start a valid title. */
const NOT_IMPERATIVE =
  /^(added|adds|adding|fixed|fixes|fixing|updated|updates|updating|removed|removes|removing|implemented|implements|implementing|created|creates|creating|improved|improves|improving|refactored|renamed|moved|bumped)$/i;

const VIETNAMESE = /[ăâđêôơưàáạảãầấậẩẫằắặẳẵèéẹẻẽềếệểễìíịỉĩòóọỏõồốộổỗờớợởỡùúụủũừứựửữỳýỵỷỹ]/i;

/** Problems with a title, as sentences a contributor can act on; empty when it is fine. */
export function titleProblems(title) {
  const text = (title ?? "").trim();
  if (!text) return ["the title is empty"];
  const problems = [];
  if (text.length > MAX_LENGTH) problems.push(`${text.length} characters; keep it under ${MAX_LENGTH}`);
  if (VIETNAMESE.test(text)) problems.push("write the title in English");
  if (!/^[A-Z]/.test(text)) problems.push("start with a capital letter");
  if (/^[\w-]+(\([^)]*\))?!?:/.test(text))
    problems.push("no `type:` prefix such as `feat:` or `fix:`; start with a verb");
  if (/^\[?(wip|draft)\b/i.test(text)) problems.push("no WIP or Draft in the title; open a draft pull request instead");
  if (/[.!?:;,]$/.test(text)) problems.push("no punctuation at the end");
  const first = /^[A-Za-z]+/.exec(text)?.[0] ?? "";
  if (NOT_IMPERATIVE.test(first)) problems.push(`start with an imperative verb ("Add", "Fix"), not "${first}"`);
  return problems;
}
