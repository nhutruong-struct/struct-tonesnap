// Writes latest.json, the manifest the in-app updater reads (src-tauri/src/update.rs), for a release
// built by .github/workflows/release.yml. Usage:
//   node scripts/release/updater-manifest.mjs <version> <release dir> <notes file> <owner/repo>
// The release dir holds the installers, the macOS update archive and the `.sig` file of each.

import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

/**
 * The manifest for one release. The updater looks for `windows-x86_64-<installer>` first, so an MSI
 * install stays an MSI install; `windows-x86_64` is the fallback for anything else. macOS (Apple
 * Silicon) updates from the signed `.app.tar.gz`; `darwin-aarch64-app` is the key for that bundle type.
 */
export function updaterManifest({ version, repo, notes, pubDate, nsis, msi, mac }) {
  const asset = (file) => `https://github.com/${repo}/releases/download/v${version}/${encodeURIComponent(file)}`;
  const nsisEntry = { signature: nsis.signature, url: asset(nsis.file) };
  const macEntry = { signature: mac.signature, url: asset(mac.file) };
  return {
    version,
    notes,
    pub_date: pubDate,
    platforms: {
      "windows-x86_64": nsisEntry,
      "windows-x86_64-nsis": nsisEntry,
      "windows-x86_64-msi": { signature: msi.signature, url: asset(msi.file) },
      "darwin-aarch64": macEntry,
      "darwin-aarch64-app": macEntry,
    },
  };
}

/** The one installer in `files` whose name ends with `suffix`, and its signature. */
export function signedInstaller(dir, files, version, suffix) {
  const matches = files.filter((f) => f.endsWith(suffix) && f.includes(`_v${version}_`));
  if (matches.length !== 1) {
    throw new Error(`expected one ${suffix} for ${version} in ${dir}, found ${matches.length}`);
  }
  const file = matches[0];
  if (!files.includes(`${file}.sig`)) throw new Error(`${file} has no .sig file; is the signing key set?`);
  return { file, signature: readFileSync(join(dir, `${file}.sig`), "utf8").trim() };
}

function main([version, dir, notesFile, repo]) {
  if (!version || !dir || !notesFile || !repo) {
    throw new Error("usage: updater-manifest.mjs <version> <release dir> <notes file> <owner/repo>");
  }
  const files = readdirSync(dir);
  const manifest = updaterManifest({
    version,
    repo,
    notes: readFileSync(notesFile, "utf8").trim(),
    pubDate: new Date().toISOString(),
    nsis: signedInstaller(dir, files, version, "-setup.exe"),
    msi: signedInstaller(dir, files, version, ".msi"),
    mac: signedInstaller(dir, files, version, ".app.tar.gz"),
  });
  writeFileSync(join(dir, "latest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main(process.argv.slice(2));
}
