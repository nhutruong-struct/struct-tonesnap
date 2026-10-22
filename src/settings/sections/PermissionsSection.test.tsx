import { act, fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, test } from "vitest";
import type { Permissions } from "../../lib/api";
import { translator } from "../../lib/i18n";
import { fakeBackend, settle } from "../../test/backend";
import { PermissionsSection, usePermissions } from "./PermissionsSection";

const t = translator("en", false);

function Harness() {
  return <PermissionsSection permissions={usePermissions()} t={t} />;
}

async function show(permissions: Permissions) {
  const backend = fakeBackend("settings", { get_permissions: () => permissions });
  render(<Harness />);
  await settle();
  return backend;
}

describe("permissions", () => {
  test("shows each permission as allowed or not, with buttons only for the missing one", async () => {
    await show({ required: true, accessibility: true, screenRecording: false });
    expect(screen.getAllByText("Allowed")).toHaveLength(1);
    expect(screen.getAllByText("Not allowed")).toHaveLength(1);
    expect(screen.getAllByRole("button", { name: "Allow…" })).toHaveLength(1);
    expect(screen.getAllByRole("button", { name: "Open System Settings" })).toHaveLength(1);
  });

  test("Allow asks the system and Open System Settings opens the pane of that permission", async () => {
    const backend = await show({ required: true, accessibility: false, screenRecording: true });
    act(() => {
      fireEvent.click(screen.getByRole("button", { name: "Allow…" }));
      fireEvent.click(screen.getByRole("button", { name: "Open System Settings" }));
    });
    await settle();
    expect(backend.callsTo("request_permission")).toEqual([{ permission: "accessibility" }]);
    expect(backend.callsTo("open_permission_settings")).toEqual([{ permission: "accessibility" }]);
  });

  test("reads the status again when the window regains focus", async () => {
    const backend = await show({ required: true, accessibility: false, screenRecording: false });
    const before = backend.callsTo("get_permissions").length;
    act(() => {
      window.dispatchEvent(new Event("focus"));
    });
    await settle();
    expect(backend.callsTo("get_permissions").length).toBeGreaterThan(before);
  });
});
