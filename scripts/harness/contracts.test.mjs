// Tests for the Rust ⇄ TypeScript contract parsers that the shapes, enums and commands rules use.
// Run by the `harness` gate of check.mjs, or directly: node --test scripts/harness

import assert from "node:assert/strict";
import { test } from "node:test";
import {
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

const RUST = `
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: u64,
    #[serde(skip)]
    pub hwnd: isize,
    pub context_source: ContextSource,
}

#[derive(Serialize)]
struct RequestEvent {
    session_id: u64,
    error_kind: Option<String>,
}

#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UpdateStatus {
    #[default]
    Idle,
    UpToDate,
    Failed {
        step: FailedStep,
    },
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, new_settings: Settings) -> Result<(), CommandError> {
    Ok(())
}

#[tauri::command]
pub fn send_request(state: tauri::State<'_, AppState>, target_language: TargetLanguage, mode: Mode) {}
`;

const TS = `
export type Mode = "translate" | "rewrite";

export interface RequestOptions {
  targetLanguage: TargetLanguage;
  mode: Mode;
}

export interface Payload extends RequestOptions {
  app: string;
  /** Prose: may mention a field: like this. */
  tones?: ToneId[];
}

export interface Session {
  source: "uia" | "clipboard";
  mode: Mode;
}

export type UpdateStatus =
  | { kind: "idle" }
  | { kind: "up_to_date" }
  | { kind: "failed"; step: "check" | "download"; detail: string };

export const api = {
  saveSettings: (newSettings: Settings) => invoke<void>("save_settings", { newSettings }),
  sendRequest: (options: RequestOptions) => invoke<void>("send_request", { ...options }),
  hidePopup: () => invoke<void>("hide_popup"),
};
`;

test("Rust struct fields skip #[serde(skip)] and include private structs", () => {
  assert.deepEqual(rustStructFields(RUST, "Session"), ["id", "context_source"]);
  assert.deepEqual(rustStructFields(RUST, "RequestEvent"), ["session_id", "error_kind"]);
  assert.equal(rustStructFields(RUST, "Missing"), null);
});

test("Rust enum variants are snake_cased, struct variants included", () => {
  assert.deepEqual(rustEnumVariants(RUST, "UpdateStatus"), ["idle", "up_to_date", "failed"]);
});

test("TS interfaces include the fields they inherit", () => {
  assert.deepEqual(tsInterfaceFields(TS, "Payload"), ["targetLanguage", "mode", "app", "tones"]);
});

test("TS unions are read from a type, a field's inline union or a field's named union", () => {
  assert.deepEqual(tsUnion(TS, "Mode"), ["translate", "rewrite"]);
  assert.deepEqual(tsFieldUnion(TS, "Session", "source"), ["uia", "clipboard"]);
  assert.deepEqual(tsFieldUnion(TS, "Session", "mode"), ["translate", "rewrite"]);
});

test("TS discriminated unions give the kind of every member, past the semicolons inside them", () => {
  assert.deepEqual(tsTaggedKinds(TS, "UpdateStatus"), ["idle", "up_to_date", "failed"]);
});

test("command arguments leave out what Tauri injects and are camelCased", () => {
  const args = rustCommandArgs(RUST);
  assert.deepEqual(args.get("save_settings"), ["newSettings"]);
  assert.deepEqual(args.get("send_request"), ["targetLanguage", "mode"]);
});

test("invoke arguments expand a spread into the fields of its parameter type", () => {
  const args = tsInvokeArgs(TS);
  assert.deepEqual(args.get("save_settings"), ["newSettings"]);
  assert.deepEqual(args.get("send_request"), ["targetLanguage", "mode"]);
  assert.deepEqual(args.get("hide_popup"), []);
});

test("differences name what each side lacks", () => {
  assert.deepEqual(differences("Session", "rust", ["id", "app"], "ts", ["id", "title"]), [
    "Session: [app] in rust but not in ts",
    "Session: [title] in ts but not in rust",
  ]);
  assert.deepEqual(differences("Session", "rust", ["id"], "ts", ["id"]), []);
});

test("a renamed field or a new enum value on one side is caught", () => {
  const renamed = TS.replace("app: string;", "appName: string;");
  const rustPayload =
    "pub struct Payload {\n    pub target_language: L,\n    pub mode: Mode,\n    pub app: String,\n    pub tones: Vec<String>,\n}";
  const rustFields = rustStructFields(rustPayload, "Payload").map((f) =>
    f.replace(/_(\w)/g, (_, c) => c.toUpperCase()),
  );
  assert.equal(differences("Payload", "rust", rustFields, "ts", tsInterfaceFields(renamed, "Payload")).length, 2);
  const extra = RUST.replace("    UpToDate,", "    UpToDate,\n    Paused,");
  assert.deepEqual(
    differences(
      "UpdateStatus",
      "rust",
      rustEnumVariants(extra, "UpdateStatus"),
      "ts",
      tsTaggedKinds(TS, "UpdateStatus"),
    ),
    ["UpdateStatus: [paused] in rust but not in ts"],
  );
});
