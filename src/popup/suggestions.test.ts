import { describe, expect, test } from "vitest";
import { ANSWER } from "../test/fixtures";
import { parseSuggestions, toVariants } from "./suggestions";

const redactions: [string, string][] = [["[EMAIL_1]", "an@corp.vn"]];

describe("parseSuggestions", () => {
  test("reads a complete answer", () => {
    expect(parseSuggestions(ANSWER).variants?.formal?.text).toBe("Could you please review my PR, [EMAIL_1]?");
  });

  test("reads an answer that is still streaming", () => {
    const partial = ANSWER.slice(0, ANSWER.indexOf("Fixed the"));
    const parsed = parseSuggestions(partial);
    expect(parsed.variants?.concise?.text).toBe("Please review my PR, [EMAIL_1].");
    expect(parsed.variants?.formal).toBeUndefined();
  });

  test("returns nothing for empty or broken output", () => {
    expect(parseSuggestions("")).toEqual({});
    expect(parseSuggestions("not json at all")).toEqual({});
  });
});

describe("toVariants", () => {
  test("a variant is ready once the next one has started", () => {
    const partial = parseSuggestions(ANSWER.slice(0, ANSWER.indexOf('"formal"') + 12));
    const [concise, formal] = toVariants(["concise", "formal"], partial, false, redactions);
    expect(concise.ready).toBe(true);
    expect(concise.text).toBe("Please review my PR, an@corp.vn.");
    expect(concise.note).toBe("Fixed the spelling.");
    expect(formal.ready).toBe(false);
  });

  test("the last variant is ready once the issues start or the request finishes", () => {
    const withIssues = parseSuggestions(ANSWER.slice(0, ANSWER.indexOf('"issues"') + 12));
    expect(toVariants(["concise", "formal"], withIssues, false, redactions)[1].ready).toBe(true);

    const noIssues = parseSuggestions(ANSWER.slice(0, ANSWER.indexOf('"issues"') - 1) + "}");
    expect(toVariants(["concise", "formal"], noIssues, false, redactions)[1].ready).toBe(false);
    expect(toVariants(["concise", "formal"], noIssues, true, redactions)[1].ready).toBe(true);
  });

  test("a variant without text is never ready", () => {
    const [concise] = toVariants(["concise"], {}, true, []);
    expect(concise).toEqual({ tone: "concise", text: "", note: "", ready: false });
  });
});
