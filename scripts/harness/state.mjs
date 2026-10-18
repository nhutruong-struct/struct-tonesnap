// Fingerprints of the source groups and the small JSON file the hooks share between calls.

import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { GROUPS, IGNORED_DIRS, ROOT, STATE_FILE } from "./config.mjs";

/** Every file under `entries` (files or folders, relative to ROOT), as forward-slash paths. */
export function listFiles(entries) {
  const out = [];
  const walk = (abs) => {
    let stat;
    try {
      stat = statSync(abs);
    } catch {
      return;
    }
    if (stat.isDirectory()) {
      for (const name of readdirSync(abs)) {
        if (!IGNORED_DIRS.has(name)) walk(join(abs, name));
      }
    } else {
      out.push(relative(ROOT, abs).replaceAll("\\", "/"));
    }
  };
  for (const entry of entries) walk(join(ROOT, entry));
  return out.sort();
}

/** Hash of path, size and mtime for every file in each group. Cheap enough to run per hook. */
export function fingerprints() {
  const result = {};
  for (const [group, entries] of Object.entries(GROUPS)) {
    const hash = createHash("sha1");
    for (const file of listFiles(entries)) {
      const stat = statSync(join(ROOT, file));
      hash.update(`${file}|${stat.size}|${Math.round(stat.mtimeMs)}\n`);
    }
    result[group] = hash.digest("hex");
  }
  return result;
}

/** Groups whose fingerprint differs between two snapshots. */
export function changedGroups(before = {}, after = fingerprints()) {
  return Object.keys(GROUPS).filter((g) => before[g] !== after[g]);
}

const EMPTY_STATE = {
  /** Fingerprints after the last tool call, used to attribute changes to the agent. */
  snapshot: null,
  /** Groups the agent changed since the last passing check. */
  dirty: [],
  /** Consecutive Stop-hook blocks, to avoid an endless fix loop. */
  stopBlocks: 0,
  lastCheck: null,
};

export function readState() {
  try {
    return { ...EMPTY_STATE, ...JSON.parse(readFileSync(STATE_FILE, "utf8")) };
  } catch {
    return { ...EMPTY_STATE };
  }
}

export function writeState(state) {
  if (!existsSync(dirname(STATE_FILE))) mkdirSync(dirname(STATE_FILE), { recursive: true });
  writeFileSync(STATE_FILE, JSON.stringify(state, null, 2));
}
