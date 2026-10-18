import { expect, test } from "vitest";
import { restore } from "./redaction";

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
