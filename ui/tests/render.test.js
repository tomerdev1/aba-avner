import { describe, expect, it } from "vitest";

import { renderApp } from "../src/app/render.js";
import { createRenderHarness } from "./helpers/ui-test-helpers.js";

describe("app render", () => {
  it("renders localized static copy and locale controls", () => {
    const { elements, localization } = createRenderHarness("he");

    renderApp(
      {
        view: "main",
        runStatus: "idle",
        minSizeMb: 0,
        similarityThreshold: 10,
        filterExistingOutput: true,
        locale: "he",
        isLanguageMenuOpen: true,
        isSummaryExpanded: false,
        progress: { stage: "Idle", current: 0, total: 1 },
        issues: { title: "", hint: "", items: [] },
      },
      elements,
      localization,
    );

    expect(elements.appEyebrow.textContent).toBe("מסיר כפילויות חזותיות");
    expect(elements.startButton.textContent).toBe("בחירת תיקיות והפעלה");
    expect(elements.languageButton.textContent).toBe("🇮🇱");
    expect(elements.languageButton.getAttribute("aria-expanded")).toBe("true");
    expect(elements.languageMenu.hidden).toBe(false);
    expect(elements.languageMenu.getAttribute("aria-hidden")).toBe("false");
    expect(elements.localeHeButton.getAttribute("aria-pressed")).toBe("true");
    expect(elements.localeHeButton.getAttribute("tabindex")).toBe("0");
    expect(elements.localeEnButton.getAttribute("tabindex")).toBe("0");
    expect(elements.localeEnButton.textContent).toContain("English");
  });

  it("renders settings view, running state, and localized progress labels", () => {
    const { elements, localization } = createRenderHarness("en");

    renderApp(
      {
        view: "settings",
        runStatus: "running",
        minSizeMb: 3.4,
        similarityThreshold: 6,
        filterExistingOutput: false,
        visualReviewEnabled: false,
        exportSimilarImageGroups: false,
        locale: "en",
        isLanguageMenuOpen: false,
        isSummaryExpanded: false,
        progress: { stage: "Starting", current: 0, total: 1 },
        summary: {
          totalImages: null,
          uniqueImages: null,
          duplicateImages: null,
          outputDir: "",
        },
        issues: { title: "", hint: "", items: [] },
      },
      elements,
      localization,
    );

    expect(elements.cardStage.dataset.view).toBe("settings");
    expect(elements.settingsCard.getAttribute("aria-hidden")).toBe("false");
    expect(elements.startButton.disabled).toBe(false);
    expect(elements.startButton.textContent).toBe("Cancel run");
    expect(elements.settingsButton.disabled).toBe(true);
    expect(elements.languageButton.disabled).toBe(true);
    expect(elements.backButton.disabled).toBe(true);
    expect(elements.resetSettingsButton.disabled).toBe(true);
    expect(elements.minSizeSlider.disabled).toBe(true);
    expect(elements.mainCard.getAttribute("aria-busy")).toBe("true");
    expect(elements.progressStage.textContent).toBe("Starting");
    expect(elements.progressPercent.textContent).toBe("0%");
    expect(elements.progressBar.getAttribute("aria-valuenow")).toBe("0");
    expect(elements.progressBar.getAttribute("aria-label")).toBe("Dedupe progress");
    expect(elements.minSizeSlider.value).toBe("3.4");
    expect(elements.minSizeSlider.getAttribute("aria-describedby")).toBe("minSizeDescription minSizeValue");
    expect(elements.minSizeSlider.getAttribute("aria-valuenow")).toBe("3.4");
    expect(elements.minSizeSlider.getAttribute("aria-valuetext")).toBe("3.4 MB");
    expect(elements.minSizeValue.textContent).toBe("3.4 MB");
    expect(elements.similarityThresholdLabel.textContent).toBe("Similarity threshold");
    expect(elements.similarityThresholdInput.disabled).toBe(true);
    expect(elements.similarityThresholdInput.value).toBe("6");
    expect(elements.similarityThresholdInput.getAttribute("aria-describedby")).toBe(
      "similarityThresholdDescription similarityThresholdValue",
    );
    expect(elements.similarityThresholdValue.textContent).toBe("6");
    expect(elements.similarityThresholdMinLabel.textContent).toBe("Stricter match");
    expect(elements.similarityThresholdMaxLabel.textContent).toBe("Looser match");
    expect(elements.resetSettingsButton.textContent).toBe("Reset to defaults");
    expect(elements.existingOutputToggle.disabled).toBe(true);
    expect(elements.existingOutputToggle.checked).toBe(false);
    expect(elements.existingOutputLabel.textContent).toBe("Skip images already in output");
    expect(elements.visualReviewToggle.disabled).toBe(true);
    expect(elements.visualReviewToggle.checked).toBe(false);
    expect(elements.visualReviewLabel.textContent).toBe("Visual duplicate review step");
    expect(elements.exportGroupsToggle.disabled).toBe(true);
    expect(elements.exportGroupsToggle.checked).toBe(false);
    expect(elements.exportGroupsLabel.textContent).toBe('Export "similar image groups" folder');
    expect(elements.summaryPanel.hidden).toBe(true);
  });

  it("localizes known runtime progress stage codes", () => {
    const { elements, localization } = createRenderHarness("he");

    renderApp(
      {
        view: "main",
        runStatus: "running",
        minSizeMb: 0,
        similarityThreshold: 10,
        filterExistingOutput: true,
        locale: "he",
        isLanguageMenuOpen: false,
        isSummaryExpanded: false,
        progress: { stage: "hash", current: 1, total: 3 },
        summary: {
          totalImages: null,
          uniqueImages: null,
          duplicateImages: null,
          outputDir: "",
        },
        issues: { title: "", hint: "", items: [] },
      },
      elements,
      localization,
    );

    expect(elements.progressStage.textContent).toBe("מחשב hashes");
    expect(elements.progressPercent.textContent).toBe("27%");
  });

  it("localizes the cancelling stage and swaps the main action to cancel", () => {
    const { elements, localization } = createRenderHarness("he");

    renderApp(
      {
        view: "main",
        runStatus: "cancelling",
        minSizeMb: 0,
        similarityThreshold: 10,
        filterExistingOutput: true,
        locale: "he",
        isLanguageMenuOpen: false,
        isSummaryExpanded: false,
        progress: { stage: "Cancelling", current: 1, total: 3 },
        summary: {
          totalImages: null,
          uniqueImages: null,
          duplicateImages: null,
          outputDir: "",
        },
        issues: { title: "", hint: "", items: [] },
      },
      elements,
      localization,
    );

    expect(elements.startButton.textContent).toBe("ביטול הרצה");
    expect(elements.progressStage.textContent).toBe("מבטל");
  });

  it("passes through unknown progress stages and renders warning issues", () => {
    const { elements, localization } = createRenderHarness("en");

    renderApp(
      {
        view: "main",
        runStatus: "success_with_warnings",
        minSizeMb: 1,
        similarityThreshold: 10,
        filterExistingOutput: true,
        locale: "en",
        isLanguageMenuOpen: false,
        isSummaryExpanded: false,
        progress: { stage: "custom-stage", current: 3, total: 4 },
        summary: {
          totalImages: 40,
          uniqueImages: 21,
          duplicateImages: 19,
          outputDir: "/output",
        },
        issues: {
          title: "2 warnings",
          hint: "Review skipped files and non-fatal problems.",
          items: ["warning 1", "warning 2"],
        },
      },
      elements,
      localization,
    );

    expect(elements.progressStage.textContent).toBe("custom-stage");
    expect(elements.progressPercent.textContent).toBe("100%");
    expect(elements.progressBar.style.width).toBe("100%");
    expect(elements.progressBar.getAttribute("aria-valuetext")).toBe("custom-stage 100%");
    expect(elements.summaryPanel.hidden).toBe(false);
    expect(elements.summaryBadge.textContent).toBe("Run summary");
    expect(elements.summaryToggleButton.textContent).toBe("Show details");
    expect(elements.summaryToggleButton.getAttribute("aria-expanded")).toBe("false");
    expect(elements.summaryTotalValue.textContent).toBe("40");
    expect(elements.summaryDuplicateValue.textContent).toBe("19");
    expect(elements.summaryStorageSavedValue.textContent).toBe("0 B");
    expect(elements.summaryReportNote.hidden).toBe(true);
    expect(elements.summaryDetails.hidden).toBe(true);
    expect(elements.summaryBreakdown.hidden).toBe(true);
    expect(elements.summaryBreakdownList.children).toHaveLength(0);
    expect(elements.summaryOutput.textContent).toBe("Output folder: /output");
    expect(elements.summaryOutput.getAttribute("dir")).toBe("auto");
    expect(elements.summaryOutput.title).toBe("/output");
    expect(elements.issuesPanel.hidden).toBe(false);
    expect(elements.issuesBadge.textContent).toBe("2 warnings");
    expect(elements.issuesHint.textContent).toBe("Review skipped files and non-fatal problems.");
    expect(elements.issuesList.children).toHaveLength(2);
    expect(elements.issuesList.children[0].getAttribute("dir")).toBe("auto");
    expect(elements.issuesList.children[0].title).toBe("warning 1");
  });

  it("hides the issues panel when there are no issues", () => {
    const { elements, localization } = createRenderHarness("en");

    renderApp(
      {
        view: "main",
        runStatus: "idle",
        minSizeMb: 0,
        similarityThreshold: 10,
        filterExistingOutput: true,
        locale: "en",
        isLanguageMenuOpen: false,
        isSummaryExpanded: false,
        progress: { stage: "Working", current: 0, total: 1 },
        summary: {
          totalImages: null,
          uniqueImages: null,
          duplicateImages: null,
          outputDir: "",
        },
        issues: { title: "", hint: "", items: [] },
      },
      elements,
      localization,
    );

    expect(elements.issuesPanel.hidden).toBe(true);
    expect(elements.issuesList.children).toHaveLength(0);
    expect(elements.summaryPanel.hidden).toBe(true);
  });

  it("renders duplication report metrics collapsed by default", () => {
    const { elements, localization } = createRenderHarness("en");

    renderApp(
      {
        view: "main",
        runStatus: "success",
        minSizeMb: 0,
        filterExistingOutput: true,
        locale: "en",
        isLanguageMenuOpen: false,
        isSummaryExpanded: false,
        progress: { stage: "copy", current: 1, total: 1 },
        summary: {
          totalImages: 86,
          uniqueImages: 48,
          duplicateImages: 38,
          outputDir: "/output/session-alpha",
          report: {
            storageSavedBytes: 1932735283,
            storageSavedHuman: "1.8 GB",
            summary: {
              totalImages: 86,
              uniqueImages: 51,
              duplicateImages: 35,
            },
            folderBreakdown: [
              {
                path: "/input/wedding/raw-dumps/day-01",
                duplicates: 14,
                wastedBytes: 882900992,
              },
              {
                path: "/input/phone-backup/2024/exports",
                duplicates: 11,
                wastedBytes: 540016640,
              },
            ],
          },
        },
        issues: { title: "", hint: "", items: [] },
      },
      elements,
      localization,
    );

    expect(elements.summaryStorageSavedLabel.textContent).toBe("Storage saved");
    expect(elements.summaryStorageSavedValue.textContent).toBe("1.8 GB");
    expect(elements.summaryToggleButton.textContent).toBe("Show details");
    expect(elements.summaryDetails.hidden).toBe(true);
    expect(elements.summaryReportNote.hidden).toBe(true);
    expect(elements.summaryBreakdown.hidden).toBe(true);
  });

  it("renders mismatch note and folder rows when the summary is expanded", () => {
    const { elements, localization } = createRenderHarness("en");

    renderApp(
      {
        view: "main",
        runStatus: "success",
        minSizeMb: 0,
        filterExistingOutput: true,
        locale: "en",
        isLanguageMenuOpen: false,
        isSummaryExpanded: true,
        progress: { stage: "copy", current: 1, total: 1 },
        summary: {
          totalImages: 86,
          uniqueImages: 48,
          duplicateImages: 38,
          outputDir: "/output/session-alpha",
          report: {
            storageSavedBytes: 1932735283,
            storageSavedHuman: "1.8 GB",
            summary: {
              totalImages: 86,
              uniqueImages: 51,
              duplicateImages: 35,
            },
            folderBreakdown: [
              {
                path: "/input/wedding/raw-dumps/day-01",
                duplicates: 14,
                wastedBytes: 882900992,
              },
              {
                path: "/input/phone-backup/2024/exports",
                duplicates: 11,
                wastedBytes: 540016640,
              },
            ],
          },
        },
        issues: { title: "", hint: "", items: [] },
      },
      elements,
      localization,
    );

    expect(elements.summaryToggleButton.textContent).toBe("Hide details");
    expect(elements.summaryToggleButton.getAttribute("aria-expanded")).toBe("true");
    expect(elements.summaryDetails.hidden).toBe(false);
    expect(elements.summaryReportNote.hidden).toBe(false);
    expect(elements.summaryReportNote.textContent).toContain("Grouping found 51 representatives");
    expect(elements.summaryReportNote.textContent).toContain("48 were actually copied");
    expect(elements.summaryBreakdown.hidden).toBe(false);
    expect(elements.summaryBreakdownTitle.textContent).toBe("Top folders by duplicate waste");
    expect(elements.summaryBreakdownHint.textContent).toBe(
      "Highest duplicate counts and wasted storage by source folder.",
    );
    expect(elements.summaryBreakdownEmpty.hidden).toBe(true);
    expect(elements.summaryBreakdownList.children).toHaveLength(2);
    expect(elements.summaryBreakdownList.children[0].textContent).toContain(
      "/input/wedding/raw-dumps/day-01",
    );
    expect(elements.summaryBreakdownList.children[0].textContent).toContain("14 duplicates");
    expect(elements.summaryBreakdownList.children[0].textContent).toContain("842 MB");
    expect(elements.summaryBreakdownList.children[1].textContent).toContain("515 MB");
  });

  it("renders successful runs as fully complete even when the last raw progress payload was partial", () => {
    const { elements, localization } = createRenderHarness("en");

    renderApp(
      {
        view: "main",
        runStatus: "success",
        minSizeMb: 0,
        filterExistingOutput: true,
        locale: "en",
        isLanguageMenuOpen: false,
        isSummaryExpanded: false,
        progress: { stage: "copy", current: 0, total: 1 },
        summary: {
          totalImages: 0,
          uniqueImages: 0,
          duplicateImages: 0,
          outputDir: "/output",
        },
        issues: { title: "", hint: "", items: [] },
      },
      elements,
      localization,
    );

    expect(elements.progressStage.textContent).toBe("Copying");
    expect(elements.progressPercent.textContent).toBe("100%");
    expect(elements.progressBar.style.width).toBe("100%");
  });

  it("renders review labels safely for non-file preview sources", () => {
    const { elements, localization } = createRenderHarness("en");

    renderApp(
      {
        view: "main",
        runStatus: "review",
        minSizeMb: 0,
        similarityThreshold: 10,
        filterExistingOutput: true,
        locale: "en",
        isLanguageMenuOpen: false,
        isSummaryExpanded: false,
        progress: { stage: "review", current: 1, total: 1 },
        review: {
          groups: [
            {
              groupId: "g1",
              images: [
                "data:image/svg+xml;charset=UTF-8,%3Csvg%3E%3C%2Fsvg%3E",
                "/input/review/second.jpg",
              ],
            },
          ],
          selections: {
            g1: "data:image/svg+xml;charset=UTF-8,%3Csvg%3E%3C%2Fsvg%3E",
          },
        },
        issues: { title: "", hint: "", items: [] },
      },
      elements,
      localization,
    );

    expect(elements.reviewPanel.hidden).toBe(false);
    expect(elements.reviewGroups.children).toHaveLength(1);
    expect(elements.reviewGroups.textContent).toContain("Preview image");
    expect(elements.reviewGroups.textContent).toContain("second.jpg");
    expect(elements.reviewGroups.textContent).not.toContain("data:image/svg+xml");
  });
});
