import { describe, expect, test } from "vitest";
import { ANSWER } from "../test/fixtures";
import { parseSuggestions, toVariants } from "./suggestions";

const redactions: [string, string][] = [["[EMAIL_1]", "an@corp.vn"]];

describe("parseSuggestions", () => {
  test("reads a complete answer", () => {
    expect(parseSuggestions(ANSWER).variants?.polite?.text).toBe("Could you please review my PR, [EMAIL_1]?");
  });

  test("reads an answer that is still streaming", () => {
    const partial = ANSWER.slice(0, ANSWER.indexOf("Fixed the"));
    const parsed = parseSuggestions(partial);
    expect(parsed.variants?.minimal?.text).toBe("Please review my PR, [EMAIL_1].");
    expect(parsed.variants?.polite).toBeUndefined();
  });

  test("returns nothing for empty or broken output", () => {
    expect(parseSuggestions("")).toEqual({});
    expect(parseSuggestions("not json at all")).toEqual({});
  });
});

describe("toVariants", () => {
  test("a variant is ready once the next one has started", () => {
    const partial = parseSuggestions(ANSWER.slice(0, ANSWER.indexOf('"polite"') + 12));
    const [minimal, polite] = toVariants(["minimal", "polite"], partial, false, redactions);
    expect(minimal.ready).toBe(true);
    expect(minimal.text).toBe("Please review my PR, an@corp.vn.");
    expect(minimal.note).toBe("Fixed the spelling.");
    expect(polite.ready).toBe(false);
  });

  test("the last variant is ready once the issues start or the request finishes", () => {
    const withIssues = parseSuggestions(ANSWER.slice(0, ANSWER.indexOf('"issues"') + 12));
    expect(toVariants(["minimal", "polite"], withIssues, false, redactions)[1].ready).toBe(true);

    const noIssues = parseSuggestions(ANSWER.slice(0, ANSWER.indexOf('"issues"') - 1) + "}");
    expect(toVariants(["minimal", "polite"], noIssues, false, redactions)[1].ready).toBe(false);
    expect(toVariants(["minimal", "polite"], noIssues, true, redactions)[1].ready).toBe(true);
  });

  test("a variant without text is never ready", () => {
    const [minimal] = toVariants(["minimal"], {}, true, []);
    expect(minimal).toEqual({ tone: "minimal", text: "", note: "", ready: false });
  });
});
