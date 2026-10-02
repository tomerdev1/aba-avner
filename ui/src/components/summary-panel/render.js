export function renderSummaryPanel(summary, isExpanded, elements, localization) {
  if (
    !elements.summaryPanel ||
    !elements.summaryBadge ||
    !elements.summaryHint ||
    !elements.summaryToggleButton ||
    !elements.summaryDetails ||
    !elements.summaryTotalLabel ||
    !elements.summaryTotalValue ||
    !elements.summaryDuplicateLabel ||
    !elements.summaryDuplicateValue ||
    !elements.summaryStorageSavedLabel ||
    !elements.summaryStorageSavedValue ||
    !elements.summaryReportNote ||
    !elements.summaryBreakdown ||
    !elements.summaryBreakdownTitle ||
    !elements.summaryBreakdownHint ||
    !elements.summaryBreakdownEmpty ||
    !elements.summaryBreakdownList ||
    !elements.summaryOutput
  ) {
    return;
  }

  if (!hasSummary(summary)) {
    elements.summaryPanel.hidden = true;
    elements.summaryBadge.textContent = "";
    elements.summaryHint.textContent = "";
    elements.summaryToggleButton.textContent = "";
    elements.summaryToggleButton.setAttribute("aria-expanded", "false");
    elements.summaryDetails.hidden = true;
    elements.summaryTotalLabel.textContent = "";
    elements.summaryTotalValue.textContent = "";
    elements.summaryDuplicateLabel.textContent = "";
    elements.summaryDuplicateValue.textContent = "";
    elements.summaryStorageSavedLabel.textContent = "";
    elements.summaryStorageSavedValue.textContent = "";
    elements.summaryReportNote.textContent = "";
    elements.summaryReportNote.hidden = true;
    elements.summaryBreakdown.hidden = true;
    elements.summaryBreakdownTitle.textContent = "";
    elements.summaryBreakdownHint.textContent = "";
    elements.summaryBreakdownEmpty.textContent = "";
    elements.summaryBreakdownEmpty.hidden = true;
    elements.summaryBreakdownList.replaceChildren();
    elements.summaryOutput.textContent = "";
    return;
  }

  const formatNumber = new Intl.NumberFormat(localization.getLocale());
  const report = normalizeReport(summary.report);
  const hasReportMismatch =
    report.hasGroupedSummary && report.summary.uniqueImages !== summary.uniqueImages;

  elements.summaryBadge.textContent = localization.t("summary.badge");
  elements.summaryHint.textContent = localization.t("summary.hint");
  elements.summaryToggleButton.textContent = localization.t(
    isExpanded ? "summary.hideDetails" : "summary.showDetails",
  );
  elements.summaryToggleButton.setAttribute("aria-expanded", String(isExpanded));
  elements.summaryDetails.hidden = !isExpanded;
  elements.summaryTotalLabel.textContent = localization.t("summary.totalLabel");
  elements.summaryDuplicateLabel.textContent = localization.t("summary.duplicateLabel");
  elements.summaryStorageSavedLabel.textContent = localization.t("summary.storageSavedLabel");
  elements.summaryTotalValue.textContent = formatNumber.format(summary.totalImages);
  elements.summaryDuplicateValue.textContent = formatNumber.format(summary.duplicateImages);
  elements.summaryStorageSavedValue.textContent = getStorageSavedLabel(report);
  elements.summaryReportNote.textContent = hasReportMismatch
    ? localization.t("summary.reportMismatch", {
        groupedUniqueImages: formatNumber.format(report.summary.uniqueImages),
        copiedUniqueImages: formatNumber.format(summary.uniqueImages),
      })
    : "";
  elements.summaryReportNote.hidden = !isExpanded || !hasReportMismatch;
  renderFolderBreakdown(report.folderBreakdown, isExpanded, elements, localization);
  elements.summaryOutput.textContent = localization.t("summary.outputDir", {
    path: summary.outputDir || "",
  });
  elements.summaryOutput.setAttribute("dir", "auto");
  elements.summaryOutput.title = summary.outputDir || "";
  elements.summaryPanel.hidden = false;
}

function hasSummary(summary) {
  return Boolean(
    summary &&
      Number.isFinite(summary.totalImages) &&
      Number.isFinite(summary.uniqueImages) &&
      Number.isFinite(summary.duplicateImages),
  );
}

function normalizeReport(report) {
  const value = report && typeof report === "object" ? report : {};
  const reportSummary = value.summary && typeof value.summary === "object" ? value.summary : {};

  return {
    hasGroupedSummary: Number.isFinite(reportSummary.uniqueImages),
    storageSavedHuman:
      typeof value.storageSavedHuman === "string" && value.storageSavedHuman.trim()
        ? value.storageSavedHuman.trim()
        : "0 B",
    summary: {
      uniqueImages: Number.isFinite(reportSummary.uniqueImages) ? reportSummary.uniqueImages : 0,
    },
    folderBreakdown: Array.isArray(value.folderBreakdown) ? value.folderBreakdown : [],
  };
}

function getStorageSavedLabel(report) {
  return report.storageSavedHuman;
}

function renderFolderBreakdown(folderBreakdown, isExpanded, elements, localization) {
  const rows = Array.isArray(folderBreakdown) ? folderBreakdown : [];

  elements.summaryBreakdown.hidden = !isExpanded;
  elements.summaryBreakdownTitle.textContent = localization.t("summary.folderBreakdownTitle");
  elements.summaryBreakdownHint.textContent = localization.t("summary.folderBreakdownHint");
  elements.summaryBreakdownEmpty.textContent = localization.t("summary.folderBreakdownEmpty");
  elements.summaryBreakdownEmpty.hidden = rows.length > 0;
  elements.summaryBreakdownList.replaceChildren();

  for (const entry of rows) {
    const item = document.createElement("li");
    item.className = "app-summary__breakdown-item";

    const path = document.createElement("span");
    path.className = "app-summary__breakdown-path";
    path.textContent = entry.path || "";
    path.setAttribute("dir", "auto");
    path.title = entry.path || "";

    const duplicates = document.createElement("span");
    duplicates.className = "app-summary__breakdown-duplicates";
    duplicates.textContent = localization.t("summary.folderDuplicates", {
      count: formatInteger(entry.duplicates, localization.getLocale()),
    });

    const wasted = document.createElement("span");
    wasted.className = "app-summary__breakdown-wasted";
    wasted.textContent = formatBytes(entry.wastedBytes, localization.getLocale());

    item.append(path, duplicates, wasted);
    elements.summaryBreakdownList.append(item);
  }
}

function formatInteger(value, locale) {
  return new Intl.NumberFormat(locale).format(Number.isFinite(value) ? value : 0);
}

function formatBytes(bytes, locale) {
  const value = Number.isFinite(bytes) ? bytes : 0;
  const units = ["B", "KB", "MB", "GB", "TB"];

  if (value < 1024) {
    return `${formatInteger(value, locale)} B`;
  }

  let unitIndex = 0;
  let scaled = value;

  while (scaled >= 1024 && unitIndex < units.length - 1) {
    scaled /= 1024;
    unitIndex += 1;
  }

  const digits = scaled >= 10 || unitIndex === 0 ? 0 : 1;
  return `${new Intl.NumberFormat(locale, {
    minimumFractionDigits: 0,
    maximumFractionDigits: digits,
  }).format(scaled)} ${units[unitIndex]}`;
}
