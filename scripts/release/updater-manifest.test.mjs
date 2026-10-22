// Tests for the release manifest the in-app updater reads. Run by the `harness` gate of check.mjs.

import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { signedInstaller, updaterManifest } from "./updater-manifest.mjs";

const NSIS = "ToneSnap_1.1.0_x64-setup.exe";
const MSI = "ToneSnap_1.1.0_x64_en-US.msi";
const MAC = "ToneSnap_1.1.0_aarch64.app.tar.gz";

test("the manifest points every key at the installer or archive of its kind", () => {
  const manifest = updaterManifest({
    version: "1.1.0",
    repo: "owner/repo",
    notes: "Fixes",
    pubDate: "2026-11-01T00:00:00.000Z",
    nsis: { file: NSIS, signature: "nsis-sig" },
    msi: { file: MSI, signature: "msi-sig" },
    mac: { file: MAC, signature: "mac-sig" },
  });
  const mac = { signature: "mac-sig", url: `https://github.com/owner/repo/releases/download/v1.1.0/${MAC}` };
  const nsis = { signature: "nsis-sig", url: `https://github.com/owner/repo/releases/download/v1.1.0/${NSIS}` };
  assert.deepEqual(manifest, {
    version: "1.1.0",
    notes: "Fixes",
    pub_date: "2026-11-01T00:00:00.000Z",
    platforms: {
      "windows-x86_64": nsis,
      "windows-x86_64-nsis": nsis,
      "windows-x86_64-msi": {
        signature: "msi-sig",
        url: `https://github.com/owner/repo/releases/download/v1.1.0/${MSI}`,
      },
      "darwin-aarch64": mac,
      "darwin-aarch64-app": mac,
    },
  });
});

test("an installer is found with its signature, and a missing one is an error", () => {
  const dir = mkdtempSync(join(tmpdir(), "tonesnap-manifest-"));
  try {
    writeFileSync(join(dir, NSIS), "");
    writeFileSync(join(dir, `${NSIS}.sig`), "nsis-sig\n");
    writeFileSync(join(dir, MSI), "");
    const files = [NSIS, `${NSIS}.sig`, MSI];
    assert.deepEqual(signedInstaller(dir, files, "1.1.0", "-setup.exe"), { file: NSIS, signature: "nsis-sig" });
    assert.throws(() => signedInstaller(dir, files, "1.1.0", ".msi"), /has no \.sig file/);
    assert.throws(() => signedInstaller(dir, files, "1.2.0", "-setup.exe"), /found 0/);
    assert.throws(() => signedInstaller(dir, files, "1.1.0", ".app.tar.gz"), /found 0/);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});
