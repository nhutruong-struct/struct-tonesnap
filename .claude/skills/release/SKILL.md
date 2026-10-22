---
name: release
description: Prepare and open ToneSnap's release pull request from develop into master, bringing the README, the website, issues, the Roadmap project and the Discussions announcement up to date with the release. Use when the user asks to release, ship a version, or open the develop → master PR (in any language).
argument-hint: "[version, e.g. 1.2.0]"
---

# Release

The steps of [docs/MAINTAINING.md → Releasing](../../../docs/MAINTAINING.md#releasing), done with `gh`. Merging the
release PR publishes the release, and CI's `check` fails on that PR until its body, from
`.github/PULL_REQUEST_TEMPLATE/release.md`, has every box ticked and no placeholder left (`release-pr` gate in
`scripts/harness/release-pr.mjs`). Repository: `nhutruong-struct/struct-tonesnap`.

Everything that leaves this machine needs the user's yes, one action at a time: pushing, opening a PR, closing or
commenting on an issue, editing the project, posting a discussion. Show the exact text first. Never merge.

1. **Version.** Read the version in `src-tauri/tauri.conf.json` on `origin/develop` and the `Unreleased` section of
   `CHANGELOG.md`. If `$ARGUMENTS` names a newer version, or `Unreleased` is not empty, prepare it on a
   `chore/release-x.y.z` branch from `origin/develop`: bump `package.json`, `src-tauri/Cargo.toml` and
   `src-tauri/tauri.conf.json`, run `cargo update --manifest-path src-tauri/Cargo.toml -p tonesnap --offline`, move
   `Unreleased` into a `## [x.y.z] - date` section with the **Install** notes of the previous release (new file
   names, new platforms), check it against `git log vPREV..origin/develop` so no user-visible PR is missing, add the
   compare links at the end, and update `docs/PLAN.md`. Run `node scripts/harness/check.mjs --full`, commit, and
   offer a PR into `develop`. The release PR waits until it is merged.
2. **README.** Read `README.md` against the new `CHANGELOG.md` section: features, supported platforms, install,
   update and uninstall steps, file names, settings, the intro poster. Fix what no longer matches.
3. **Website.** Read `site/index.html` against the same section (see MAINTAINING.md for the list, including the
   intro video in `site/media/`). README and site fixes go in one `docs/release-x.y.z` PR into `develop`, merged
   before the release PR; the site deploys from `develop`. Note its number, or why no change was needed.
4. **Issues.** List what the release resolves: the `Closes #n` of each PR in `git log vPREV..origin/develop`, and
   open issues the changelog section answers (`gh issue list --state open`). For each, draft a comment naming the
   release ("Shipped in vx.y.z: …", one line on what changed). Already closed issues get the comment only; open ones
   are closed with it (`gh issue close n --comment …`); partly done ones get a comment on what is left and stay
   open. Read `.github/ISSUE_TEMPLATE/*.yml`: platform and version choices still right?
5. **Project.** On the **ToneSnap Roadmap** project (`gh project item-list 2 --owner nhutruong-struct`), set the
   shipped issues' **Status** to **Done** (`gh project field-list 2 --owner nhutruong-struct` gives the field and
   option ids for `gh project item-edit`), and look for **Done** items whose issue is still open.
6. **Announcement.** Draft the Discussions post, titled "ToneSnap x.y.z is released" like the earlier ones in
   Announcements: two or three sentences on what is new, the download link
   (`https://github.com/nhutruong-struct/struct-tonesnap/releases/tag/vx.y.z`) and who updates by themselves. It goes
   into the PR body; it is posted only after the release is published (step 8).
7. **Release PR.** With the user's yes, open it from `develop` into `master`:
   `gh pr create --base master --head develop --title "Release vx.y.z" --template release.md`, then fill the body
   (`gh pr edit --body-file`) with the version PR, the site PR, the shipped issues and the draft, ticking only the
   boxes whose step is really done. Report the PR link and what the CI `check` says; the user merges it with
   **Create a merge commit**.
8. **After the merge.** Once the **Release** workflow has published `vx.y.z`, and with the user's yes, post the
   announcement in the **Announcements** category (`gh api graphql` with `createDiscussion`, the repository id and
   the category id from `repository { id discussionCategories }`), and give the user its link.
