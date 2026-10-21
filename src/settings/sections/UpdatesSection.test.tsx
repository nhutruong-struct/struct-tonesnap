import { act, fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, test } from "vitest";
import type { Settings, UpdateInfo, UpdateStatus } from "../../lib/api";
import { fakeBackend, sendEvent, settle } from "../../test/backend";
import { settings as savedSettings } from "../../test/fixtures";
import SettingsPage from "../SettingsPage";

const info = (status: UpdateStatus, msiInstall = false, supported = true): UpdateInfo => ({
  currentVersion: "1.0.0",
  status,
  msiInstall,
  supported,
});

async function openUpdates(
  status: UpdateStatus = { kind: "idle" },
  settings: Partial<Settings> = {},
  msiInstall = false,
  supported = true,
) {
  const backend = fakeBackend("settings", {
    get_settings: () => savedSettings(settings),
    hotkey_error: () => null,
    settings_issue: () => null,
    has_api_key: () => true,
    update_info: () => info(status, msiInstall, supported),
    save_settings: () => null,
    "plugin:autostart|is_enabled": () => false,
  });
  render(<SettingsPage />);
  await settle();
  act(() => {
    fireEvent.click(screen.getByRole("button", { name: /Updates/ }));
  });
  await settle();
  return backend;
}

async function click(text: string) {
  act(() => {
    fireEvent.click(screen.getByText(text));
  });
  await settle();
}

describe("updates", () => {
  test("say so, and offer no check, where releases carry no updater installer", async () => {
    await openUpdates({ kind: "idle" }, {}, false, false);
    expect(screen.getByText(/Automatic updates are not available on macOS yet/)).toBeTruthy();
    expect(screen.queryByText("Check for Updates")).toBeNull();
  });

  test("shows the running version and the status", async () => {
    await openUpdates({ kind: "up_to_date" });
    expect(screen.getByText("Current version: 1.0.0")).toBeTruthy();
    expect(screen.getByRole("status").textContent).toBe("You have the latest version.");
    expect(screen.queryByText("Update & Restart")).toBeNull();
  });

  test("the update mode is saved with the other settings", async () => {
    const backend = await openUpdates();
    const auto = screen.getByLabelText<HTMLInputElement>(/Update automatically/);
    expect(auto.checked).toBe(true);
    await click("Ask before updating");
    await click("Save Settings");
    const saved = backend.callsTo("save_settings").map((args) => args.newSettings as Settings);
    expect(saved.map((s) => s.updateMode)).toEqual(["manual"]);
  });

  test("Check for Updates asks Rust at once and is disabled while it checks", async () => {
    const backend = await openUpdates();
    await click("Check for Updates");
    expect(backend.callsTo("check_for_updates")).toHaveLength(1);
    expect(backend.callsTo("save_settings")).toHaveLength(0);
    await sendEvent("update-status", info({ kind: "checking" }));
    expect(screen.getByText<HTMLButtonElement>("Check for Updates").disabled).toBe(true);
    expect(screen.getByRole("status").textContent).toBe("Checking for a new version…");
  });

  test("an available version can be installed with Update & Restart", async () => {
    const backend = await openUpdates({ kind: "available", version: "1.1.0" }, { updateMode: "manual" });
    expect(screen.getByRole("status").textContent).toBe("Version 1.1.0 is available.");
    await click("Update & Restart");
    expect(backend.callsTo("install_update")).toHaveLength(1);
  });

  test("a downloaded version says when it will be installed, following the chosen mode", async () => {
    await openUpdates({ kind: "ready", version: "1.1.0" });
    expect(screen.getByRole("status").textContent).toMatch(/installs it when you are not using the app/);
    await click("Ask before updating");
    expect(screen.getByRole("status").textContent).toBe("Version 1.1.0 is downloaded and ready to install.");
    expect(screen.getByText("Update & Restart")).toBeTruthy();
  });

  test("status events from Rust replace the shown status", async () => {
    await openUpdates({ kind: "up_to_date" });
    await sendEvent("update-status", info({ kind: "failed", step: "download", detail: "offline" }));
    expect(screen.getByRole("status").textContent).toBe("Could not download the new version: offline");
    await sendEvent("update-status", info({ kind: "installing", version: "1.1.0" }));
    expect(screen.getByRole("status").textContent).toBe("Installing version 1.1.0. ToneSnap will restart by itself.");
  });

  test("an MSI install explains that it always asks first", async () => {
    await openUpdates({ kind: "ready", version: "1.1.0" }, {}, true);
    expect(screen.getByText(/installed with the MSI package/)).toBeTruthy();
    expect(screen.getByRole("status").textContent).toBe("Version 1.1.0 is downloaded and ready to install.");
  });

  test("a status event is not replaced by the answer to the first request", async () => {
    let answer: (value: UpdateInfo) => void = () => undefined;
    fakeBackend("settings", {
      get_settings: () => savedSettings(),
      hotkey_error: () => null,
      settings_issue: () => null,
      has_api_key: () => true,
      update_info: () => new Promise<UpdateInfo>((resolve) => (answer = resolve)),
      "plugin:autostart|is_enabled": () => false,
    });
    render(<SettingsPage />);
    await settle();
    await sendEvent("update-status", info({ kind: "checking" }));
    answer(info({ kind: "idle" }));
    await settle();
    expect(screen.getByRole("status", { hidden: true }).textContent).toBe("Checking for a new version…");
  });
});
