import { formatMegabytes, updateMinSizeLabel } from "../../app/progress.js";

export function renderSettingsCard(state, elements, t) {
  if (elements.settingsEyebrow) {
    elements.settingsEyebrow.textContent = t("settings.eyebrow");
  }
  if (elements.settingsTitle) {
    elements.settingsTitle.textContent = t("settings.title");
  }
  if (elements.backButton) {
    elements.backButton.textContent = t("settings.back");
  }
  if (elements.resetSettingsButton) {
    elements.resetSettingsButton.textContent = t("settings.reset");
  }
  if (elements.minSizeLabel) {
    elements.minSizeLabel.textContent = t("settings.minSizeLabel");
  }
  if (elements.minSizeDescription) {
    elements.minSizeDescription.textContent = t("settings.minSizeDescription");
  }
  if (elements.minSizePrompt) {
    elements.minSizePrompt.textContent = t("settings.ignoreSmallerThan");
  }
  if (elements.minSizeSlider) {
    elements.minSizeSlider.value = String(state.minSizeMb);
    elements.minSizeSlider.setAttribute("aria-valuenow", String(state.minSizeMb));
    elements.minSizeSlider.setAttribute("aria-valuetext", formatMegabytes(state.minSizeMb, t));
  }

  updateMinSizeLabel(state.minSizeMb, elements, t);

  if (elements.similarityThresholdLabel) {
    elements.similarityThresholdLabel.textContent = t("settings.similarityThresholdLabel");
  }
  if (elements.similarityThresholdDescription) {
    elements.similarityThresholdDescription.textContent = t(
      "settings.similarityThresholdDescription",
    );
  }
  if (elements.similarityThresholdPrompt) {
    elements.similarityThresholdPrompt.textContent = t("settings.matchImagesWithin");
  }
  if (elements.similarityThresholdMinLabel) {
    elements.similarityThresholdMinLabel.textContent = t("settings.similarityThresholdMin");
  }
  if (elements.similarityThresholdMaxLabel) {
    elements.similarityThresholdMaxLabel.textContent = t("settings.similarityThresholdMax");
  }
  if (elements.similarityThresholdInput) {
    elements.similarityThresholdInput.value = String(state.similarityThreshold);
    elements.similarityThresholdInput.setAttribute(
      "aria-valuenow",
      String(state.similarityThreshold),
    );
    elements.similarityThresholdInput.setAttribute(
      "aria-valuetext",
      String(state.similarityThreshold),
    );
  }
  if (elements.similarityThresholdValue) {
    elements.similarityThresholdValue.textContent = String(state.similarityThreshold);
  }

  if (elements.existingOutputLabel) {
    elements.existingOutputLabel.textContent = t("settings.filterExistingOutputLabel");
  }
  if (elements.existingOutputDescription) {
    elements.existingOutputDescription.textContent = t("settings.filterExistingOutputDescription");
  }
  if (elements.existingOutputToggle) {
    elements.existingOutputToggle.checked = Boolean(state.filterExistingOutput);
    elements.existingOutputToggle.setAttribute(
      "aria-label",
      t("settings.filterExistingOutputLabel"),
    );
  }

  if (elements.visualReviewLabel) {
    elements.visualReviewLabel.textContent = t("settings.visualReviewLabel");
  }
  if (elements.visualReviewDescription) {
    elements.visualReviewDescription.textContent = t("settings.visualReviewDescription");
  }
  if (elements.visualReviewToggle) {
    elements.visualReviewToggle.checked = Boolean(state.visualReviewEnabled);
    elements.visualReviewToggle.setAttribute("aria-label", t("settings.visualReviewLabel"));
  }

  if (elements.exportGroupsLabel) {
    elements.exportGroupsLabel.textContent = t("settings.exportGroupsLabel");
  }
  if (elements.exportGroupsDescription) {
    elements.exportGroupsDescription.textContent = t("settings.exportGroupsDescription");
  }
  if (elements.exportGroupsToggle) {
    elements.exportGroupsToggle.checked = Boolean(state.exportSimilarImageGroups);
    elements.exportGroupsToggle.setAttribute("aria-label", t("settings.exportGroupsLabel"));
  }
}
