export function bindSettingsInteractions(elements, app) {
  function focusSettingsPanel() {
    const focusTarget = elements.minSizeSlider ?? elements.backButton ?? elements.settingsCard;
    focusTarget?.focus();
  }

  if (elements.minSizeSlider) {
    elements.minSizeSlider.addEventListener("input", (event) => {
      const value = parseFloat(event.target.value || "0");
      app.setMinSize(value);
    });
  }

  if (elements.similarityThresholdInput) {
    elements.similarityThresholdInput.addEventListener("input", (event) => {
      const value = Number.parseInt(event.target.value || "10", 10);
      app.setSimilarityThreshold(value);
    });
  }

  if (elements.existingOutputToggle) {
    elements.existingOutputToggle.addEventListener("change", (event) => {
      app.setFilterExistingOutput(event.target.checked);
    });
  }

  if (elements.visualReviewToggle) {
    elements.visualReviewToggle.addEventListener("change", (event) => {
      app.setVisualReviewEnabled(event.target.checked);
    });
  }

  if (elements.exportGroupsToggle) {
    elements.exportGroupsToggle.addEventListener("change", (event) => {
      app.setExportSimilarImageGroups(event.target.checked);
    });
  }

  elements.settingsButton?.addEventListener("click", () => {
    app.openSettings();
    focusSettingsPanel();
  });
  elements.backButton?.addEventListener("click", () => {
    app.closeSettings();
    elements.settingsButton?.focus();
  });

  elements.resetSettingsButton?.addEventListener("click", () => {
    app.resetSettings();
  });
}
