// Project rules that compilers cannot see.
//
// Per-file rules (language, privacy, code, network, secrets, whitespace, suppress) judge one file at a time, so the
// hooks can also run them on a single edit. Cross-file rules (tones, shapes, enums, commands,
// events, capabilities, setup) check the contracts between the Rust core and the React UI.
// Each rule yields errors (fail the check) and warnings (reported only).
//
// Usage: node scripts/harness/invariants.mjs

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import {
  ALLOWED_HOSTS,
  BROWSER_LINK_HOSTS,
  CONTENT_IDENTIFIERS,
  ENUMS,
  GENERATED,
  INVOKE_FILE,
  MAX_FILE_LINES,
  RECORDED,
  REQUIRED_PERMISSIONS,
  ROOT,
  SECRET_PATTERNS,
  SHAPES,
  TEXT_EXTENSIONS,
  TEXT_NAMES,
  UNSAFE_ALLOWED,
  VIETNAMESE_ALLOWED,
} from "./config.mjs";
import {
  camel,
  differences,
  rustCommandArgs,
  rustEnumVariants,
  rustStructFields,
  tsFieldUnion,
  tsInterfaceFields,
  tsInvokeArgs,
  tsTaggedKinds,
  tsUnion,
} from "./contracts.mjs";
import { listFiles, repoFiles } from "./state.mjs";

const read = (rel) => readFileSync(join(ROOT, rel), "utf8");
const lineOf = (text, index) => text.slice(0, index).split("\n").length;
const sameSet = (a, b) => a.length === b.length && a.every((x) => b.includes(x));

const VIETNAMESE = /[ăâđêôơưàáạảãầấậẩẫằắặẳẵèéẹẻẽềếệểễìíịỉĩòóọỏõồốộổỗờớợởỡùúụủũừứựửữỳýỵỷỹ]/i;

const TAURI_CONFIG = "src-tauri/tauri.conf.json";

/** The updater's endpoints in the Tauri config, one per line: the only URLs there the app requests. */
function updaterEndpoints(text) {
  try {
    const endpoints = JSON.parse(text).plugins?.updater?.endpoints;
    return Array.isArray(endpoints) ? endpoints.join("\n") : "";
  } catch {
    return "";
  }
}

const isRust = (f) => f.startsWith("src-tauri/src/") && f.endsWith(".rs");
const isWeb = (f) => f.startsWith("src/") && /\.(ts|tsx)$/.test(f);
const rustFiles = () => listFiles(["src-tauri/src"]).filter(isRust);
const webFiles = () => listFiles(["src"]).filter(isWeb);

/**
 * Rust source with `#[cfg(test)] mod … { … }` blocks blanked out, so test-only code is exempt.
 * Blanking (instead of cutting) keeps line numbers and code after the module intact.
 */
function withoutTests(source) {
  let out = source;
  for (const m of source.matchAll(/#\[cfg\(test\)\]\s*mod\s+\w+\s*\{/g)) {
    let depth = 0;
    let end = source.length;
    for (let i = m.index + m[0].length - 1; i < source.length; i++) {
      if (source[i] === "{") depth++;
      else if (source[i] === "}" && --depth === 0) {
        end = i + 1;
        break;
      }
    }
    out = out.slice(0, m.index) + out.slice(m.index, end).replace(/[^\n]/g, " ") + out.slice(end);
  }
  return out;
}

/** Comments removed (line comments only; enough for the patterns below). */
const stripLineComment = (line) => line.replace(/(^|[^:"'])\/\/.*$/, "$1");

// ---------------------------------------------------------------------------------------------
// Per-file rules: (file, text) → [{ line, message, severity? }]

const FILE_RULES = {
  /** The repository is English; Vietnamese only in i18n files and Vietnamese test data. */
  language(file, text) {
    if (VIETNAMESE_ALLOWED.some((p) => p.test(file))) return [];
    const source = file.endsWith(".rs") ? withoutTests(text) : text;
    const out = [];
    source.split("\n").forEach((line, i) => {
      // Character sets (regex classes, letter lists) detect Vietnamese; they are not prose.
      const prose = line.replace(/\[[^\]]*\]/g, "").replace(/(["'`])[^\sA-Za-z"'`]{8,}\1/g, "");
      if (VIETNAMESE.test(prose)) {
        out.push({ line: i + 1, message: `Vietnamese text outside the i18n files: ${line.trim().slice(0, 70)}` });
      }
    });
    return out;
  },

  /** User text must never reach stdout/stderr or the browser console. */
  privacy(file, text) {
    const out = [];
    if (isRust(file)) {
      const source = withoutTests(text);
      const content = new RegExp(`\\b(${CONTENT_IDENTIFIERS.join("|")})\\b`);
      for (const m of source.matchAll(/\b(println!|print!|dbg!)\s*\(/g)) {
        out.push({
          line: lineOf(source, m.index),
          message: `${m[1]} is not allowed; use eprintln! for diagnostics without user text`,
        });
      }
      for (const m of source.matchAll(/\beprintln!\s*\(([^;]*)\)\s*;/g)) {
        if (content.test(m[1])) {
          out.push({ line: lineOf(source, m.index), message: `may log user text: eprintln!(${m[1].slice(0, 60)})` });
        }
      }
    }
    if (isWeb(file)) {
      for (const m of text.matchAll(/\bconsole\.(log|debug|info|trace)\s*\(/g)) {
        out.push({ line: lineOf(text, m.index), message: `console.${m[1]} is not allowed; remove debug logging` });
      }
    }
    return out;
  },

  /** Code-generation rules: where unsafe may live, error handling, typing, call boundaries. */
  code(file, text) {
    const out = [];
    if (isRust(file) || isWeb(file)) {
      const lines = text.split("\n").length;
      if (lines > MAX_FILE_LINES) {
        out.push({
          line: 1,
          severity: "warning",
          message: `${lines} lines (limit ${MAX_FILE_LINES}); split it by responsibility`,
        });
      }
    }
    if (isRust(file)) {
      const source = withoutTests(text);
      const lines = source.split("\n");
      if (!UNSAFE_ALLOWED.test(file)) {
        for (const m of source.matchAll(/\bunsafe\b/g)) {
          out.push({ line: lineOf(source, m.index), message: "unsafe code belongs in src-tauri/src/platform/" });
        }
      }
      for (const m of source.matchAll(/\.unwrap\(\)/g)) {
        // A poisoned mutex means another thread already panicked; unwrapping the lock is fine.
        if (!/\.lock\(\)\s*$/.test(source.slice(0, m.index))) {
          out.push({
            line: lineOf(source, m.index),
            message: 'unwrap() outside a mutex lock; use ?, a fallback, or expect("why this cannot fail")',
          });
        }
      }
      for (const m of source.matchAll(/\b(todo|unimplemented|panic)!\s*\(/g)) {
        out.push({
          line: lineOf(source, m.index),
          message: `${m[1]}! is not allowed in shipped code; return an error instead`,
        });
      }
      lines.forEach((line, i) => {
        if (/#!?\[allow\(/.test(line) && !/\/\/\s*\S/.test(line) && !/^\s*\/\/\s*\S/.test(lines[i - 1] ?? "")) {
          out.push({ line: i + 1, message: "#[allow(...)] needs a comment explaining why (same or previous line)" });
        }
      });
    }
    if (isWeb(file)) {
      text.split("\n").forEach((raw, i) => {
        // Lines of a block comment are prose, not code.
        const line = /^\s*(\/\*|\*)/.test(raw) ? "" : stripLineComment(raw);
        if (/(:\s*any\b|\bas\s+any\b|<any>|any\[\])/.test(line)) {
          out.push({ line: i + 1, message: "`any` is not allowed; use a precise type or `unknown`" });
        }
        if (/@ts-(ignore|nocheck)/.test(raw)) {
          out.push({ line: i + 1, message: "@ts-ignore/@ts-nocheck are not allowed; fix the type" });
        }
        if (/@ts-expect-error\s*$/.test(raw)) {
          out.push({ line: i + 1, message: "@ts-expect-error needs a description of the expected error" });
        }
        if (file !== INVOKE_FILE && /\binvoke\s*(<[^>]*>)?\s*\(/.test(line)) {
          out.push({ line: i + 1, message: `call Tauri commands through the typed wrappers in ${INVOKE_FILE}` });
        }
        if (file !== INVOKE_FILE && /\blisten\s*(<[^>]*>)?\s*\(/.test(line)) {
          out.push({ line: i + 1, message: `subscribe to Rust events through the typed \`events\` in ${INVOKE_FILE}` });
        }
      });
    }
    return out;
  },

  /** Code may only talk to known hosts, so user text cannot leak to a new endpoint. */
  network(file, text) {
    if (!isRust(file) && !isWeb(file) && file !== TAURI_CONFIG) return [];
    const source = file === TAURI_CONFIG ? updaterEndpoints(text) : isRust(file) ? withoutTests(text) : text;
    const out = [];
    for (const m of source.matchAll(/\b(?:https?|wss?):\/\/([A-Za-z0-9.-]+)/g)) {
      const host = m[1].toLowerCase();
      if (!ALLOWED_HOSTS.includes(host) && !BROWSER_LINK_HOSTS.includes(host)) {
        out.push({
          line: lineOf(source, m.index),
          message: `${m[1]} is not an allowed host (${ALLOWED_HOSTS.join(", ")}); new endpoints need the user's approval`,
        });
      }
    }
    return out;
  },

  /** No real credentials anywhere in the tree. */
  secrets(file, text) {
    if (GENERATED.test(file)) return [];
    return SECRET_PATTERNS.flatMap(({ name, pattern }) => {
      const m = pattern.exec(text);
      return m ? [{ line: lineOf(text, m.index), message: `looks like a real ${name}` }] : [];
    });
  },

  /** The .editorconfig rules for files no formatter touches: LF, no trailing blanks, one final newline, spaces. */
  whitespace(file, text) {
    if (GENERATED.test(file) || RECORDED.test(file)) return [];
    const out = [];
    const cr = text.indexOf("\r");
    if (cr >= 0) out.push({ line: lineOf(text, cr), message: "CR in a line ending; the repository uses LF" });
    const markdown = file.endsWith(".md");
    const lf = text.replaceAll("\r", "");
    const lines = lf.split("\n");
    lines.forEach((line, i) => {
      // Two trailing spaces are a line break in Markdown, so .editorconfig exempts it.
      if (!markdown && /[ \t]+$/.test(line)) out.push({ line: i + 1, message: "trailing whitespace" });
      if (!file.endsWith(".rs") && /^ *\t/.test(line)) {
        out.push({ line: i + 1, message: "tab indentation; indent with spaces" });
      }
    });
    if (lf.length > 0 && !lf.endsWith("\n")) {
      out.push({ line: lines.length, message: "no newline at the end of the file" });
    } else if (lf.endsWith("\n\n")) {
      out.push({ line: lines.length - 1, message: "more than one newline at the end of the file" });
    }
    return out;
  },

  /** A lint or format suppression hides a problem from the gates, so it must say why or not exist. */
  suppress(file, text) {
    const out = [];
    const script = /\.(ts|tsx|js|mjs)$/.test(file);
    if (file.endsWith(".rs")) {
      for (const m of text.matchAll(/#!?\[rustfmt::skip/g)) {
        out.push({ line: lineOf(text, m.index), message: "rustfmt::skip is not allowed; let rustfmt format the code" });
      }
    }
    if (script) {
      text.split("\n").forEach((line, i) => {
        if (/(?:\/\/|\/\*)\s*eslint-disable/.test(line) && !/\s--\s+\S/.test(line)) {
          out.push({ line: i + 1, message: "eslint-disable needs a reason after ` -- `; better, fix the code" });
        }
        if (/\/\*\s*eslint\s+[\w@/-]+\s*:/.test(line)) {
          out.push({ line: i + 1, message: "inline ESLint rule settings are not allowed; change eslint.config.js" });
        }
      });
    }
    if (script || /\.(css|html|ya?ml|json|toml)$/.test(file)) {
      for (const m of text.matchAll(/(?:\/\/|\/\*|<!--|#)\s*prettier-ignore/g)) {
        out.push({ line: lineOf(text, m.index), message: "prettier-ignore is not allowed; let Prettier format it" });
      }
    }
    return out;
  },
};

/** The text with CRLF turned into LF, trailing blanks removed (except in Markdown) and one final newline. */
export function fixWhitespace(file, text) {
  if (GENERATED.test(file) || RECORDED.test(file)) return text;
  let fixed = text.replace(/\r\n?/g, "\n");
  if (!file.endsWith(".md")) fixed = fixed.replace(/[ \t]+$/gm, "");
  fixed = fixed.replace(/\n+$/, "");
  return fixed.length > 0 ? `${fixed}\n` : fixed;
}

/** Violations of the per-file rules in one file, each tagged with its rule and source line. */
export function fileViolations(file, text, ruleNames = Object.keys(FILE_RULES)) {
  const lines = text.split("\n");
  return ruleNames.flatMap((rule) =>
    FILE_RULES[rule](file, text).map((v) => ({
      rule,
      severity: v.severity ?? "error",
      line: v.line,
      message: v.message,
      snippet: (lines[v.line - 1] ?? "").trim(),
    })),
  );
}

/** Every text file git would commit; the per-file rules apply to all of them, wherever they live. */
export function scannedFiles() {
  return repoFiles().filter((f) => TEXT_EXTENSIONS.test(f) || TEXT_NAMES.test(f));
}

// ---------------------------------------------------------------------------------------------
// Cross-file rules: () → { errors, warnings }

const CROSS_RULES = {
  /** The tone ids are listed in Rust, in the TS union and in the TS display order. */
  tones() {
    const errors = [];
    const rust = /pub const TONES:[^=]*=\s*&\[([\s\S]*?)\];/.exec(read("src-tauri/src/prompt.rs"));
    const rustTones = rust ? [...rust[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]) : null;
    const union = tsUnion(read("src/lib/api.ts"), "ToneId");
    const order = /TONE_IDS[^=]*=\s*\[([^\]]*)\]/.exec(read("src/lib/tones.ts"));
    const orderTones = order ? [...order[1].matchAll(/"([^"]+)"/g)].map((x) => x[1]) : null;
    if (!rustTones || !union || !orderTones) {
      return { errors: ["could not find TONES (prompt.rs), ToneId (api.ts) or TONE_IDS (tones.ts)"] };
    }
    if (!sameSet(rustTones, union)) errors.push(`prompt.rs TONES [${rustTones}] ≠ api.ts ToneId [${union}]`);
    if (rustTones.join() !== orderTones.join()) {
      errors.push(`tones.ts TONE_IDS [${orderTones}] must list prompt.rs TONES in the same order [${rustTones}]`);
    }
    return { errors };
  },

  /** Structs Rust sends to the UI (`SHAPES`) have the same camelCase field names as their api.ts interfaces. */
  shapes() {
    const ts = read(INVOKE_FILE);
    return {
      errors: SHAPES.flatMap(({ rust, name }) => {
        const rustFields = rustStructFields(read(rust), name)?.map(camel);
        const tsFields = tsInterfaceFields(ts, name);
        if (!rustFields || !tsFields)
          return [`could not find struct ${name} in ${rust} or interface ${name} in api.ts`];
        return differences(name, rust, rustFields, INVOKE_FILE, tsFields);
      }),
    };
  },

  /** Enums shared with the UI (`ENUMS`) have the same snake_case values as their api.ts unions. */
  enums() {
    const ts = read(INVOKE_FILE);
    return {
      errors: ENUMS.flatMap(({ rust, name, ts: tsName, tagged }) => {
        const variants = rustEnumVariants(read(rust), name);
        const [iface, field] = tsName.split(".");
        const values = tagged
          ? tsTaggedKinds(ts, tsName)
          : field
            ? tsFieldUnion(ts, iface, field)
            : tsUnion(ts, tsName);
        if (!variants || !values) return [`could not find enum ${name} in ${rust} or ${tsName} in api.ts`];
        return differences(`${name} ⇄ ${tsName}`, rust, variants, INVOKE_FILE, values);
      }),
    };
  },

  /**
   * Every `invoke("x")` has a `#[tauri::command] fn x` that is registered in generate_handler!, and the
   * typed wrapper in api.ts passes exactly the arguments the command takes.
   */
  commands() {
    const errors = [];
    const warnings = [];
    const rust = rustFiles()
      .map((f) => read(f))
      .join("\n");
    const defined = [...rust.matchAll(/#\[tauri::command\]\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)/g)].map((m) => m[1]);
    const handler = /generate_handler!\[([\s\S]*?)\]/.exec(rust);
    const registered = handler ? [...handler[1].matchAll(/(\w+)\s*,?/g)].map((m) => m[1]) : [];
    const invoked = new Map();
    for (const file of webFiles()) {
      const text = read(file);
      for (const m of text.matchAll(/invoke(?:<[^>]*>)?\(\s*["']([\w]+)["']/g)) {
        if (!invoked.has(m[1])) invoked.set(m[1], `${file}:${lineOf(text, m.index)}`);
      }
    }
    for (const [name, where] of invoked) {
      if (!defined.includes(name)) errors.push(`${where} invokes "${name}" but no #[tauri::command] fn ${name} exists`);
    }
    for (const name of defined) {
      if (!registered.includes(name)) errors.push(`#[tauri::command] fn ${name} is not listed in generate_handler!`);
      else if (!invoked.has(name)) warnings.push(`command ${name} is registered but never invoked from the UI`);
    }
    const takes = rustCommandArgs(rust);
    for (const [name, passed] of tsInvokeArgs(read(INVOKE_FILE))) {
      const expected = takes.get(name);
      if (expected)
        errors.push(...differences(`arguments of ${name}`, "the Rust command", expected, INVOKE_FILE, passed));
    }
    return { errors, warnings };
  },

  /** Every event the UI listens for is emitted by Rust. */
  events() {
    const errors = [];
    const warnings = [];
    const rust = rustFiles()
      .map((f) => read(f))
      .join("\n");
    const emitted = new Set(
      [...rust.matchAll(/\.emit(?:_to)?\(\s*(?:[A-Za-z_:]+\s*,\s*)?"([\w:-]+)"/g)].map((m) => m[1]),
    );
    const listened = new Map();
    for (const file of webFiles()) {
      const text = read(file);
      for (const m of text.matchAll(/\blisten(?:<[^>]*>)?\(\s*["']([\w:-]+)["']/g)) {
        listened.set(m[1], `${file}:${lineOf(text, m.index)}`);
      }
    }
    for (const [name, where] of listened) {
      if (!emitted.has(name)) errors.push(`${where} listens for "${name}" but Rust never emits it`);
    }
    for (const name of emitted) {
      if (!listened.has(name)) warnings.push(`Rust emits "${name}" but no UI code listens for it`);
    }
    return { errors, warnings };
  },

  /** Capabilities name real windows and grant what the frontend calls. */
  capabilities() {
    const errors = [];
    const conf = JSON.parse(read("src-tauri/tauri.conf.json"));
    const labels = (conf.app?.windows ?? []).map((w) => w.label ?? "main");
    const granted = new Set();
    for (const file of listFiles(["src-tauri/capabilities"]).filter((f) => f.endsWith(".json"))) {
      const cap = JSON.parse(read(file));
      for (const w of cap.windows ?? []) {
        if (w !== "*" && !labels.includes(w)) {
          errors.push(`${file} targets window "${w}", but tauri.conf.json only defines [${labels}]`);
        }
      }
      for (const p of cap.permissions ?? []) granted.add(typeof p === "string" ? p : p.identifier);
    }
    for (const file of webFiles()) {
      const text = read(file);
      for (const { pattern, permission } of REQUIRED_PERMISSIONS) {
        const m = pattern.exec(text);
        if (m && !granted.has(permission)) {
          errors.push(`${file}:${lineOf(text, m.index)} needs "${permission}" in src-tauri/capabilities`);
        }
      }
    }
    return { errors };
  },

  /** Lessons already paid for: keep them from regressing. */
  setup() {
    const errors = [];
    const cargo = read("src-tauri/Cargo.toml");
    const reqwest = /^reqwest\s*=.*$/m.exec(cargo)?.[0] ?? "";
    if (/rustls|default-features\s*=\s*false/.test(reqwest)) {
      errors.push(
        "reqwest must use the OS TLS stack (default features); rustls ignores corporate TLS-inspection roots",
      );
    }

    // Our `windows` crate must be the same minor version Tauri uses, or it compiles twice.
    const ours = /^windows\s*=\s*\{\s*version\s*=\s*"(\d+\.\d+)/m.exec(cargo)?.[1];
    const lock = read("src-tauri/Cargo.lock");
    const tauriPkg = /\[\[package\]\]\nname = "tauri"\nversion = "[^"]+"[\s\S]*?dependencies = \[([\s\S]*?)\]/.exec(
      lock,
    );
    const tauriWindows = tauriPkg && /"windows(?: (\d+\.\d+)[^"]*)?"/.exec(tauriPkg[1]);
    if (ours && tauriWindows) {
      const theirs = tauriWindows[1] ?? /\[\[package\]\]\nname = "windows"\nversion = "(\d+\.\d+)/.exec(lock)?.[1];
      if (theirs && theirs !== ours)
        errors.push(`windows crate ${ours} in Cargo.toml but Tauri uses ${theirs}; align them`);
    }

    const devPort = /localhost:(\d+)/.exec(JSON.parse(read("src-tauri/tauri.conf.json")).build?.devUrl ?? "")?.[1];
    const vitePort = /port:\s*(\d+)/.exec(read("vite.config.ts"))?.[1];
    if (devPort && vitePort && devPort !== vitePort) {
      errors.push(`tauri.conf.json devUrl port ${devPort} ≠ vite.config.ts port ${vitePort}`);
    }
    return { errors };
  },
};

/** Runs every rule; a rule that throws counts as a failure of that rule. */
export function runInvariants() {
  const results = Object.entries(CROSS_RULES).map(([name, rule]) => {
    try {
      const { errors = [], warnings = [] } = rule();
      return { name, errors, warnings };
    } catch (e) {
      return { name, errors: [`rule crashed: ${e.message}`], warnings: [] };
    }
  });

  const perFile = Object.fromEntries(Object.keys(FILE_RULES).map((r) => [r, { name: r, errors: [], warnings: [] }]));
  for (const file of scannedFiles()) {
    try {
      for (const v of fileViolations(file, read(file))) {
        const entry = `${file}:${v.line} ${v.message}`;
        perFile[v.rule][v.severity === "warning" ? "warnings" : "errors"].push(entry);
      }
    } catch (e) {
      perFile.code.errors.push(`${file}: rule crashed: ${e.message}`);
    }
  }
  return [...Object.values(perFile), ...results];
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  let failed = false;
  for (const { name, errors, warnings } of runInvariants()) {
    console.log(`${errors.length ? "FAIL" : "ok  "} ${name}`);
    for (const e of errors) console.log(`     ✗ ${e}`);
    for (const w of warnings) console.log(`     ! ${w}`);
    failed ||= errors.length > 0;
  }
  process.exit(failed ? 1 : 0);
}
