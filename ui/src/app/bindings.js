import { bindLanguageMenuInteractions } from "../features/language-menu/bindings.js";
import { bindSettingsInteractions } from "../features/settings/bindings.js";
import { syncStageHeight as defaultSyncStageHeight } from "./view.js";

export function bindAppInteractions(elements, app, options = {}) {
  const documentRef = options.documentRef ?? globalThis.document;
  const ResizeObserverRef = options.ResizeObserverRef ?? globalThis.ResizeObserver;
  const syncStageHeight = options.syncStageHeight ?? defaultSyncStageHeight;

  bindAppShellInteractions(elements, app, {
    ResizeObserverRef,
    syncStageHeight,
  });
  bindLanguageMenuInteractions(elements, app, {
    documentRef,
  });
  bindSettingsInteractions(elements, app);
}

function bindAppShellInteractions(elements, app, options = {}) {
  const ResizeObserverRef = options.ResizeObserverRef ?? globalThis.ResizeObserver;
  const syncStageHeight = options.syncStageHeight ?? defaultSyncStageHeight;

  syncStageHeight(elements);

  if (typeof ResizeObserverRef !== "undefined") {
    const resizeObserver = new ResizeObserverRef(() => syncStageHeight(elements));
    if (elements.mainCard) {
      resizeObserver.observe(elements.mainCard);
    }
    if (elements.settingsCard) {
      resizeObserver.observe(elements.settingsCard);
    }
  }

  elements.startButton.addEventListener("click", app.toggleRun);
  elements.summaryToggleButton.addEventListener("click", app.toggleSummaryExpanded);
  elements.reviewApplyButton.addEventListener("click", app.applyReview);
  elements.reviewCancelButton.addEventListener("click", app.cancelReview);
  elements.reviewGroups.addEventListener("click", (event) => {
    const button = event.target.closest("[data-group-id][data-image-path]");
    if (!button) {
      return;
    }

    app.selectReviewImage(button.dataset.groupId, button.dataset.imagePath);
  });
}
