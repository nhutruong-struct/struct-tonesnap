import { expect, test } from "vitest";
import { missingRedactions, restore } from "./redaction";

test("restores every placeholder, including repeats", () => {
  const redactions: [string, string][] = [
    ["[EMAIL_1]", "an@corp.vn"],
    ["[PHONE_1]", "0912 345 678"],
  ];
  expect(restore("Mail [EMAIL_1] or call [PHONE_1]; [EMAIL_1] again", redactions)).toBe(
    "Mail an@corp.vn or call 0912 345 678; an@corp.vn again",
  );
});

test("does not confuse placeholders that share a prefix", () => {
  const redactions: [string, string][] = [
    ["[TERM_1]", "Falcon"],
    ["[TERM_10]", "Osprey"],
  ];
  expect(restore("[TERM_10] and [TERM_1]", redactions)).toBe("Osprey and Falcon");
});

test("leaves text without placeholders alone", () => {
  expect(restore("Nothing to restore", [])).toBe("Nothing to restore");
});

test("restores placeholders the model reworded", () => {
  const redactions: [string, string][] = [["[EMAIL_1]", "an@corp.vn"]];
  for (const reworded of ["[email_1]", "[Email 1]", "[EMAIL-1]", "[ EMAIL_1 ]"]) {
    expect(restore(`Mail ${reworded}`, redactions)).toBe("Mail an@corp.vn");
  }
});

test("keeps bracketed text that is no placeholder of this session", () => {
  expect(restore("See [NOTE_2] and [1]", [["[NOTE_1]", "x"]])).toBe("See [NOTE_2] and [1]");
});

test("never re-reads a restored value as a placeholder", () => {
  const redactions: [string, string][] = [
    ["[TERM_1]", "[TERM_2]"],
    ["[TERM_2]", "Osprey"],
  ];
  expect(restore("[TERM_1]", redactions)).toBe("[TERM_2]");
});

test("names the masked values of the source that the output lost", () => {
  const redactions: [string, string][] = [
    ["[EMAIL_1]", "an@corp.vn"],
    ["[PHONE_1]", "0912 345 678"],
    ["[TERM_1]", "Osprey"],
  ];
  const source = "mail [EMAIL_1], call [PHONE_1]";
  expect(missingRedactions(source, "Mail [email 1] or call.", redactions)).toEqual(["0912 345 678"]);
  // A value that was only in the context need not reappear.
  expect(missingRedactions(source, "Mail [EMAIL_1], call [PHONE_1].", redactions)).toEqual([]);
});
