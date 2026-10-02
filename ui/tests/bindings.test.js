import { describe, expect, it, vi } from "vitest";

import { bindAppInteractions } from "../src/app/bindings.js";
import { getElements } from "../src/app/dom.js";
import { loadHtml } from "./helpers/ui-test-helpers.js";

function createApp() {
  return {
    cancelDedupe: vi.fn(),
    closeLanguageMenu: vi.fn(),
    closeSettings: vi.fn(),
    openSettings: vi.fn(),
    resetSettings: vi.fn(),
    runDedupe: vi.fn(),
    setLocale: vi.fn(),
    setFilterExistingOutput: vi.fn(),
    setVisualReviewEnabled: vi.fn(),
    setExportSimilarImageGroups: vi.fn(),
    setMinSize: vi.fn(),
    toggleSummaryExpanded: vi.fn(),
    setSimilarityThreshold: vi.fn(),
    toggleRun: vi.fn(),
    toggleLanguageMenu: vi.fn(),
  };
}

describe("app bindings", () => {
  it("wires button and slider interactions to controller methods", () => {
    loadHtml();
    const elements = getElements();
    const app = createApp();

    bindAppInteractions(elements, app, {
      documentRef: document,
      ResizeObserverRef: undefined,
      syncStageHeight: vi.fn(),
    });

    elements.startButton.click();
    elements.settingsButton.click();
    elements.resetSettingsButton.click();
    elements.backButton.click();
    elements.languageButton.click();
    elements.summaryToggleButton.click();
    elements.localeEnButton.click();
    elements.localeHeButton.click();
    elements.minSizeSlider.value = "2.5";
    elements.minSizeSlider.dispatchEvent(new Event("input"));
    elements.similarityThresholdInput.value = "6";
    elements.similarityThresholdInput.dispatchEvent(new Event("input"));
    elements.existingOutputToggle.checked = false;
    elements.existingOutputToggle.dispatchEvent(new Event("change"));
    elements.visualReviewToggle.checked = false;
    elements.visualReviewToggle.dispatchEvent(new Event("change"));
    elements.exportGroupsToggle.checked = false;
    elements.exportGroupsToggle.dispatchEvent(new Event("change"));

    expect(app.toggleRun).toHaveBeenCalledTimes(1);
    expect(app.openSettings).toHaveBeenCalledTimes(1);
    expect(app.resetSettings).toHaveBeenCalledTimes(1);
    expect(app.closeSettings).toHaveBeenCalledTimes(1);
    expect(app.toggleLanguageMenu).toHaveBeenCalledTimes(1);
    expect(app.toggleSummaryExpanded).toHaveBeenCalledTimes(1);
    expect(app.setLocale).toHaveBeenNthCalledWith(1, "en");
    expect(app.setLocale).toHaveBeenNthCalledWith(2, "he");
    expect(app.setMinSize).toHaveBeenCalledWith(2.5);
    expect(app.setSimilarityThreshold).toHaveBeenCalledWith(6);
    expect(app.setFilterExistingOutput).toHaveBeenCalledWith(false);
    expect(app.setVisualReviewEnabled).toHaveBeenCalledWith(false);
    expect(app.setExportSimilarImageGroups).toHaveBeenCalledWith(false);
  });

  it("moves focus into settings and restores it to the trigger on close", () => {
    loadHtml();
    const elements = getElements();
    const app = createApp();

    bindAppInteractions(elements, app, {
      documentRef: document,
      ResizeObserverRef: undefined,
      syncStageHeight: vi.fn(),
    });

    elements.settingsButton.focus();
    elements.settingsButton.click();

    expect(app.openSettings).toHaveBeenCalledTimes(1);
    expect(document.activeElement).toBe(elements.minSizeSlider);

    elements.backButton.click();

    expect(app.closeSettings).toHaveBeenCalledTimes(1);
    expect(document.activeElement).toBe(elements.settingsButton);
  });

  it("toggles the existing-output checkbox when its label copy is clicked", async () => {
    loadHtml();
    const elements = getElements();
    const app = createApp();

    bindAppInteractions(elements, app, {
      documentRef: document,
      ResizeObserverRef: undefined,
      syncStageHeight: vi.fn(),
    });

    elements.existingOutputToggle.checked = true;
    elements.existingOutputLabel.click();
    await Promise.resolve();

    expect(elements.existingOutputToggle.checked).toBe(false);
    expect(app.setFilterExistingOutput).toHaveBeenCalledTimes(1);
    expect(app.setFilterExistingOutput).toHaveBeenCalledWith(false);
  });

  it("closes the language menu on escape and outside clicks only when open", () => {
    loadHtml();
    const elements = getElements();
    const app = createApp();
    app.toggleLanguageMenu.mockImplementation(() => {
      elements.languageMenu.hidden = false;
      elements.languageButton.setAttribute("aria-expanded", "true");
    });
    app.closeLanguageMenu.mockImplementation(() => {
      elements.languageMenu.hidden = true;
      elements.languageButton.setAttribute("aria-expanded", "false");
    });

    bindAppInteractions(elements, app, {
      documentRef: document,
      ResizeObserverRef: undefined,
      syncStageHeight: vi.fn(),
    });

    elements.headerControls.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    expect(app.closeLanguageMenu).not.toHaveBeenCalled();

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter" }));
    expect(app.closeLanguageMenu).not.toHaveBeenCalled();

    document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
    expect(app.closeLanguageMenu).toHaveBeenCalledTimes(0);

    document.body.click();
    expect(app.closeLanguageMenu).toHaveBeenCalledTimes(0);

    elements.languageButton.click();
    document.body.click();
    expect(app.closeLanguageMenu).toHaveBeenCalledTimes(1);
  });

  it("supports keyboard navigation for the language menu and restores focus on dismiss", () => {
    loadHtml();
    const elements = getElements();
    const app = createApp();
    app.toggleLanguageMenu.mockImplementation(() => {
      elements.languageMenu.hidden = false;
      elements.languageButton.setAttribute("aria-expanded", "true");
    });
    app.closeLanguageMenu.mockImplementation(() => {
      elements.languageMenu.hidden = true;
      elements.languageButton.setAttribute("aria-expanded", "false");
    });

    bindAppInteractions(elements, app, {
      documentRef: document,
      ResizeObserverRef: undefined,
      syncStageHeight: vi.fn(),
    });

    elements.languageButton.focus();
    elements.languageButton.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));

    expect(app.toggleLanguageMenu).toHaveBeenCalledTimes(1);
    expect(document.activeElement).toBe(elements.localeEnButton);

    elements.languageMenu.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
    expect(document.activeElement).toBe(elements.localeHeButton);

    elements.languageMenu.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowUp", bubbles: true }));
    expect(document.activeElement).toBe(elements.localeEnButton);

    elements.languageMenu.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(app.closeLanguageMenu).toHaveBeenCalledTimes(1);
    expect(document.activeElement).toBe(elements.languageButton);
  });

  it("syncs stage height immediately and through resize observer updates", () => {
    loadHtml();
    const elements = getElements();
    const app = createApp();
    const syncStageHeight = vi.fn();
    let resizeCallback;
    const observe = vi.fn();

    class ResizeObserverMock {
      constructor(callback) {
        resizeCallback = callback;
      }

      observe(target) {
        observe(target);
      }
    }

    bindAppInteractions(elements, app, {
      documentRef: document,
      ResizeObserverRef: ResizeObserverMock,
      syncStageHeight,
    });

    expect(syncStageHeight).toHaveBeenCalledTimes(1);
    expect(observe).toHaveBeenCalledWith(elements.mainCard);
    expect(observe).toHaveBeenCalledWith(elements.settingsCard);

    resizeCallback();
    expect(syncStageHeight).toHaveBeenCalledTimes(2);
  });
});
