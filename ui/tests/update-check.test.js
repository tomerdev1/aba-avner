import { describe, expect, it, vi } from "vitest";

import { checkForAppUpdate } from "../src/services/update-check.js";

function fakeLocalization() {
  return { t: (key, params) => `${key}:${params.version}` };
}

describe("checkForAppUpdate", () => {
  it("does nothing when no update is available", async () => {
    const confirm = vi.fn();
    const tauriClient = { checkForUpdate: vi.fn().mockResolvedValue(null), installUpdate: vi.fn() };

    await checkForAppUpdate(tauriClient, fakeLocalization(), confirm);

    expect(confirm).not.toHaveBeenCalled();
    expect(tauriClient.installUpdate).not.toHaveBeenCalled();
  });

  it("installs the update when the user confirms", async () => {
    const confirm = vi.fn().mockReturnValue(true);
    const tauriClient = {
      checkForUpdate: vi.fn().mockResolvedValue({ rid: 7, version: "0.2.0" }),
      installUpdate: vi.fn().mockResolvedValue(true),
    };

    await checkForAppUpdate(tauriClient, fakeLocalization(), confirm);

    expect(confirm).toHaveBeenCalledWith("updates.confirmInstall:0.2.0");
    expect(tauriClient.installUpdate).toHaveBeenCalledWith(7);
  });

  it("skips installing when the user declines", async () => {
    const confirm = vi.fn().mockReturnValue(false);
    const tauriClient = {
      checkForUpdate: vi.fn().mockResolvedValue({ rid: 7, version: "0.2.0" }),
      installUpdate: vi.fn(),
    };

    await checkForAppUpdate(tauriClient, fakeLocalization(), confirm);

    expect(tauriClient.installUpdate).not.toHaveBeenCalled();
  });

  it("skips the check entirely for an MSIX install", async () => {
    const confirm = vi.fn();
    const tauriClient = {
      isMsixInstall: vi.fn().mockResolvedValue(true),
      checkForUpdate: vi.fn(),
      installUpdate: vi.fn(),
    };

    await checkForAppUpdate(tauriClient, fakeLocalization(), confirm);

    expect(tauriClient.checkForUpdate).not.toHaveBeenCalled();
    expect(confirm).not.toHaveBeenCalled();
  });

  it("swallows errors from a failed update check", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    const tauriClient = { checkForUpdate: vi.fn().mockRejectedValue(new Error("offline")) };

    await expect(checkForAppUpdate(tauriClient, fakeLocalization(), vi.fn())).resolves.toBeUndefined();
    expect(consoleError).toHaveBeenCalled();
    consoleError.mockRestore();
  });

  it("no-ops when the client can't check for updates", async () => {
    await expect(checkForAppUpdate({}, fakeLocalization(), vi.fn())).resolves.toBeUndefined();
  });
});
