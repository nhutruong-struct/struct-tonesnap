<!-- Release pull request, `develop` → `master`, titled "Release vX.Y.Z". Merging it publishes the release
     (.github/workflows/release.yml), so CI's `check` fails while the title does not match the version, a box is
     unticked or a placeholder (vX.Y.Z, #…) is left. Steps: docs/MAINTAINING.md → Releasing, or /release. -->

## Version

vX.Y.Z, prepared in #….

- [ ] `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` and `Cargo.lock` carry the version, and
      `CHANGELOG.md` has its section, with the install notes, and an empty `Unreleased`.
- [ ] `docs/PLAN.md` and `docs/specs/` describe what ships.

## README

- [ ] `README.md` matches the release: features, supported platforms, install, update and uninstall steps, file
      names, settings, and the intro poster or video.

## Website

Updated in #… <!-- or: "No change needed: …" -->

- [ ] `site/` matches the `CHANGELOG.md` section (features, tones, languages, the mock popup, default settings,
      download buttons, install and uninstall steps, the intro video), and that PR is merged into `develop`.

## Issues

Shipped: #… <!-- every issue this release resolves, or "none" -->

- [ ] Each shipped issue is closed with a comment naming the release; partly done ones say what is left.
- [ ] The issue forms (`.github/ISSUE_TEMPLATE/`) still offer the right platforms, versions and settings.

## Project

- [ ] On the **ToneSnap Roadmap** project, the shipped issues are **Done**, and no **Done** item is still open.

## Announcement

- [ ] The post for Discussions → Announcements ("ToneSnap X.Y.Z is released") is drafted below; it goes up once
      the release is published.

<details><summary>Draft</summary>

</details>
