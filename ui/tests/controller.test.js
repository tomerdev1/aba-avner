import { describe, expect, it, vi } from "vitest";

import { createAppController } from "../src/app/controller.js";

function createLocalization() {
  let locale = "en";

  return {
    getLocale: () => locale,
    setLocale(nextLocale) {
      locale = nextLocale;
    },
    subscribe: vi.fn(),
    t(key) {
      return key;
    },
  };
}

function createSettingsStore(
  minSizeMb = 0,
  filterExistingOutput = true,
  similarityThreshold = 10,
  visualReviewEnabled = true,
  exportSimilarImageGroups = true,
) {
  let value = minSizeMb;
  let filterValue = filterExistingOutput;
  let thresholdValue = similarityThreshold;
  let reviewValue = visualReviewEnabled;
  let exportGroupsValue = exportSimilarImageGroups;

  return {
    getMinSizeMb: () => value,
    setMinSizeMb(nextValue) {
      value = nextValue;
    },
    getSimilarityThreshold: () => thresholdValue,
    setSimilarityThreshold(nextValue) {
      thresholdValue = nextValue;
    },
    getFilterExistingOutput: () => filterValue,
    setFilterExistingOutput(nextValue) {
      filterValue = nextValue;
    },
    getVisualReviewEnabled: () => reviewValue,
    setVisualReviewEnabled(nextValue) {
      reviewValue = nextValue;
    },
    getExportSimilarImageGroups: () => exportGroupsValue,
    setExportSimilarImageGroups(nextValue) {
      exportGroupsValue = nextValue;
    },
  };
}

function createHarness(overrides = {}) {
  const render = vi.fn();
  const localization = overrides.localization ?? createLocalization();
  const settingsStore = overrides.settingsStore ?? createSettingsStore(2.5);
  const tauriClient = overrides.tauriClient ?? {
    isAvailable: () => true,
    chooseDirectory: vi.fn().mockResolvedValue("/chosen"),
    cancelDedupe: vi.fn().mockResolvedValue(true),
    listenToProgress: vi.fn().mockResolvedValue(() => {}),
    runDedupe: vi.fn().mockResolvedValue({ warnings: [] }),
  };

  const controller = createAppController(
    {},
    tauriClient,
    localization,
    settingsStore,
    render,
  );

  return {
    controller,
    localization,
    render,
    settingsStore,
    tauriClient,
  };
}

describe("app controller", () => {
  it("renders the initial state from persisted settings", () => {
    const { render } = createHarness();

    expect(render).toHaveBeenCalledTimes(1);
    expect(render.mock.calls[0][0]).toMatchObject({
      view: "main",
      runStatus: "idle",
      minSizeMb: 2.5,
      similarityThreshold: 10,
      filterExistingOutput: true,
      visualReviewEnabled: true,
      exportSimilarImageGroups: true,
      locale: "en",
    });
  });

  it("shows a Tauri-unavailable issue without attempting a run", async () => {
    const tauriClient = {
      isAvailable: () => false,
      chooseDirectory: vi.fn(),
      listenToProgress: vi.fn(),
      runDedupe: vi.fn(),
    };
    const { controller, render } = createHarness({ tauriClient });

    await controller.runDedupe();

    const latestState = render.mock.calls.at(-1)[0];
    expect(latestState.view).toBe("main");
    expect(latestState.issues.title).toBe("issues.tauriUnavailableTitle");
    expect(latestState.issues.items).toEqual(["issues.tauriUnavailableItem"]);
    expect(tauriClient.chooseDirectory).not.toHaveBeenCalled();
    expect(tauriClient.runDedupe).not.toHaveBeenCalled();
  });

  it("surfaces directory picker failures instead of leaving the app stuck running", async () => {
    const tauriClient = {
      isAvailable: () => true,
      chooseDirectory: vi.fn().mockRejectedValue({
        code: "invalid_input_dir",
        message: "Invalid input directory: /missing",
      }),
      cancelDedupe: vi.fn().mockResolvedValue(true),
      listenToProgress: vi.fn(),
      runDedupe: vi.fn(),
    };
    const { controller, render } = createHarness({ tauriClient });

    await controller.runDedupe();

    const latestState = render.mock.calls.at(-1)[0];
    expect(latestState.runStatus).toBe("error");
    expect(latestState.issues.items).toEqual(["issues.errors.invalidInputDir"]);
    expect(tauriClient.runDedupe).not.toHaveBeenCalled();
  });

  it("surfaces progress listener failures instead of leaving the run active", async () => {
    const tauriClient = {
      isAvailable: () => true,
      chooseDirectory: vi.fn().mockResolvedValueOnce("/input").mockResolvedValueOnce("/output"),
      cancelDedupe: vi.fn().mockResolvedValue(true),
      listenToProgress: vi.fn().mockRejectedValue({
        code: "run_in_progress",
        message: "A dedupe run is already in progress",
      }),
      runDedupe: vi.fn(),
    };
    const { controller, render } = createHarness({ tauriClient });

    await controller.runDedupe();

    const latestState = render.mock.calls.at(-1)[0];
    expect(latestState.runStatus).toBe("error");
    expect(latestState.issues.items).toEqual(["issues.errors.runInProgress"]);
    expect(tauriClient.runDedupe).not.toHaveBeenCalled();
  });

  it("runs dedupe, subscribes to progress once, and stores warnings", async () => {
    let progressListener;
    let resolveFirstRun;
    let resolveSecondRun;
    const tauriClient = {
      isAvailable: () => true,
      chooseDirectory: vi
        .fn()
        .mockResolvedValueOnce("/input")
        .mockResolvedValueOnce("/output")
        .mockResolvedValueOnce("/input-again")
        .mockResolvedValueOnce("/output-again"),
      cancelDedupe: vi.fn().mockResolvedValue(true),
      listenToProgress: vi.fn().mockImplementation(async (listener) => {
        progressListener = listener;
        return () => {};
      }),
      runDedupe: vi
        .fn()
        .mockImplementationOnce(
          () =>
            new Promise((resolve) => {
              resolveFirstRun = resolve;
            }),
        )
        .mockImplementationOnce(
          () =>
            new Promise((resolve) => {
              resolveSecondRun = resolve;
            }),
        ),
    };
    const { controller, render } = createHarness({ tauriClient });

    const firstRunPromise = controller.runDedupe();
    await vi.waitFor(() => {
      expect(typeof progressListener).toBe("function");
    });
    const firstRunId = tauriClient.runDedupe.mock.calls[0][0].runId;
    progressListener({ runId: firstRunId, stage: "hash", current: 1, total: 3 });
    resolveFirstRun({
      totalImages: 4,
      uniqueImages: 2,
      duplicateImages: 2,
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
            path: "/input",
            duplicates: 2,
            wastedBytes: 50,
          },
        ],
      },
      outputDir: "/output",
      warnings: [
        "/input/a.png: permission denied",
        "Skipped /input/b.png because a similar image already exists in the output folder.",
      ],
      warningDetails: [
        {
          code: "file_issue",
          path: "/input/a.png",
          detail: "permission denied",
        },
        {
          code: "similar_image_in_output",
          path: "/input/b.png",
        },
      ],
    });
    await firstRunPromise;
    const secondRunPromise = controller.runDedupe();
    await vi.waitFor(() => {
      expect(typeof resolveSecondRun).toBe("function");
    });
    resolveSecondRun({
      totalImages: 4,
      uniqueImages: 2,
      duplicateImages: 2,
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
            path: "/input",
            duplicates: 2,
            wastedBytes: 50,
          },
        ],
      },
      outputDir: "/output",
      warnings: [
        "/input/a.png: permission denied",
        "Skipped /input/b.png because a similar image already exists in the output folder.",
      ],
      warningDetails: [
        {
          code: "file_issue",
          path: "/input/a.png",
          detail: "permission denied",
        },
        {
          code: "similar_image_in_output",
          path: "/input/b.png",
        },
      ],
    });
    await secondRunPromise;

    const latestState = render.mock.calls.at(-1)[0];
    expect(tauriClient.listenToProgress).toHaveBeenCalledTimes(1);
    expect(tauriClient.runDedupe).toHaveBeenNthCalledWith(1, {
      inputDir: "/input",
      outputDir: "/output",
      minSizeMb: 2.5,
      similarityThreshold: 10,
      filterExistingOutput: true,
      runId: expect.any(String),
      locale: "en",
    });
    expect(tauriClient.runDedupe).toHaveBeenNthCalledWith(2, {
      inputDir: "/input-again",
      outputDir: "/output-again",
      minSizeMb: 2.5,
      similarityThreshold: 10,
      filterExistingOutput: true,
      runId: expect.any(String),
      locale: "en",
    });
    expect(latestState.runStatus).toBe("success_with_warnings");
    expect(latestState.summary).toEqual({
      totalImages: 4,
      uniqueImages: 2,
      duplicateImages: 2,
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
            path: "/input",
            duplicates: 2,
            wastedBytes: 50,
          },
        ],
      },
      outputDir: "/output",
    });
    expect(latestState.issues.items).toEqual([
      "issues.warningItems.fileIssue",
      "issues.warningItems.similarImageInOutput",
    ]);
    expect(
      render.mock.calls.some(
        ([state]) =>
          state.progress.stage === "hash" &&
          state.progress.current === 1 &&
          state.progress.total === 3,
      ),
    ).toBe(true);
  });

  it("ignores stale progress events from a previous run id", async () => {
    let progressListener;
    let resolveFirstRun;
    let resolveSecondRun;
    const tauriClient = {
      isAvailable: () => true,
      chooseDirectory: vi
        .fn()
        .mockResolvedValueOnce("/input")
        .mockResolvedValueOnce("/output")
        .mockResolvedValueOnce("/input-2")
        .mockResolvedValueOnce("/output-2"),
      cancelDedupe: vi.fn().mockResolvedValue(true),
      listenToProgress: vi.fn().mockImplementation(async (listener) => {
        progressListener = listener;
        return () => {};
      }),
      runDedupe: vi
        .fn()
        .mockImplementationOnce(
          () =>
            new Promise((resolve) => {
              resolveFirstRun = resolve;
            }),
        )
        .mockImplementationOnce(
          () =>
            new Promise((resolve) => {
              resolveSecondRun = resolve;
            }),
        ),
    };
    const { controller, render } = createHarness({ tauriClient });

    const firstRunPromise = controller.runDedupe();
    await vi.waitFor(() => {
      expect(tauriClient.runDedupe).toHaveBeenCalledTimes(1);
    });
    const firstRunId = tauriClient.runDedupe.mock.calls[0][0].runId;
    resolveFirstRun({ warnings: [] });
    await firstRunPromise;

    const secondRunPromise = controller.runDedupe();
    await vi.waitFor(() => {
      expect(tauriClient.runDedupe).toHaveBeenCalledTimes(2);
    });
    const secondRunId = tauriClient.runDedupe.mock.calls[1][0].runId;
    const rendersBeforeStaleEvent = render.mock.calls.length;

    progressListener({ runId: firstRunId, stage: "hash", current: 1, total: 3 });

    expect(render).toHaveBeenCalledTimes(rendersBeforeStaleEvent);

    progressListener({ runId: secondRunId, stage: "hash", current: 2, total: 3 });

    expect(render.mock.calls.at(-1)[0].progress).toEqual({
      stage: "hash",
      current: 2,
      total: 3,
    });

    resolveSecondRun({ warnings: [] });
    await secondRunPromise;
  });

  it("ignores duplicate progress payloads that normalize to the current state", async () => {
    let progressListener;
    const tauriClient = {
      isAvailable: () => true,
      chooseDirectory: vi.fn().mockResolvedValueOnce("/input").mockResolvedValueOnce("/output"),
      cancelDedupe: vi.fn().mockResolvedValue(true),
      listenToProgress: vi.fn().mockImplementation(async (listener) => {
        progressListener = listener;
        return () => {};
      }),
      runDedupe: vi.fn().mockResolvedValue({ warnings: [] }),
    };
    const { controller, render } = createHarness({ tauriClient });

    await controller.runDedupe();
    const runId = tauriClient.runDedupe.mock.calls[0][0].runId;
    progressListener({ runId, stage: "hash", current: 1, total: 3 });
    const rendersAfterFirstHash = render.mock.calls.length;

    progressListener({ runId, stage: "hash", current: 1, total: 3 });

    expect(render).toHaveBeenCalledTimes(rendersAfterFirstHash);
  });

  it("ignores late progress payloads after the run has already completed", async () => {
    let progressListener;
    let resolveRun;
    const tauriClient = {
      isAvailable: () => true,
      chooseDirectory: vi.fn().mockResolvedValueOnce("/input").mockResolvedValueOnce("/output"),
      cancelDedupe: vi.fn().mockResolvedValue(true),
      listenToProgress: vi.fn().mockImplementation(async (listener) => {
        progressListener = listener;
        return () => {};
      }),
      runDedupe: vi.fn().mockImplementation(
        () =>
          new Promise((resolve) => {
            resolveRun = resolve;
          }),
      ),
    };
    const { controller, render } = createHarness({ tauriClient });

    const runPromise = controller.runDedupe();
    await vi.waitFor(() => {
      expect(typeof resolveRun).toBe("function");
    });
    resolveRun({ warnings: [] });
    await runPromise;
    const rendersAfterCompletion = render.mock.calls.length;
    const runId = tauriClient.runDedupe.mock.calls[0][0].runId;

    progressListener({ runId, stage: "hash", current: 1, total: 3 });

    expect(render).toHaveBeenCalledTimes(rendersAfterCompletion);
  });

  it("persists minimum size updates and uses them in subsequent runs", async () => {
    const settingsStore = createSettingsStore(1);
    const tauriClient = {
      isAvailable: () => true,
      chooseDirectory: vi.fn().mockResolvedValueOnce("/input").mockResolvedValueOnce("/output"),
      cancelDedupe: vi.fn().mockResolvedValue(true),
      listenToProgress: vi.fn().mockResolvedValue(() => {}),
      runDedupe: vi.fn().mockResolvedValue({ warnings: [] }),
    };
    const { controller } = createHarness({ settingsStore, tauriClient });

    controller.setMinSize(3.4);
    await controller.runDedupe();

    expect(settingsStore.getMinSizeMb()).toBe(3.4);
    expect(tauriClient.runDedupe).toHaveBeenCalledWith({
      inputDir: "/input",
      outputDir: "/output",
      minSizeMb: 3.4,
      similarityThreshold: 10,
      filterExistingOutput: true,
      runId: expect.any(String),
      locale: "en",
    });
  });

  it("persists similarity threshold updates and uses them in subsequent runs", async () => {
    const settingsStore = createSettingsStore(1, true, 10);
    const tauriClient = {
      isAvailable: () => true,
      chooseDirectory: vi.fn().mockResolvedValueOnce("/input").mockResolvedValueOnce("/output"),
      cancelDedupe: vi.fn().mockResolvedValue(true),
      listenToProgress: vi.fn().mockResolvedValue(() => {}),
      runDedupe: vi.fn().mockResolvedValue({ warnings: [] }),
    };
    const { controller } = createHarness({ settingsStore, tauriClient });

    controller.setSimilarityThreshold(6);
    await controller.runDedupe();

    expect(settingsStore.getSimilarityThreshold()).toBe(6);
    expect(tauriClient.runDedupe).toHaveBeenCalledWith({
      inputDir: "/input",
      outputDir: "/output",
      minSizeMb: 1,
      similarityThreshold: 6,
      filterExistingOutput: true,
      runId: expect.any(String),
      locale: "en",
    });
  });

  it("persists the existing-output filter setting and uses it in runs", async () => {
    const settingsStore = createSettingsStore(1, true);
    const tauriClient = {
      isAvailable: () => true,
      chooseDirectory: vi.fn().mockResolvedValueOnce("/input").mockResolvedValueOnce("/output"),
      cancelDedupe: vi.fn().mockResolvedValue(true),
      listenToProgress: vi.fn().mockResolvedValue(() => {}),
      runDedupe: vi.fn().mockResolvedValue({ warnings: [] }),
    };
    const { controller } = createHarness({ settingsStore, tauriClient });

    controller.setFilterExistingOutput(false);
    await controller.runDedupe();

    expect(settingsStore.getFilterExistingOutput()).toBe(false);
    expect(tauriClient.runDedupe).toHaveBeenCalledWith({
      inputDir: "/input",
      outputDir: "/output",
      minSizeMb: 1,
      similarityThreshold: 10,
      filterExistingOutput: false,
      runId: expect.any(String),
      locale: "en",
    });
  });

  it("persists the visual review setting", () => {
    const settingsStore = createSettingsStore(1, true, 10, true, true);
    const { controller, render } = createHarness({ settingsStore });

    controller.setVisualReviewEnabled(false);

    expect(settingsStore.getVisualReviewEnabled()).toBe(false);
    expect(render.mock.calls.at(-1)[0].visualReviewEnabled).toBe(false);
  });

  it("persists the similar-group export setting", () => {
    const settingsStore = createSettingsStore(1, true, 10, true, true);
    const { controller, render } = createHarness({ settingsStore });

    controller.setExportSimilarImageGroups(false);

    expect(settingsStore.getExportSimilarImageGroups()).toBe(false);
    expect(render.mock.calls.at(-1)[0].exportSimilarImageGroups).toBe(false);
  });

  it("skips the review screen and exports immediately when visual review is disabled", async () => {
    const settingsStore = createSettingsStore(2.5, true, 10, false, false);
    const tauriClient = {
      isAvailable: () => true,
      chooseDirectory: vi.fn().mockResolvedValueOnce("/input").mockResolvedValueOnce("/output"),
      cancelDedupe: vi.fn().mockResolvedValue(true),
      listenToProgress: vi.fn().mockResolvedValue(() => {}),
      prepareDedupeReview: vi.fn().mockResolvedValue({
        groups: [
          {
            groupId: "g1",
            images: ["/input/a.jpg", "/input/b.jpg"],
            suggested: "/input/a.jpg",
          },
        ],
        warnings: ["prepare warning"],
        warningDetails: [
          {
            code: "file_issue",
            path: "/input/a.jpg",
            detail: "prepare warning",
          },
        ],
      }),
      exportDedupeReview: vi.fn().mockResolvedValue({
        totalImages: 2,
        uniqueImages: 1,
        duplicateImages: 1,
        report: {
          storageSavedBytes: 10,
          storageSavedHuman: "10 B",
          summary: { totalImages: 2, uniqueImages: 1, duplicateImages: 1 },
          folderBreakdown: [],
        },
        outputDir: "/output",
        warnings: ["export warning"],
        warningDetails: [
          {
            code: "similar_image_in_output",
            path: "/input/b.jpg",
          },
        ],
      }),
    };
    const { controller, render } = createHarness({ settingsStore, tauriClient });

    await controller.runDedupe();

    expect(tauriClient.prepareDedupeReview).toHaveBeenCalledTimes(1);
    expect(tauriClient.exportDedupeReview).toHaveBeenCalledWith({
      outputDir: "/output",
      similarityThreshold: 10,
      filterExistingOutput: true,
      exportSimilarImageGroups: false,
      locale: "en",
      groups: [
        {
          groupId: "g1",
          images: ["/input/a.jpg", "/input/b.jpg"],
          suggested: "/input/a.jpg",
        },
      ],
      selections: {},
      runId: expect.any(String),
    });

    const latestState = render.mock.calls.at(-1)[0];
    expect(latestState.runStatus).toBe("success_with_warnings");
    expect(latestState.review.groups).toEqual([]);
    expect(latestState.summary.totalImages).toBe(2);
    expect(latestState.issues.items).toEqual([
      "issues.warningItems.fileIssue",
      "issues.warningItems.similarImageInOutput",
    ]);
  });

  it("resets settings to their defaults", () => {
    const settingsStore = createSettingsStore(2.5, false, 6, false, false);
    const { controller, render } = createHarness({ settingsStore });

    controller.resetSettings();

    expect(settingsStore.getMinSizeMb()).toBe(0);
    expect(settingsStore.getSimilarityThreshold()).toBe(10);
    expect(settingsStore.getFilterExistingOutput()).toBe(true);
    expect(settingsStore.getVisualReviewEnabled()).toBe(true);
    expect(settingsStore.getExportSimilarImageGroups()).toBe(true);

    const latestState = render.mock.calls.at(-1)[0];
    expect(latestState.minSizeMb).toBe(0);
    expect(latestState.similarityThreshold).toBe(10);
    expect(latestState.filterExistingOutput).toBe(true);
    expect(latestState.visualReviewEnabled).toBe(true);
    expect(latestState.exportSimilarImageGroups).toBe(true);
  });

  it("uses the currently selected locale in dedupe runs", async () => {
    const tauriClient = {
      isAvailable: () => true,
      chooseDirectory: vi.fn().mockResolvedValueOnce("/input").mockResolvedValueOnce("/output"),
      cancelDedupe: vi.fn().mockResolvedValue(true),
      listenToProgress: vi.fn().mockResolvedValue(() => {}),
      runDedupe: vi.fn().mockResolvedValue({ warnings: [] }),
    };
    const { controller } = createHarness({ tauriClient });

    controller.setLocale("he");
    await controller.runDedupe();

    expect(tauriClient.runDedupe).toHaveBeenCalledWith({
      inputDir: "/input",
      outputDir: "/output",
      minSizeMb: 2.5,
      similarityThreshold: 10,
      filterExistingOutput: true,
      runId: expect.any(String),
      locale: "he",
    });
  });

  it("ignores conflicting actions while a run is active", async () => {
    let resolveRun;
    const tauriClient = {
      isAvailable: () => true,
      chooseDirectory: vi.fn().mockResolvedValueOnce("/input").mockResolvedValueOnce("/output"),
      cancelDedupe: vi.fn().mockResolvedValue(true),
      listenToProgress: vi.fn().mockResolvedValue(() => {}),
      runDedupe: vi.fn().mockImplementation(
        () =>
          new Promise((resolve) => {
            resolveRun = resolve;
          }),
      ),
    };
    const { controller, localization, render, settingsStore } = createHarness({ tauriClient });

    const runPromise = controller.runDedupe();
    await vi.waitFor(() => {
      expect(tauriClient.runDedupe).toHaveBeenCalledTimes(1);
    });

    controller.openSettings();
    controller.toggleLanguageMenu();
    controller.setLocale("he");
    controller.setMinSize(4.2);
    controller.setSimilarityThreshold(6);
    controller.setFilterExistingOutput(false);

    let latestState = render.mock.calls.at(-1)[0];
    expect(latestState.runStatus).toBe("running");
    expect(latestState.view).toBe("main");
    expect(latestState.isLanguageMenuOpen).toBe(false);
    expect(latestState.locale).toBe("en");
    expect(settingsStore.getMinSizeMb()).toBe(2.5);
    expect(settingsStore.getSimilarityThreshold()).toBe(10);
    expect(settingsStore.getFilterExistingOutput()).toBe(true);
    expect(localization.getLocale()).toBe("en");

    resolveRun({ totalImages: 1, uniqueImages: 1, duplicateImages: 0, outputDir: "/output", warnings: [] });
    await runPromise;

    latestState = render.mock.calls.at(-1)[0];
    expect(latestState.runStatus).toBe("success");
    expect(latestState.summary.totalImages).toBe(1);
  });

  it("requests cancellation and renders a cancelled issue when the run aborts", async () => {
    let rejectRun;
    const tauriClient = {
      isAvailable: () => true,
      chooseDirectory: vi.fn().mockResolvedValueOnce("/input").mockResolvedValueOnce("/output"),
      cancelDedupe: vi.fn().mockResolvedValue(true),
      listenToProgress: vi.fn().mockResolvedValue(() => {}),
      runDedupe: vi.fn().mockImplementation(
        () =>
          new Promise((_, reject) => {
            rejectRun = reject;
          }),
      ),
    };
    const { controller, render } = createHarness({ tauriClient });

    const runPromise = controller.runDedupe();
    await vi.waitFor(() => {
      expect(tauriClient.runDedupe).toHaveBeenCalledTimes(1);
    });

    await expect(controller.toggleRun()).resolves.toBe(true);
    expect(tauriClient.cancelDedupe).toHaveBeenCalledTimes(1);

    rejectRun({ code: "cancelled", message: "Dedupe run cancelled" });
    await runPromise;

    const latestState = render.mock.calls.at(-1)[0];
    expect(latestState.runStatus).toBe("idle");
    expect(latestState.progress.stage).toBe("Cancelling");
    expect(latestState.issues.title).toBe("issues.runCancelledTitle");
    expect(latestState.issues.items).toEqual(["issues.errors.cancelled"]);
  });
});
