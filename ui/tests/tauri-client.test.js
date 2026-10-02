import { describe, expect, it, vi } from "vitest";

import {
  createTauriClient,
  isStructuredTauriError,
} from "../src/services/tauri-client.js";

describe("tauri client", () => {
  it("reports availability only when invoke and dialog APIs exist", () => {
    expect(createTauriClient({}).isAvailable()).toBe(false);
    expect(
      createTauriClient({
        core: { invoke: vi.fn() },
        dialog: { open: vi.fn() },
      }).isAvailable(),
    ).toBe(true);
  });

  it("forwards dedupe requests using the current command contract", async () => {
    const invoke = vi.fn().mockResolvedValue({ warnings: [] });
    const client = createTauriClient({
      core: { invoke },
      dialog: { open: vi.fn() },
    });

    await client.runDedupe({
      inputDir: "/input",
      outputDir: "/output",
      minSizeMb: 2.5,
      similarityThreshold: 6,
      filterExistingOutput: false,
      runId: "run-123",
      locale: "he",
    });

    expect(invoke).toHaveBeenCalledWith("run_dedupe", {
      config: {
        inputDir: "/input",
        outputDir: "/output",
        similarityThreshold: 6,
        minImageSizeBytes: Math.round(2.5 * 1024 * 1024),
        filterExistingOutput: false,
        runId: "run-123",
        locale: "he",
      },
    });
  });

  it("normalizes warning arrays from dedupe results", async () => {
    const invoke = vi.fn().mockResolvedValue({
      totalImages: 4,
      report: {
        storageSavedBytes: 50,
        storageSavedHuman: "50 B",
        summary: {
          totalImages: 4,
          uniqueImages: 2,
          duplicateImages: 2,
        },
        folderBreakdown: [
          {
            path: "/input/a",
            duplicates: 1,
            wastedBytes: 20,
          },
        ],
      },
      warnings: ["warning 1", 2, null],
      warningDetails: [
        {
          code: "file_issue",
          path: "/input/a.png",
          detail: "permission denied",
        },
        {
          code: "bad_detail",
          path: 5,
        },
      ],
    });
    const client = createTauriClient({
      core: { invoke },
      dialog: { open: vi.fn() },
    });

    await expect(
      client.runDedupe({
        inputDir: "/input",
        outputDir: "/output",
        minSizeMb: 2.5,
        filterExistingOutput: true,
      }),
    ).resolves.toEqual({
      totalImages: 4,
      report: {
        storageSavedBytes: 50,
        storageSavedHuman: "50 B",
        summary: {
          totalImages: 4,
          uniqueImages: 2,
          duplicateImages: 2,
        },
        folderBreakdown: [
          {
            path: "/input/a",
            duplicates: 1,
            wastedBytes: 20,
          },
        ],
      },
      warnings: ["warning 1"],
      warningDetails: [
        {
          code: "file_issue",
          path: "/input/a.png",
          detail: "permission denied",
        },
      ],
    });
  });

  it("defaults missing warning arrays to empty lists", async () => {
    const client = createTauriClient({
      core: { invoke: vi.fn().mockResolvedValue({ totalImages: 1 }) },
      dialog: { open: vi.fn() },
    });

    await expect(
      client.runDedupe({
        inputDir: "/input",
        outputDir: "/output",
        minSizeMb: 1,
        filterExistingOutput: true,
      }),
    ).resolves.toEqual({
      totalImages: 1,
      warnings: [],
      warningDetails: [],
    });
  });

  it("returns null when directory selection is unavailable", async () => {
    const client = createTauriClient({});

    await expect(client.chooseDirectory("Pick folder")).resolves.toBeNull();
  });

  it("subscribes to the current progress event name", async () => {
    const listen = vi.fn().mockResolvedValue(() => {});
    const client = createTauriClient({
      event: { listen },
    });
    const handler = vi.fn();

    await client.listenToProgress(handler);

    expect(listen).toHaveBeenCalledWith("dedupe_progress", expect.any(Function));
  });

  it("filters malformed progress payloads before forwarding them", async () => {
    let forwardedListener;
    const listen = vi.fn().mockImplementation(async (_event, listener) => {
      forwardedListener = listener;
      return () => {};
    });
    const client = createTauriClient({
      event: { listen },
    });
    const handler = vi.fn();

    await client.listenToProgress(handler);
    forwardedListener({ payload: { stage: "hash", current: 1, total: 2 } });
    forwardedListener({ payload: { runId: "run-123", stage: "hash", current: 1, total: 2 } });

    expect(handler).toHaveBeenCalledTimes(1);
    expect(handler).toHaveBeenCalledWith({
      runId: "run-123",
      stage: "hash",
      current: 1,
      total: 2,
    });
  });

  it("invokes the cancellation command and returns a boolean", async () => {
    const invoke = vi.fn().mockResolvedValue(true);
    const client = createTauriClient({
      core: { invoke },
      dialog: { open: vi.fn() },
    });

    await expect(client.cancelDedupe()).resolves.toBe(true);
    expect(invoke).toHaveBeenCalledWith("cancel_dedupe");
  });

  it("reports MSIX install status", async () => {
    const invoke = vi.fn().mockResolvedValue(true);
    const client = createTauriClient({ core: { invoke } });

    await expect(client.isMsixInstall()).resolves.toBe(true);
    expect(invoke).toHaveBeenCalledWith("is_msix_install");
  });

  it("treats a missing invoke or a failed MSIX check as not MSIX", async () => {
    await expect(createTauriClient({}).isMsixInstall()).resolves.toBe(false);

    const client = createTauriClient({
      core: { invoke: vi.fn().mockRejectedValue(new Error("no such command")) },
    });
    await expect(client.isMsixInstall()).resolves.toBe(false);
  });

  it("forwards review export requests with the similar-group export flag", async () => {
    const invoke = vi.fn().mockResolvedValue({ warnings: [] });
    const client = createTauriClient({
      core: { invoke },
      dialog: { open: vi.fn() },
    });

    await client.exportDedupeReview({
      outputDir: "/output",
      similarityThreshold: 6,
      filterExistingOutput: false,
      exportSimilarImageGroups: false,
      groups: [],
      selections: {},
      runId: "run-123",
      locale: "he",
    });

    expect(invoke).toHaveBeenCalledWith("export_dedupe_review", {
      config: {
        outputDir: "/output",
        similarityThreshold: 6,
        filterExistingOutput: false,
        exportSimilarImageGroups: false,
        groups: [],
        selections: {},
        runId: "run-123",
        locale: "he",
      },
    });
  });

  it("normalizes structured Tauri boundary errors", async () => {
    const client = createTauriClient({
      core: {
        invoke: vi.fn().mockRejectedValue({
          code: "invalid_input_dir",
          message: "Invalid input directory: /input",
        }),
      },
      dialog: { open: vi.fn() },
    });

    await expect(
      client.runDedupe({
        inputDir: "/input",
        outputDir: "/output",
        minSizeMb: 1,
        filterExistingOutput: true,
      }),
    ).rejects.toEqual({
      code: "invalid_input_dir",
      message: "Invalid input directory: /input",
    });
  });

  it("normalizes unknown errors into a developer-traceable contract", async () => {
    const error = new Error("disk full");
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    const client = createTauriClient({
      core: { invoke: vi.fn().mockRejectedValue(error) },
      dialog: { open: vi.fn() },
    });

    await expect(
      client.runDedupe({
        inputDir: "/input",
        outputDir: "/output",
        minSizeMb: 1,
        filterExistingOutput: true,
      }),
    ).rejects.toEqual({
      code: "unexpected_tauri_error",
      message: "unexpected_tauri_error:Error",
    });

    expect(consoleError).toHaveBeenCalledWith("Unexpected Tauri error payload", error);
    consoleError.mockRestore();
  });

  it("checks for updates and normalizes the response", async () => {
    const invoke = vi.fn().mockResolvedValue({
      rid: 1,
      currentVersion: "0.1.0",
      version: "0.2.0",
      body: "Bug fixes",
      rawJson: {},
    });
    const client = createTauriClient({ core: { invoke } });

    await expect(client.checkForUpdate()).resolves.toEqual({
      rid: 1,
      version: "0.2.0",
      currentVersion: "0.1.0",
      body: "Bug fixes",
    });
    expect(invoke).toHaveBeenCalledWith("plugin:updater|check", {});
  });

  it("returns null when no update is available or invoke is unavailable", async () => {
    const client = createTauriClient({ core: { invoke: vi.fn().mockResolvedValue(null) } });
    await expect(client.checkForUpdate()).resolves.toBeNull();
    await expect(createTauriClient({}).checkForUpdate()).resolves.toBeNull();
  });

  it("downloads and installs an update via a Tauri IPC channel", async () => {
    const invoke = vi.fn().mockResolvedValue(undefined);
    class FakeChannel {}
    const client = createTauriClient({ core: { invoke, Channel: FakeChannel } });

    await expect(client.installUpdate(1)).resolves.toBe(true);
    expect(invoke).toHaveBeenCalledWith("plugin:updater|download_and_install", {
      rid: 1,
      onEvent: expect.any(FakeChannel),
    });
  });

  it("skips installing when the Channel API is unavailable", async () => {
    const client = createTauriClient({ core: { invoke: vi.fn() } });
    await expect(client.installUpdate(1)).resolves.toBe(false);
  });

  it("detects the structured Tauri error contract", () => {
    expect(
      isStructuredTauriError({
        code: "copy_image_failed",
        message: "Failed to copy image: file.png",
      }),
    ).toBe(true);
    expect(isStructuredTauriError(new Error("disk full"))).toBe(false);
    expect(isStructuredTauriError("disk full")).toBe(false);
  });
});
