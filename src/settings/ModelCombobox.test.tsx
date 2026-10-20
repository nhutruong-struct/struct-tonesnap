import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { afterEach, describe, expect, test } from "vitest";
import { ModelCombobox } from "./ModelCombobox";

// Vitest globals are off, so Testing Library does not unmount between tests on its own.
afterEach(cleanup);

const OPTIONS = ["alpha-1", "beta-2", "gamma-3"];

function Harness({ initial = "alpha-1" }: { initial?: string }) {
  const [value, setValue] = useState(initial);
  return <ModelCombobox value={value} options={OPTIONS} onChange={setValue} toggleLabel="Show suggestions" />;
}

const input = () => screen.getByRole<HTMLInputElement>("combobox");
const options = () => screen.queryAllByRole("option").map((o) => o.textContent);

describe("ModelCombobox", () => {
  test("stays closed until the arrow is clicked, then lists every suggestion whatever the field holds", () => {
    render(<Harness initial="my-own-model" />);
    expect(screen.queryByRole("listbox")).toBeNull();

    fireEvent.click(screen.getByRole("button", { name: "Show suggestions" }));
    expect(options()).toEqual(OPTIONS);
    expect(input().getAttribute("aria-expanded")).toBe("true");
  });

  test("clicking a suggestion fills the field and closes the list", () => {
    render(<Harness />);
    fireEvent.click(screen.getByRole("button", { name: "Show suggestions" }));
    fireEvent.click(screen.getByRole("option", { name: "beta-2" }));

    expect(input().value).toBe("beta-2");
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  test("marks the current value as selected", () => {
    render(<Harness initial="beta-2" />);
    fireEvent.click(screen.getByRole("button", { name: "Show suggestions" }));

    expect(screen.getByRole("option", { name: "beta-2" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByRole("option", { name: "alpha-1" }).getAttribute("aria-selected")).toBe("false");
  });

  test("the arrow keys open the list, move through it and wrap around; Enter picks", () => {
    render(<Harness />);
    fireEvent.keyDown(input(), { key: "ArrowDown" });
    expect(options()).toEqual(OPTIONS);

    fireEvent.keyDown(input(), { key: "ArrowDown" });
    fireEvent.keyDown(input(), { key: "ArrowDown" });
    fireEvent.keyDown(input(), { key: "ArrowDown" });
    expect(input().getAttribute("aria-activedescendant")).toBe(screen.getByRole("option", { name: "alpha-1" }).id);

    fireEvent.keyDown(input(), { key: "ArrowUp" });
    fireEvent.keyDown(input(), { key: "Enter" });
    expect(input().value).toBe("gamma-3");
    expect(screen.queryByRole("listbox")).toBeNull();
  });

  test("Escape closes the list and keeps the text", () => {
    render(<Harness />);
    fireEvent.keyDown(input(), { key: "ArrowDown" });
    fireEvent.keyDown(input(), { key: "Escape" });

    expect(screen.queryByRole("listbox")).toBeNull();
    expect(input().value).toBe("alpha-1");
  });

  test("any other name can still be typed", () => {
    render(<Harness />);
    fireEvent.change(input(), { target: { value: "vendor/custom-model" } });
    expect(input().value).toBe("vendor/custom-model");
  });

  test("moving the focus away closes the list", () => {
    render(
      <>
        <Harness />
        <button>elsewhere</button>
      </>,
    );
    input().focus();
    fireEvent.keyDown(input(), { key: "ArrowDown" });
    fireEvent.blur(input(), { relatedTarget: screen.getByText("elsewhere") });

    expect(screen.queryByRole("listbox")).toBeNull();
  });
});
