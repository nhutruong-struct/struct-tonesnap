// Parsers for the Rust ⇄ TypeScript contracts. They read source text only, so the cross-file rules
// in invariants.mjs and the tests in contracts.test.mjs share them. They are regular expressions
// tuned to this codebase's style (rustfmt and Prettier output), not full parsers.

export const camel = (snake) => snake.replace(/_([a-z0-9])/g, (_, c) => c.toUpperCase());

/**
 * The text of a TS type alias: everything after `type Name =` up to its closing semicolon, which is
 * the first one outside braces (object members are separated by semicolons too).
 */
function tsTypeBody(source, name) {
  const m = new RegExp(`type\\s+${name}\\s*=`).exec(source);
  if (!m) return null;
  let depth = 0;
  for (let i = m.index + m[0].length; i < source.length; i++) {
    const c = source[i];
    if (c === "{") depth++;
    else if (c === "}") depth--;
    else if (c === ";" && depth === 0) return source.slice(m.index + m[0].length, i);
  }
  return null;
}

/** Members of a TS string-literal union, e.g. `export type ToneId = "a" | "b"`. */
export function tsUnion(source, name) {
  const body = tsTypeBody(source, name);
  return body === null ? null : [...body.matchAll(/"([^"]+)"/g)].map((x) => x[1]);
}

/** The `kind` of each member of a TS discriminated union, e.g. `{ kind: "idle" } | { kind: "failed"; … }`. */
export function tsTaggedKinds(source, name) {
  const body = tsTypeBody(source, name);
  return body === null ? null : [...body.matchAll(/\bkind:\s*"([^"]+)"/g)].map((x) => x[1]);
}

/** The body of a TS interface and the interfaces it extends. */
function tsInterface(source, name) {
  const m = new RegExp(`interface\\s+${name}(?:\\s+extends\\s+([\\w,\\s]+?))?\\s*\\{([\\s\\S]*?)\\n\\}`).exec(source);
  if (!m) return null;
  const parents = m[1] ? m[1].split(",").map((p) => p.trim()) : [];
  return { parents, body: m[2] };
}

/** Field names of a TS interface, inherited ones included (top level only). */
export function tsInterfaceFields(source, name) {
  const found = tsInterface(source, name);
  if (!found) return null;
  const own = [...found.body.matchAll(/^\s+(\w+)\??\s*:/gm)].map((x) => x[1]);
  const inherited = found.parents.flatMap((p) => tsInterfaceFields(source, p) ?? []);
  return [...inherited, ...own];
}

/**
 * The string literals a TS interface field allows: written inline (`source: "a" | "b"`) or through
 * the name of a union type.
 */
export function tsFieldUnion(source, iface, field) {
  const body = tsInterface(source, iface)?.body ?? "";
  const type = new RegExp(`^\\s+${field}\\??:\\s*([^;]+);`, "m").exec(body)?.[1].trim();
  if (!type) return null;
  return /^\w+$/.test(type) ? tsUnion(source, type) : [...type.matchAll(/"([^"]+)"/g)].map((x) => x[1]);
}

/** Field names of a Rust struct (public or not), skipping `#[serde(skip)]` fields. */
export function rustStructFields(source, name) {
  const m = new RegExp(`(?:^|\\n)\\s*(?:pub(?:\\([^)]*\\))?\\s+)?struct ${name}\\s*\\{([\\s\\S]*?)\\n\\}`).exec(source);
  if (!m) return null;
  const fields = [];
  let skip = false;
  for (const line of m[1].split("\n")) {
    if (/#\[serde\([^)]*\bskip\b/.test(line)) skip = true;
    const f = /^\s*(?:pub(?:\([^)]*\))?\s+)?(\w+)\s*:/.exec(line);
    if (f && !line.trim().startsWith("//")) {
      if (!skip) fields.push(f[1]);
      skip = false;
    }
  }
  return fields;
}

/** Variants of a Rust enum, snake_cased the way `#[serde(rename_all = "snake_case")]` does. */
export function rustEnumVariants(source, name) {
  const m = new RegExp(`pub enum ${name}\\s*\\{([\\s\\S]*?)\\n\\}`).exec(source);
  if (!m) return null;
  return [...m[1].matchAll(/^\s*([A-Z]\w*)\s*[,{(]/gm)].map((x) =>
    x[1].replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase(),
  );
}

/** Parameter types Tauri fills in itself; the UI never passes them. */
const INJECTED = /(?:^|::)(AppHandle|State|Window|WebviewWindow)$/;

/** The arguments each `#[tauri::command]` takes from the UI, camelCased as Tauri expects them. */
export function rustCommandArgs(source) {
  const commands = new Map();
  const re = /#\[tauri::command\]\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*(?:<[^>]*>)?\(([^)]*)\)/g;
  for (const [, name, params] of source.matchAll(re)) {
    const args = [...params.matchAll(/(\w+)\s*:(?!:)\s*([\w:]+)/g)]
      .filter(([, , type]) => !INJECTED.test(type))
      .map(([, param]) => camel(param));
    commands.set(name, args);
  }
  return commands;
}

/**
 * The arguments each typed wrapper in api.ts passes to `invoke`, e.g.
 * `setApiKey: (provider: ProviderId, key: string) => invoke<void>("set_api_key", { provider, key })`.
 * A spread (`{ ...options }`) stands for the fields of that parameter's interface.
 */
export function tsInvokeArgs(source) {
  const calls = new Map();
  const re = /\(([^)]*)\)\s*=>\s*invoke(?:<[^>]*>)?\(\s*"(\w+)"\s*(?:,\s*\{([^}]*)\})?\s*\)/g;
  for (const [, params, name, object = ""] of source.matchAll(re)) {
    const types = new Map(
      params
        .split(",")
        .map((p) => p.split(":").map((s) => s.trim()))
        .filter(([param, type]) => param && type),
    );
    const args = object
      .split(",")
      .map((part) => part.trim())
      .filter(Boolean)
      .flatMap((part) => {
        if (!part.startsWith("...")) return [part.split(":")[0].trim()];
        const type = types.get(part.slice(3).trim());
        return (type && tsInterfaceFields(source, type)) ?? [part];
      });
    calls.set(name, args);
  }
  return calls;
}

/** What `a` has that `b` lacks and the other way round, as messages naming both sides. */
export function differences(label, aName, a, bName, b) {
  const errors = [];
  const onlyA = a.filter((x) => !b.includes(x));
  const onlyB = b.filter((x) => !a.includes(x));
  if (onlyA.length) errors.push(`${label}: [${onlyA.join(", ")}] in ${aName} but not in ${bName}`);
  if (onlyB.length) errors.push(`${label}: [${onlyB.join(", ")}] in ${bName} but not in ${aName}`);
  return errors;
}
