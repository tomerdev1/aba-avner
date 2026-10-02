import { renderHeader } from "../components/header/render.js";
import { renderIssues } from "../components/issues-panel/render.js";
import { renderProgressSection } from "../components/progress-section/render.js";
import { renderReviewPanel } from "../components/review-panel/render.js";
import { renderSummaryPanel } from "../components/summary-panel/render.js";
import { renderLanguageMenu } from "../features/language-menu/render.js";
import { renderSettingsCard } from "../features/settings/render.js";
import { isBusyStatus } from "./state.js";
import { setView, syncStageHeight } from "./view.js";

export function renderApp(state, elements, localization) {
  renderAppShell(state, elements, { isBusy: isBusyStatus(state.runStatus) });
  renderHeader(state, elements, localization.t);
  renderLanguageMenu(state, elements, localization.t);
  renderProgressSection(state.progress, elements, localization.t, state.runStatus);
  renderSettingsCard(state, elements, localization.t);
  renderReviewPanel(state.review, state.runStatus, elements, localization);
  renderSummaryPanel(state.summary, state.isSummaryExpanded, elements, localization);
  renderIssues(state.issues, elements);
  syncStageHeight(elements);
}

function renderAppShell(state, elements, { isBusy }) {
  const isReviewing = Array.isArray(state.review?.groups) && state.review.groups.length > 0;

  setView(state.view, elements);

  if (elements.startButton) {
    elements.startButton.disabled = false;
    elements.startButton.hidden = isReviewing;
  }
  if (elements.mainCard) {
    elements.mainCard.setAttribute("aria-busy", String(isBusy));
  }

  for (const control of [
    elements.settingsButton,
    elements.languageButton,
    elements.localeEnButton,
    elements.localeHeButton,
    elements.backButton,
    elements.resetSettingsButton,
    elements.minSizeSlider,
    elements.similarityThresholdInput,
    elements.existingOutputToggle,
    elements.visualReviewToggle,
    elements.exportGroupsToggle,
    elements.reviewApplyButton,
    elements.reviewCancelButton,
  ]) {
    if (control) {
      control.disabled = isBusy || (isReviewing && control === elements.settingsButton);
    }
  }

  if (elements.settingsButton) {
    elements.settingsButton.disabled = isBusy || isReviewing;
  }
}
