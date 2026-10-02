import { beforeEach, describe, expect, it, vi } from "vitest";
import { loadAppWithTauri, loadHtml } from "./helpers/ui-test-helpers.js";

function createReviewResponse(overrides = {}) {
  return {
    groups: [
      {
        groupId: "g1",
        images: ["/input/a.jpg", "/input/b.jpg"],
        suggested: "/input/a.jpg",
      },
    ],
    warnings: [],
    warningDetails: [],
    ...overrides,
  };
}

function createSummaryResponse(overrides = {}) {
  return {
    totalImages: 10,
    uniqueImages: 7,
    duplicateImages: 3,
    report: {
      storageSavedBytes: 262144000,
      storageSavedHuman: "250 MB",
      summary: {
        totalImages: 10,
        uniqueImages: 7,
        duplicateImages: 3,
      },
      folderBreakdown: [
        {
          path: "/input/favorites",
          duplicates: 3,
          wastedBytes: 262144000,
        },
      ],
    },
    outputDir: "/output",
    warnings: [],
    warningDetails: [],
    ...overrides,
  };
}

describe("Aba Avner UI", () => {
  beforeEach(() => {
    window.__TAURI__ = undefined;
    window.__ABA_AVNER_TESTING__ = false;
    window.__ABA_AVNER_TEST_API__ = undefined;
    window.localStorage.clear();
    loadHtml();
  });

  it("shows a helpful message when Tauri APIs are missing", async () => {
    await loadAppWithTauri(undefined);

    const startButton = document.getElementById("startButton");
    const issuesBadge = document.getElementById("issuesBadge");
    const issuesList = document.getElementById("issuesList");

    startButton.click();

    expect(issuesBadge.textContent).toBe("Tauri unavailable");
    expect(issuesList.textContent).toContain("Tauri APIs not available.");
  });

  it("keeps the issues panel hidden on initialization", async () => {
    await loadAppWithTauri(undefined);

    const issuesPanel = document.getElementById("issuesPanel");
    expect(issuesPanel.hidden).toBe(true);
  });

  it("applies persisted localization on initialization", async () => {
    window.localStorage.setItem("aba_avner.locale", "he");

    await loadAppWithTauri(undefined);

    expect(document.documentElement.lang).toBe("he");
    expect(document.documentElement.dir).toBe("rtl");
    expect(document.getElementById("startButton").textContent).toBe("בחירת תיקיות והפעלה");
    expect(document.getElementById("settingsTitle").textContent).toBe("הגדרות");
  });

  it("applies persisted minimum image size on initialization", async () => {
    window.localStorage.setItem("aba_avner.min_size_mb", "3.4");

    await loadAppWithTauri(undefined);

    expect(document.getElementById("minSizeValue").textContent).toBe("3.4 MB");
    expect(document.getElementById("minSizeSlider").value).toBe("3.4");
  });

  it("applies persisted similarity threshold on initialization", async () => {
    window.localStorage.setItem("aba_avner.similarity_threshold", "6");

    await loadAppWithTauri(undefined);

    expect(document.getElementById("similarityThresholdValue").textContent).toBe("6");
    expect(document.getElementById("similarityThresholdInput").value).toBe("6");
    expect(document.getElementById("similarityThresholdMinLabel").textContent).toBe(
      "Stricter match",
    );
    expect(document.getElementById("similarityThresholdMaxLabel").textContent).toBe(
      "Looser match",
    );
  });

  it("applies persisted toggle settings on initialization", async () => {
    window.localStorage.setItem("aba_avner.visual_review_enabled", "false");
    window.localStorage.setItem("aba_avner.export_similar_image_groups", "false");

    await loadAppWithTauri(undefined);

    expect(document.getElementById("visualReviewToggle").checked).toBe(false);
    expect(document.getElementById("exportGroupsToggle").checked).toBe(false);
  });

  it("resets settings-card values back to defaults", async () => {
    window.localStorage.setItem("aba_avner.min_size_mb", "3.4");
    window.localStorage.setItem("aba_avner.similarity_threshold", "6");
    window.localStorage.setItem("aba_avner.filter_existing_output", "false");
    window.localStorage.setItem("aba_avner.visual_review_enabled", "false");
    window.localStorage.setItem("aba_avner.export_similar_image_groups", "false");

    await loadAppWithTauri(undefined);

    document.getElementById("resetSettingsButton").click();

    expect(document.getElementById("minSizeSlider").value).toBe("0");
    expect(document.getElementById("minSizeValue").textContent).toBe("0 MB");
    expect(document.getElementById("similarityThresholdInput").value).toBe("10");
    expect(document.getElementById("similarityThresholdValue").textContent).toBe("10");
    expect(document.getElementById("existingOutputToggle").checked).toBe(true);
    expect(document.getElementById("visualReviewToggle").checked).toBe(true);
    expect(document.getElementById("exportGroupsToggle").checked).toBe(true);
    expect(window.localStorage.getItem("aba_avner.min_size_mb")).toBe("0");
    expect(window.localStorage.getItem("aba_avner.similarity_threshold")).toBe("10");
    expect(window.localStorage.getItem("aba_avner.filter_existing_output")).toBe("true");
    expect(window.localStorage.getItem("aba_avner.visual_review_enabled")).toBe("true");
    expect(window.localStorage.getItem("aba_avner.export_similar_image_groups")).toBe("true");
  });

  it("switches locale from the header dropdown and persists it", async () => {
    await loadAppWithTauri(undefined);

    const languageButton = document.getElementById("languageButton");
    const languageMenu = document.getElementById("languageMenu");
    const localeHeButton = document.getElementById("localeHeButton");

    languageButton.click();
    expect(languageMenu.hidden).toBe(false);

    localeHeButton.click();

    expect(document.documentElement.lang).toBe("he");
    expect(document.documentElement.dir).toBe("rtl");
    expect(document.getElementById("startButton").textContent).toBe("בחירת תיקיות והפעלה");
    expect(window.localStorage.getItem("aba_avner.locale")).toBe("he");
    expect(languageMenu.hidden).toBe(true);
    expect(languageButton.textContent).toContain("🇮🇱");
  });

  it("localizes review actions after switching locale while review is visible", async () => {
    const dialogOpen = vi
      .fn()
      .mockResolvedValueOnce("/input")
      .mockResolvedValueOnce("/output");

    const coreInvoke = vi.fn().mockImplementation((command) => {
      if (command === "prepare_dedupe_review") {
        return createReviewResponse();
      }

      throw new Error(`Unexpected command: ${command}`);
    });

    await loadAppWithTauri({
      dialog: { open: dialogOpen },
      core: { invoke: coreInvoke },
      event: { listen: vi.fn().mockResolvedValue(() => {}) },
    });

    document.getElementById("startButton").click();
    await vi.waitFor(() => {
      expect(document.getElementById("reviewPanel").hidden).toBe(false);
    });

    expect(document.getElementById("reviewApplyButton").textContent).toBe("Apply & Export");

    document.getElementById("languageButton").click();
    document.getElementById("localeHeButton").click();

    expect(document.getElementById("reviewApplyButton").textContent).toBe("החל וייצא");
    expect(document.getElementById("reviewCancelButton").textContent).toBe("ביטול");
  });

  it("moves focus into settings and back to the trigger when closed", async () => {
    await loadAppWithTauri(undefined);

    const settingsButton = document.getElementById("settingsButton");
    const backButton = document.getElementById("backButton");
    const minSizeSlider = document.getElementById("minSizeSlider");

    settingsButton.focus();
    settingsButton.click();

    expect(document.getElementById("cardStage").dataset.view).toBe("settings");
    expect(document.activeElement).toBe(minSizeSlider);

    backButton.click();

    expect(document.getElementById("cardStage").dataset.view).toBe("main");
    expect(document.activeElement).toBe(settingsButton);
  });

  it("supports keyboard navigation in the language menu and restores focus on escape", async () => {
    await loadAppWithTauri(undefined);

    const languageButton = document.getElementById("languageButton");
    const localeEnButton = document.getElementById("localeEnButton");
    const localeHeButton = document.getElementById("localeHeButton");

    languageButton.focus();
    languageButton.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));

    expect(document.getElementById("languageMenu").hidden).toBe(false);
    expect(document.activeElement).toBe(localeEnButton);

    localeEnButton.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    expect(document.activeElement).toBe(localeHeButton);

    localeHeButton.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(document.getElementById("languageMenu").hidden).toBe(true);
    expect(document.activeElement).toBe(languageButton);
  });

  it("updates progress bar from progress events", async () => {
    let progressCallback = null;
    let resolvePrepare;
    let resolveExport;

    const dialogOpen = vi
      .fn()
      .mockResolvedValueOnce("/input")
      .mockResolvedValueOnce("/output");

    const coreInvoke = vi.fn().mockImplementation((command) => {
      if (command === "prepare_dedupe_review") {
        return new Promise((resolve) => {
          resolvePrepare = resolve;
        });
      }

      if (command === "export_dedupe_review") {
        return new Promise((resolve) => {
          resolveExport = resolve;
        });
      }

      return Promise.resolve(true);
    });

    const eventListen = vi.fn().mockImplementation(async (_event, callback) => {
      progressCallback = callback;
      return () => {};
    });

    await loadAppWithTauri({
      dialog: { open: dialogOpen },
      core: { invoke: coreInvoke },
      event: { listen: eventListen },
    });

    const progressStage = document.getElementById("progressStage");
    const progressPercent = document.getElementById("progressPercent");
    const progressBar = document.getElementById("progressBar");

    const runPromise = window.__ABA_AVNER_TEST_API__.runDedupe();
    await vi.waitFor(() => {
      expect(document.getElementById("startButton").textContent).toBe("Cancel run");
      expect(progressStage.textContent).toBe("Starting");
    });
    const firstRunConfig = coreInvoke.mock.calls.at(-1)[1].config;

    expect(progressStage.textContent).toBe("Starting");
    expect(progressPercent.textContent).toBe("0%");
    expect(progressBar.style.width).toBe("0%");
    expect(document.getElementById("startButton").textContent).toBe("Cancel run");
    expect(document.getElementById("summaryPanel").hidden).toBe(true);

    progressCallback({
      payload: { runId: firstRunConfig.runId, stage: "scan", current: 5, total: 10 },
    });

    expect(progressStage.textContent).toBe("Scanning");
    expect(progressPercent.textContent).toBe("10%");
    expect(progressBar.style.width).toBe("10%");

    resolvePrepare(createReviewResponse());
    await runPromise;

    expect(document.getElementById("reviewPanel").hidden).toBe(false);
    expect(document.getElementById("reviewApplyButton").textContent).toBe("Apply & Export");
    expect(progressPercent.textContent).toBe("60%");

    const exportPromise = window.__ABA_AVNER_TEST_API__.applyReview();
    await vi.waitFor(() => {
      expect(coreInvoke).toHaveBeenCalledWith("export_dedupe_review", expect.any(Object));
    });
    resolveExport(createSummaryResponse());
    await exportPromise;

    expect(document.getElementById("startButton").textContent).toBe("Choose folders & Run");
    expect(document.getElementById("summaryPanel").hidden).toBe(false);
    expect(document.getElementById("summaryBadge").textContent).toBe("Run summary");
    expect(document.getElementById("summaryTotalValue").textContent).toBe("10");
    expect(document.getElementById("summaryDuplicateValue").textContent).toBe("3");
    expect(document.getElementById("summaryStorageSavedValue").textContent).toBe("250 MB");
    expect(document.getElementById("summaryToggleButton").textContent).toBe("Show details");
    expect(document.getElementById("summaryDetails").hidden).toBe(true);

    document.getElementById("summaryToggleButton").click();

    expect(document.getElementById("summaryToggleButton").textContent).toBe("Hide details");
    expect(document.getElementById("summaryDetails").hidden).toBe(false);
    expect(document.getElementById("summaryBreakdownList").children).toHaveLength(1);
    expect(document.getElementById("summaryBreakdownList").textContent).toContain("/input/favorites");
    expect(progressPercent.textContent).toBe("100%");
    expect(progressBar.style.width).toBe("100%");
  });

  it("passes the minimum image size from the slider to the backend", async () => {
    const dialogOpen = vi
      .fn()
      .mockResolvedValueOnce("/input")
      .mockResolvedValueOnce("/output");

    const coreInvoke = vi.fn().mockResolvedValue(createReviewResponse());

    await loadAppWithTauri({
      dialog: { open: dialogOpen },
      core: { invoke: coreInvoke },
      event: { listen: vi.fn().mockResolvedValue(() => {}) },
    });

    const slider = document.getElementById("minSizeSlider");
    const valueLabel = document.getElementById("minSizeValue");

    slider.value = "2.5";
    slider.dispatchEvent(new Event("input"));

    expect(valueLabel.textContent).toBe("2.5 MB");

    await window.__ABA_AVNER_TEST_API__.runDedupe();

    expect(coreInvoke).toHaveBeenCalledWith("prepare_dedupe_review", {
      config: expect.objectContaining({
        minImageSizeBytes: Math.round(2.5 * 1024 * 1024),
      }),
    });
    expect(window.localStorage.getItem("aba_avner.min_size_mb")).toBe("2.5");
  });

  it("passes the similarity threshold from settings to the backend", async () => {
    const dialogOpen = vi
      .fn()
      .mockResolvedValueOnce("/input")
      .mockResolvedValueOnce("/output");

    const coreInvoke = vi.fn().mockResolvedValue(createReviewResponse());

    await loadAppWithTauri({
      dialog: { open: dialogOpen },
      core: { invoke: coreInvoke },
      event: { listen: vi.fn().mockResolvedValue(() => {}) },
    });

    const input = document.getElementById("similarityThresholdInput");
    const valueLabel = document.getElementById("similarityThresholdValue");

    input.value = "6";
    input.dispatchEvent(new Event("input"));

    expect(valueLabel.textContent).toBe("6");

    await window.__ABA_AVNER_TEST_API__.runDedupe();

    expect(coreInvoke).toHaveBeenCalledWith("prepare_dedupe_review", {
      config: expect.objectContaining({
        similarityThreshold: 6,
      }),
    });
    expect(window.localStorage.getItem("aba_avner.similarity_threshold")).toBe("6");
  });

  it("returns to the main card before running dedupe from settings", async () => {
    const dialogOpen = vi
      .fn()
      .mockResolvedValueOnce("/input")
      .mockResolvedValueOnce("/output");

    const coreInvoke = vi.fn().mockResolvedValue(createReviewResponse());

    await loadAppWithTauri({
      dialog: { open: dialogOpen },
      core: { invoke: coreInvoke },
      event: { listen: vi.fn().mockResolvedValue(() => {}) },
    });

    document.getElementById("settingsButton").click();

    await window.__ABA_AVNER_TEST_API__.runDedupe();

    expect(document.getElementById("cardStage").dataset.view).toBe("main");
  });

  it("disables conflicting controls while a run is active", async () => {
    let resolveRun;
    const dialogOpen = vi
      .fn()
      .mockResolvedValueOnce("/input")
      .mockResolvedValueOnce("/output");
    const coreInvoke = vi.fn().mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveRun = resolve;
        }),
    );

    await loadAppWithTauri({
      dialog: { open: dialogOpen },
      core: { invoke: coreInvoke },
      event: { listen: vi.fn().mockResolvedValue(() => {}) },
    });

    const runPromise = window.__ABA_AVNER_TEST_API__.runDedupe();
    await vi.waitFor(() => {
      expect(coreInvoke).toHaveBeenCalledTimes(1);
    });

    expect(document.getElementById("startButton").disabled).toBe(false);
    expect(document.getElementById("startButton").textContent).toBe("Cancel run");
    expect(document.getElementById("settingsButton").disabled).toBe(true);
    expect(document.getElementById("languageButton").disabled).toBe(true);
    expect(document.getElementById("backButton").disabled).toBe(true);
    expect(document.getElementById("resetSettingsButton").disabled).toBe(true);
    expect(document.getElementById("minSizeSlider").disabled).toBe(true);
    expect(document.getElementById("similarityThresholdInput").disabled).toBe(true);

    resolveRun({
      totalImages: 1,
      uniqueImages: 1,
      duplicateImages: 0,
      outputDir: "/output",
      warnings: [],
    });
    await runPromise;

    expect(document.getElementById("startButton").disabled).toBe(false);
    expect(document.getElementById("settingsButton").disabled).toBe(false);
    expect(document.getElementById("languageButton").disabled).toBe(false);
    expect(document.getElementById("backButton").disabled).toBe(false);
    expect(document.getElementById("resetSettingsButton").disabled).toBe(false);
    expect(document.getElementById("minSizeSlider").disabled).toBe(false);
    expect(document.getElementById("similarityThresholdInput").disabled).toBe(false);
  });

  it("cancels an in-flight run from the main action and renders a cancelled state", async () => {
    let rejectRun;
    const dialogOpen = vi
      .fn()
      .mockResolvedValueOnce("/input")
      .mockResolvedValueOnce("/output");
    const coreInvoke = vi.fn().mockImplementation((command) => {
      if (command === "cancel_dedupe") {
        return Promise.resolve(true);
      }

      return new Promise((_, reject) => {
        rejectRun = reject;
      });
    });

    await loadAppWithTauri({
      dialog: { open: dialogOpen },
      core: { invoke: coreInvoke },
      event: { listen: vi.fn().mockResolvedValue(() => {}) },
    });

    const runPromise = window.__ABA_AVNER_TEST_API__.runDedupe();
    await vi.waitFor(() => {
      expect(coreInvoke).toHaveBeenCalledWith("prepare_dedupe_review", expect.any(Object));
    });

    document.getElementById("startButton").click();

    expect(coreInvoke).toHaveBeenCalledWith("cancel_dedupe");
    expect(document.getElementById("progressStage").textContent).toBe("Cancelling");

    rejectRun({ code: "cancelled", message: "Dedupe run cancelled" });
    await runPromise;

    expect(document.getElementById("issuesBadge").textContent).toBe("Run cancelled");
    expect(document.getElementById("issuesList").textContent).toContain("The run was cancelled.");
    expect(document.getElementById("startButton").textContent).toBe("Choose folders & Run");
  });

  it("reuses a single progress listener and clears cancelled state before the next run", async () => {
    let rejectFirstRun;
    let resolveSecondPrepare;
    let resolveSecondExport;
    const dialogOpen = vi
      .fn()
      .mockResolvedValueOnce("/input-a")
      .mockResolvedValueOnce("/output-a")
      .mockResolvedValueOnce("/input-b")
      .mockResolvedValueOnce("/output-b");
    const eventListen = vi.fn().mockImplementation(async (_event, callback) => {
      window.__ABA_AVNER_PROGRESS_CALLBACK__ = callback;
      return () => {};
    });
    const coreInvoke = vi.fn().mockImplementation((command) => {
      if (command === "cancel_dedupe") {
        return Promise.resolve(true);
      }

      if (!rejectFirstRun) {
        return new Promise((_, reject) => {
          rejectFirstRun = reject;
        });
      }

      if (!resolveSecondPrepare) {
        return new Promise((resolve) => {
          resolveSecondPrepare = resolve;
        });
      }

      return new Promise((resolve) => {
        resolveSecondExport = resolve;
      });
    });

    await loadAppWithTauri({
      dialog: { open: dialogOpen },
      core: { invoke: coreInvoke },
      event: { listen: eventListen },
    });

    const firstRunPromise = window.__ABA_AVNER_TEST_API__.runDedupe();
    await vi.waitFor(() => {
      expect(coreInvoke).toHaveBeenCalledWith("prepare_dedupe_review", expect.any(Object));
    });
    const firstRunConfig = coreInvoke.mock.calls.at(-1)[1].config;

    window.__ABA_AVNER_PROGRESS_CALLBACK__?.({
      payload: { runId: firstRunConfig.runId, stage: "scan", current: 3, total: 9 },
    });
    expect(document.getElementById("progressStage").textContent).toBe("Scanning");
    expect(document.getElementById("progressPercent").textContent).toBe("7%");

    document.getElementById("startButton").click();
    rejectFirstRun({ code: "cancelled", message: "Dedupe run cancelled" });
    await firstRunPromise;

    expect(eventListen).toHaveBeenCalledTimes(1);
    expect(document.getElementById("summaryPanel").hidden).toBe(true);
    expect(document.getElementById("issuesBadge").textContent).toBe("Run cancelled");
    expect(document.getElementById("issuesList").textContent).toContain("The run was cancelled.");

    const secondRunPromise = window.__ABA_AVNER_TEST_API__.runDedupe();
    await vi.waitFor(() => {
      expect(coreInvoke).toHaveBeenCalledTimes(3);
    });
    const secondRunConfig = coreInvoke.mock.calls.at(-1)[1].config;

    expect(eventListen).toHaveBeenCalledTimes(1);
    expect(document.getElementById("summaryPanel").hidden).toBe(true);
    expect(document.getElementById("issuesPanel").hidden).toBe(true);
    expect(document.getElementById("progressStage").textContent).toBe("Starting");
    expect(document.getElementById("progressPercent").textContent).toBe("0%");

    window.__ABA_AVNER_PROGRESS_CALLBACK__?.({
      payload: { runId: secondRunConfig.runId, stage: "copy", current: 2, total: 5 },
    });
    expect(document.getElementById("progressStage").textContent).toBe("Copying");
    expect(document.getElementById("progressPercent").textContent).toBe("88%");

    resolveSecondPrepare(createReviewResponse());
    await secondRunPromise;

    const exportPromise = window.__ABA_AVNER_TEST_API__.applyReview();
    await vi.waitFor(() => {
      expect(coreInvoke).toHaveBeenCalledWith("export_dedupe_review", expect.any(Object));
    });
    resolveSecondExport(
      createSummaryResponse({
        totalImages: 5,
        uniqueImages: 3,
        duplicateImages: 2,
        outputDir: "/output-b",
        report: {
          storageSavedBytes: 262144000,
          storageSavedHuman: "250 MB",
          summary: {
            totalImages: 5,
            uniqueImages: 3,
            duplicateImages: 2,
          },
          folderBreakdown: [],
        },
      }),
    );
    await exportPromise;

    expect(document.getElementById("summaryPanel").hidden).toBe(false);
    expect(document.getElementById("summaryTotalValue").textContent).toBe("5");
    expect(document.getElementById("summaryOutput").textContent).toContain("/output-b");
    expect(document.getElementById("summaryDetails").hidden).toBe(true);
    expect(document.getElementById("issuesPanel").hidden).toBe(true);
  });

  it("renders a scrollable warning list when the backend returns many warnings", async () => {
    const dialogOpen = vi
      .fn()
      .mockResolvedValueOnce("/input")
      .mockResolvedValueOnce("/output");

    const warnings = Array.from({ length: 18 }, (_, index) => `warning ${index + 1}`);
    const coreInvoke = vi.fn().mockResolvedValue({
      totalImages: 40,
      uniqueImages: 21,
      duplicateImages: 19,
      warnings,
      warningDetails: warnings.map((warning, index) => ({
        code: "unknown_warning",
        path: `/input/${index + 1}.png`,
        detail: warning,
      })),
    });

    await loadAppWithTauri({
      dialog: { open: dialogOpen },
      core: { invoke: coreInvoke },
      event: { listen: vi.fn().mockResolvedValue(() => {}) },
    });

    await window.__ABA_AVNER_TEST_API__.runDedupe();

    const issuesPanel = document.getElementById("issuesPanel");
    const issuesBadge = document.getElementById("issuesBadge");
    const issuesList = document.getElementById("issuesList");

    expect(issuesPanel.hidden).toBe(false);
    expect(issuesBadge.textContent).toBe("18 warnings");
    expect(issuesList.children).toHaveLength(18);
    expect(issuesList.textContent).toContain("warning 18");
  });

  it("renders a grouped-summary note when output filtering changes the copied count", async () => {
    const dialogOpen = vi
      .fn()
      .mockResolvedValueOnce("/input")
      .mockResolvedValueOnce("/output");

    const coreInvoke = vi
      .fn()
      .mockResolvedValueOnce(createReviewResponse())
      .mockResolvedValueOnce({
        totalImages: 12,
        uniqueImages: 4,
        duplicateImages: 8,
        outputDir: "/output",
        report: {
          storageSavedBytes: 3221225472,
          storageSavedHuman: "3.0 GB",
          summary: {
            totalImages: 12,
            uniqueImages: 6,
            duplicateImages: 6,
          },
          folderBreakdown: [],
        },
        warnings: [],
      });

    await loadAppWithTauri({
      dialog: { open: dialogOpen },
      core: { invoke: coreInvoke },
      event: { listen: vi.fn().mockResolvedValue(() => {}) },
    });

    await window.__ABA_AVNER_TEST_API__.runDedupe();
    await window.__ABA_AVNER_TEST_API__.applyReview();

    expect(document.getElementById("summaryDetails").hidden).toBe(true);

    document.getElementById("summaryToggleButton").click();

    expect(document.getElementById("summaryReportNote").hidden).toBe(false);
    expect(document.getElementById("summaryReportNote").textContent).toContain(
      "Grouping found 6 representatives before output-folder filtering.",
    );
    expect(document.getElementById("summaryBreakdownEmpty").hidden).toBe(false);
    expect(document.getElementById("summaryBreakdownEmpty").textContent).toBe(
      "No folder hotspots for this run.",
    );
  });

  it("shows a failure panel when the backend invocation rejects", async () => {
    const dialogOpen = vi
      .fn()
      .mockResolvedValueOnce("/input")
      .mockResolvedValueOnce("/output");

    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {});
    const coreInvoke = vi.fn().mockRejectedValue(new Error("disk full"));

    await loadAppWithTauri({
      dialog: { open: dialogOpen },
      core: { invoke: coreInvoke },
      event: { listen: vi.fn().mockResolvedValue(() => {}) },
    });

    await window.__ABA_AVNER_TEST_API__.runDedupe();

    const issuesBadge = document.getElementById("issuesBadge");
    const issuesList = document.getElementById("issuesList");

    expect(issuesBadge.textContent).toBe("Run failed");
    expect(issuesList.children).toHaveLength(1);
    expect(issuesList.textContent).toContain(
      "Unexpected backend error (unexpected_tauri_error:Error). Check developer logs.",
    );
    consoleError.mockRestore();
  });

  it("maps known structured backend errors to localized copy", async () => {
    const dialogOpen = vi
      .fn()
      .mockResolvedValueOnce("/input")
      .mockResolvedValueOnce("/output");

    const coreInvoke = vi.fn().mockRejectedValue({
      code: "invalid_output_dir",
      message: "Invalid output directory: /output",
    });

    await loadAppWithTauri({
      dialog: { open: dialogOpen },
      core: { invoke: coreInvoke },
      event: { listen: vi.fn().mockResolvedValue(() => {}) },
    });

    await window.__ABA_AVNER_TEST_API__.runDedupe();

    const issuesBadge = document.getElementById("issuesBadge");
    const issuesList = document.getElementById("issuesList");

    expect(issuesBadge.textContent).toBe("Run failed");
    expect(issuesList.children).toHaveLength(1);
    expect(issuesList.textContent).toContain("The selected output folder is not usable.");
  });

  it("localizes known structured warnings and keeps raw fallback for unknown ones", async () => {
    const dialogOpen = vi
      .fn()
      .mockResolvedValueOnce("/input")
      .mockResolvedValueOnce("/output");

    const coreInvoke = vi.fn().mockResolvedValue({
      totalImages: 2,
      uniqueImages: 1,
      duplicateImages: 1,
      warnings: [
        "/input/a.png: permission denied",
        "raw backend warning",
      ],
      warningDetails: [
        {
          code: "similar_image_in_output",
          path: "/input/a.png",
        },
        {
          code: "unknown_warning",
          path: "/input/b.png",
          detail: "raw backend warning",
        },
      ],
    });

    await loadAppWithTauri({
      dialog: { open: dialogOpen },
      core: { invoke: coreInvoke },
      event: { listen: vi.fn().mockResolvedValue(() => {}) },
    });

    await window.__ABA_AVNER_TEST_API__.runDedupe();

    const issuesList = document.getElementById("issuesList");

    expect(issuesList.children).toHaveLength(2);
    expect(issuesList.textContent).toContain(
      "Skipped /input/a.png because a similar image already exists in the output folder.",
    );
    expect(issuesList.textContent).toContain("raw backend warning");
  });
});
