export function getTauriApi() {
  return window.__TAURI__ || {};
}

export function getElements() {
  return {
    ...getStageElements(),
    ...getHeaderElements(),
    ...getLanguageMenuElements(),
    ...getProgressElements(),
    ...getReviewElements(),
    ...getSummaryElements(),
    ...getIssuesElements(),
    ...getSettingsElements(),
  };
}

function getStageElements() {
  return {
    cardStage: document.getElementById("cardStage"),
    mainCard: document.getElementById("mainCard"),
    settingsCard: document.getElementById("settingsCard"),
    startButton: document.getElementById("startButton"),
  };
}

function getHeaderElements() {
  return {
    appEyebrow: document.getElementById("appEyebrow"),
    appTitle: document.querySelector(".app-title"),
    appSubtitle: document.getElementById("appSubtitle"),
    headerControls: document.getElementById("headerControls"),
    settingsButton: document.getElementById("settingsButton"),
  };
}

function getLanguageMenuElements() {
  return {
    languageButton: document.getElementById("languageButton"),
    languageMenu: document.getElementById("languageMenu"),
    localeEnButton: document.getElementById("localeEnButton"),
    localeHeButton: document.getElementById("localeHeButton"),
  };
}

function getProgressElements() {
  return {
    progressBar: document.getElementById("progressBar"),
    progressStage: document.getElementById("progressStage"),
    progressPercent: document.getElementById("progressPercent"),
  };
}

function getSummaryElements() {
  return {
    summaryPanel: document.getElementById("summaryPanel"),
    summaryBadge: document.getElementById("summaryBadge"),
    summaryHint: document.getElementById("summaryHint"),
    summaryToggleButton: document.getElementById("summaryToggleButton"),
    summaryDetails: document.getElementById("summaryDetails"),
    summaryTotalLabel: document.getElementById("summaryTotalLabel"),
    summaryTotalValue: document.getElementById("summaryTotalValue"),
    summaryDuplicateLabel: document.getElementById("summaryDuplicateLabel"),
    summaryDuplicateValue: document.getElementById("summaryDuplicateValue"),
    summaryStorageSavedLabel: document.getElementById("summaryStorageSavedLabel"),
    summaryStorageSavedValue: document.getElementById("summaryStorageSavedValue"),
    summaryReportNote: document.getElementById("summaryReportNote"),
    summaryBreakdown: document.getElementById("summaryBreakdown"),
    summaryBreakdownTitle: document.getElementById("summaryBreakdownTitle"),
    summaryBreakdownHint: document.getElementById("summaryBreakdownHint"),
    summaryBreakdownEmpty: document.getElementById("summaryBreakdownEmpty"),
    summaryBreakdownList: document.getElementById("summaryBreakdownList"),
    summaryOutput: document.getElementById("summaryOutput"),
  };
}

function getReviewElements() {
  return {
    reviewPanel: document.getElementById("reviewPanel"),
    reviewBadge: document.getElementById("reviewBadge"),
    reviewTitle: document.getElementById("reviewTitle"),
    reviewHint: document.getElementById("reviewHint"),
    reviewEmpty: document.getElementById("reviewEmpty"),
    reviewGroups: document.getElementById("reviewGroups"),
    reviewApplyButton: document.getElementById("reviewApplyButton"),
    reviewCancelButton: document.getElementById("reviewCancelButton"),
  };
}

function getIssuesElements() {
  return {
    issuesPanel: document.getElementById("issuesPanel"),
    issuesBadge: document.getElementById("issuesBadge"),
    issuesHint: document.getElementById("issuesHint"),
    issuesList: document.getElementById("issuesList"),
  };
}

function getSettingsElements() {
  return {
    backButton: document.getElementById("backButton"),
    resetSettingsButton: document.getElementById("resetSettingsButton"),
    settingsEyebrow: document.getElementById("settingsEyebrow"),
    settingsTitle: document.getElementById("settingsTitle"),
    minSizeLabel: document.getElementById("minSizeLabel"),
    minSizeDescription: document.getElementById("minSizeDescription"),
    minSizePrompt: document.getElementById("minSizePrompt"),
    minSizeSlider: document.getElementById("minSizeSlider"),
    minSizeValue: document.getElementById("minSizeValue"),
    similarityThresholdLabel: document.getElementById("similarityThresholdLabel"),
    similarityThresholdDescription: document.getElementById("similarityThresholdDescription"),
    similarityThresholdPrompt: document.getElementById("similarityThresholdPrompt"),
    similarityThresholdInput: document.getElementById("similarityThresholdInput"),
    similarityThresholdValue: document.getElementById("similarityThresholdValue"),
    similarityThresholdMinLabel: document.getElementById("similarityThresholdMinLabel"),
    similarityThresholdMaxLabel: document.getElementById("similarityThresholdMaxLabel"),
    existingOutputToggle: document.getElementById("existingOutputToggle"),
    existingOutputLabel: document.getElementById("existingOutputLabel"),
    existingOutputDescription: document.getElementById("existingOutputDescription"),
    visualReviewToggle: document.getElementById("visualReviewToggle"),
    visualReviewLabel: document.getElementById("visualReviewLabel"),
    visualReviewDescription: document.getElementById("visualReviewDescription"),
    exportGroupsToggle: document.getElementById("exportGroupsToggle"),
    exportGroupsLabel: document.getElementById("exportGroupsLabel"),
    exportGroupsDescription: document.getElementById("exportGroupsDescription"),
    licenseNotice: document.getElementById("licenseNotice"),
  };
}
