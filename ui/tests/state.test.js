import { describe, expect, it } from "vitest";

import { createInitialState, reduceAppState } from "../src/app/state.js";

describe("app state", () => {
  it("starts in the main idle view", () => {
    const state = createInitialState(2.5, 6, "en", false, false, false);

    expect(state.view).toBe("main");
    expect(state.runStatus).toBe("idle");
    expect(state.minSizeMb).toBe(2.5);
    expect(state.similarityThreshold).toBe(6);
    expect(state.filterExistingOutput).toBe(false);
    expect(state.visualReviewEnabled).toBe(false);
    expect(state.exportSimilarImageGroups).toBe(false);
    expect(state.progress).toEqual({
      stage: "Idle",
      current: 0,
      total: 1,
    });
    expect(state.issues.items).toEqual([]);
  });

  it("moves from settings to running main view when a run starts", () => {
    const settingsState = reduceAppState(createInitialState(), {
      type: "view/set",
      view: "settings",
    });

    const runningState = reduceAppState(settingsState, {
      type: "run/start",
    });

    expect(runningState.view).toBe("main");
    expect(runningState.runStatus).toBe("running");
    expect(runningState.issues).toEqual({
      title: "",
      hint: "",
      items: [],
    });
  });

  it("normalizes progress payloads with missing values", () => {
    const nextState = reduceAppState(createInitialState(), {
      type: "progress/set",
      payload: {},
    });

    expect(nextState.progress).toEqual({
      stage: "Working",
      current: 0,
      total: 1,
    });
  });

  it("preserves state identity for duplicate normalized progress updates", () => {
    const state = createInitialState();
    const nextState = reduceAppState(state, {
      type: "progress/set",
      payload: { stage: "Idle", current: 0, total: 1 },
    });

    expect(nextState).toBe(state);
  });

  it("stores warning issues and clears running state on finish", () => {
    const runningState = reduceAppState(createInitialState(), {
      type: "run/start",
    });

    const warningState = reduceAppState(runningState, {
      type: "issues/set",
      issues: {
        title: "2 warnings",
        hint: "Review skipped files and non-fatal problems.",
        items: ["warning 1", "warning 2"],
      },
    });
    const finishedState = reduceAppState(warningState, {
      type: "run/complete",
      status: "success_with_warnings",
    });

    expect(finishedState.runStatus).toBe("success_with_warnings");
    expect(finishedState.issues.title).toBe("2 warnings");
    expect(finishedState.issues.items).toHaveLength(2);
  });

  it("tracks locale changes and closes the language menu", () => {
    const withOpenMenu = reduceAppState(createInitialState(), {
      type: "languageMenu/toggle",
    });
    const localized = reduceAppState(withOpenMenu, {
      type: "locale/set",
      locale: "he",
    });

    expect(withOpenMenu.isLanguageMenuOpen).toBe(true);
    expect(localized.locale).toBe("he");
    expect(localized.isLanguageMenuOpen).toBe(false);
  });

  it("stores existing-output filter updates", () => {
    const nextState = reduceAppState(createInitialState(), {
      type: "filterExistingOutput/set",
      value: false,
    });

    expect(nextState.filterExistingOutput).toBe(false);
  });

  it("stores visual review setting updates", () => {
    const nextState = reduceAppState(createInitialState(), {
      type: "visualReviewEnabled/set",
      value: false,
    });

    expect(nextState.visualReviewEnabled).toBe(false);
  });

  it("stores similar-group export setting updates", () => {
    const nextState = reduceAppState(createInitialState(), {
      type: "exportSimilarImageGroups/set",
      value: false,
    });

    expect(nextState.exportSimilarImageGroups).toBe(false);
  });

  it("preserves dedupe report data in summary state", () => {
    const nextState = reduceAppState(createInitialState(), {
      type: "summary/set",
      summary: {
        totalImages: 4,
        uniqueImages: 2,
        duplicateImages: 2,
        outputDir: "/output",
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
      },
    });

    expect(nextState.summary.report).toEqual({
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
    });
  });

  it("stores similarity-threshold updates", () => {
    const nextState = reduceAppState(createInitialState(), {
      type: "similarityThreshold/set",
      value: 8,
    });

    expect(nextState.similarityThreshold).toBe(8);
  });
});
