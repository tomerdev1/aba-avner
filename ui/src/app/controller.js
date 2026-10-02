import {
  buildFailureIssues,
  buildTauriUnavailableIssues,
  buildWarningsIssues,
} from "./messages.js";
import { createInitialState, isBusyStatus, reduceAppState } from "./state.js";
import {
  DEFAULT_EXPORT_SIMILAR_IMAGE_GROUPS,
  DEFAULT_FILTER_EXISTING_OUTPUT,
  DEFAULT_MIN_SIZE_MB,
  DEFAULT_SIMILARITY_THRESHOLD,
  DEFAULT_VISUAL_REVIEW_ENABLED,
} from "../services/settings-store.js";

export function createAppController(elements, tauriClient, localization, settingsStore, render) {
  let progressListener = null;
  let activeRunId = null;
  let runSequence = 0;
  let reviewContext = null;
  let state = createInitialState(
    settingsStore.getMinSizeMb(),
    settingsStore.getSimilarityThreshold(),
    localization.getLocale(),
    settingsStore.getFilterExistingOutput(),
    settingsStore.getVisualReviewEnabled(),
    settingsStore.getExportSimilarImageGroups(),
  );

  function update(action) {
    const nextState = reduceAppState(state, action);
    if (nextState === state) {
      return;
    }

    state = nextState;
    render(state);
  }

  async function ensureProgressListener() {
    if (progressListener) {
      return;
    }

    progressListener = await tauriClient.listenToProgress((payload) => {
      if (!isBusyStatus(state.runStatus)) {
        return;
      }

      if (!payload?.runId || payload.runId !== activeRunId) {
        return;
      }

      update({
        type: "progress/set",
        payload,
      });
    });
  }

  async function chooseDirectory(title) {
    return tauriClient.chooseDirectory(title);
  }

  function openSettings() {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    update({ type: "view/set", view: "settings" });
  }

  function closeSettings() {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    update({ type: "view/set", view: "main" });
  }

  function setProgress(stage, current, total) {
    update({
      type: "progress/set",
      payload: { stage, current, total },
    });
  }

  function setView(view) {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    update({ type: "view/set", view });
  }

  function setMinSize(value) {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    settingsStore.setMinSizeMb(value);
    update({ type: "minSize/set", value });
  }

  function setSimilarityThreshold(value) {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    settingsStore.setSimilarityThreshold(value);
    update({ type: "similarityThreshold/set", value });
  }

  function setFilterExistingOutput(value) {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    settingsStore.setFilterExistingOutput(value);
    update({ type: "filterExistingOutput/set", value });
  }

  function setVisualReviewEnabled(value) {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    settingsStore.setVisualReviewEnabled(value);
    update({ type: "visualReviewEnabled/set", value });
  }

  function setExportSimilarImageGroups(value) {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    settingsStore.setExportSimilarImageGroups(value);
    update({ type: "exportSimilarImageGroups/set", value });
  }

  function resetSettings() {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    settingsStore.setMinSizeMb(DEFAULT_MIN_SIZE_MB);
    settingsStore.setSimilarityThreshold(DEFAULT_SIMILARITY_THRESHOLD);
    settingsStore.setFilterExistingOutput(DEFAULT_FILTER_EXISTING_OUTPUT);
    settingsStore.setVisualReviewEnabled(DEFAULT_VISUAL_REVIEW_ENABLED);
    settingsStore.setExportSimilarImageGroups(DEFAULT_EXPORT_SIMILAR_IMAGE_GROUPS);

    update({ type: "minSize/set", value: DEFAULT_MIN_SIZE_MB });
    update({
      type: "similarityThreshold/set",
      value: DEFAULT_SIMILARITY_THRESHOLD,
    });
    update({
      type: "filterExistingOutput/set",
      value: DEFAULT_FILTER_EXISTING_OUTPUT,
    });
    update({
      type: "visualReviewEnabled/set",
      value: DEFAULT_VISUAL_REVIEW_ENABLED,
    });
    update({
      type: "exportSimilarImageGroups/set",
      value: DEFAULT_EXPORT_SIMILAR_IMAGE_GROUPS,
    });
  }

  function toggleLanguageMenu() {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    update({ type: "languageMenu/toggle" });
  }

  function closeLanguageMenu() {
    update({ type: "languageMenu/close" });
  }

  function toggleSummaryExpanded() {
    if (isBusyStatus(state.runStatus) || !hasVisibleSummary(state.summary)) {
      return;
    }

    update({ type: "summary/toggle" });
  }

  function setLocale(locale) {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    localization.setLocale(locale);
    update({ type: "locale/set", locale: localization.getLocale() });
  }

  async function runDedupe() {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    if (!tauriClient.isAvailable()) {
      update({
        type: "issues/set",
        issues: buildTauriUnavailableIssues(localization.t),
      });
      update({ type: "run/complete", status: "error" });
      update({ type: "view/set", view: "main" });
      return;
    }

    update({ type: "run/start" });
    activeRunId = createRunId();

    try {
      const inputDir = await chooseDirectory(localization.t("dialogs.selectInput"));
      if (!inputDir) {
        activeRunId = null;
        update({ type: "run/complete", status: "idle" });
        return;
      }

      const outputDir = await chooseDirectory(localization.t("dialogs.selectOutput"));
      if (!outputDir) {
        activeRunId = null;
        update({ type: "run/complete", status: "idle" });
        return;
      }

      await ensureProgressListener();
      setProgress(localization.t("app.starting"), 0, 1);

      const response = await (tauriClient.prepareDedupeReview
        ? tauriClient.prepareDedupeReview({
            inputDir,
            outputDir,
            minSizeMb: state.minSizeMb,
            similarityThreshold: state.similarityThreshold,
            filterExistingOutput: state.filterExistingOutput,
            runId: activeRunId,
            locale: state.locale,
          })
        : tauriClient.runDedupe({
            inputDir,
            outputDir,
            minSizeMb: state.minSizeMb,
            similarityThreshold: state.similarityThreshold,
            filterExistingOutput: state.filterExistingOutput,
            runId: activeRunId,
            locale: state.locale,
          }));
      if (Array.isArray(response?.groups)) {
        reviewContext = {
          outputDir,
          similarityThreshold: state.similarityThreshold,
          filterExistingOutput: state.filterExistingOutput,
          exportSimilarImageGroups: state.exportSimilarImageGroups,
          locale: state.locale,
        };
        if (state.visualReviewEnabled) {
          update({
            type: "review/set",
            review: response,
          });
          update({
            type: "issues/set",
            issues: buildWarningsIssues(localization.t, response),
          });
          update({
            type: "run/complete",
            status: "review",
          });
          update({
            type: "progress/set",
            payload: { stage: "review", current: 0, total: 1 },
          });
        } else {
          const exportResponse = await exportPreparedReview(response);
          const combinedResponse = mergeRunWarnings(response, exportResponse);

          reviewContext = null;
          update({
            type: "summary/set",
            summary: combinedResponse,
          });
          update({
            type: "issues/set",
            issues: buildWarningsIssues(localization.t, combinedResponse),
          });
          update({
            type: "run/complete",
            status: hasWarnings(combinedResponse) ? "success_with_warnings" : "success",
          });
        }
      } else {
        update({
          type: "summary/set",
          summary: response,
        });
        update({
          type: "issues/set",
          issues: buildWarningsIssues(localization.t, response),
        });
        update({
          type: "run/complete",
          status: hasWarnings(response) ? "success_with_warnings" : "success",
        });
      }
    } catch (error) {
      const failureIssues = buildFailureIssues(localization.t, error);
      reviewContext = null;

      update({
        type: "issues/set",
        issues: failureIssues,
      });
      update({
        type: "run/complete",
        status: isCancelledError(error) ? "idle" : "error",
      });
    } finally {
      activeRunId = null;
    }
  }

  async function applyReview() {
    if (isBusyStatus(state.runStatus) || !reviewContext) {
      return false;
    }

    const reviewPayload = {
      outputDir: reviewContext.outputDir,
      similarityThreshold: reviewContext.similarityThreshold,
      filterExistingOutput: reviewContext.filterExistingOutput,
      exportSimilarImageGroups: reviewContext.exportSimilarImageGroups,
      locale: reviewContext.locale,
      groups: state.review.groups,
      selections: state.review.selections,
    };

    update({ type: "run/review-export-start" });
    activeRunId = createRunId();

    try {
      await ensureProgressListener();
      setProgress("review", 1, 1);

      const response = await tauriClient.exportDedupeReview({
        ...reviewPayload,
        runId: activeRunId,
      });

      reviewContext = null;
      update({
        type: "summary/set",
        summary: response,
      });
      update({
        type: "issues/set",
        issues: buildWarningsIssues(localization.t, response),
      });
      update({
        type: "run/complete",
        status: hasWarnings(response) ? "success_with_warnings" : "success",
      });
      return true;
    } catch (error) {
      update({
        type: "issues/set",
        issues: buildFailureIssues(localization.t, error),
      });
      update({
        type: "run/complete",
        status: isCancelledError(error) ? "review" : "error",
      });
      return false;
    } finally {
      activeRunId = null;
    }
  }

  async function exportPreparedReview(reviewResponse) {
    if (!reviewContext) {
      throw new Error("Missing review context");
    }

    activeRunId = createRunId();
    setProgress("copy", 0, 1);

    return tauriClient.exportDedupeReview({
      outputDir: reviewContext.outputDir,
      similarityThreshold: reviewContext.similarityThreshold,
      filterExistingOutput: reviewContext.filterExistingOutput,
      exportSimilarImageGroups: reviewContext.exportSimilarImageGroups,
      locale: reviewContext.locale,
      groups: Array.isArray(reviewResponse?.groups) ? reviewResponse.groups : [],
      selections: {},
      runId: activeRunId,
    });
  }

  function selectReviewImage(groupId, path) {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    update({ type: "review/select", groupId, path });
  }

  function cancelReview() {
    if (isBusyStatus(state.runStatus)) {
      return;
    }

    reviewContext = null;
    update({ type: "review/clear" });
    update({ type: "issues/set", issues: { title: "", hint: "", items: [] } });
    update({ type: "run/complete", status: "idle" });
    update({ type: "progress/set", payload: { stage: "Idle", current: 0, total: 1 } });
  }

  async function cancelDedupe() {
    if (state.runStatus !== "running") {
      return false;
    }

    update({ type: "run/cancelling" });
    setProgress("Cancelling", state.progress.current, state.progress.total);
    try {
      return await tauriClient.cancelDedupe();
    } catch (error) {
      update({
        type: "issues/set",
        issues: buildFailureIssues(localization.t, error),
      });
      update({ type: "run/complete", status: "error" });
      return false;
    }
  }

  async function toggleRun() {
    if (state.runStatus === "running") {
      return cancelDedupe();
    }

    if (state.runStatus === "cancelling") {
      return false;
    }

    if (state.runStatus === "review") {
      return applyReview();
    }

    return runDedupe();
  }

  render(state);

  return {
    applyReview,
    cancelDedupe,
    cancelReview,
    closeSettings,
    closeLanguageMenu,
    openSettings,
    resetSettings,
    runDedupe,
    setLocale,
    setFilterExistingOutput,
    setVisualReviewEnabled,
    setExportSimilarImageGroups,
    setMinSize,
    setSimilarityThreshold,
    selectReviewImage,
    setProgress,
    setView,
    toggleSummaryExpanded,
    toggleRun,
    toggleLanguageMenu,
  };

  function createRunId() {
    runSequence += 1;

    if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
      return crypto.randomUUID();
    }

    return `run-${Date.now()}-${runSequence}`;
  }
}

function hasWarnings(response) {
  return Array.isArray(response?.warningDetails)
    ? response.warningDetails.length > 0
    : Array.isArray(response?.warnings) && response.warnings.length > 0;
}

function mergeRunWarnings(prepareResponse, exportResponse) {
  const warnings = [
    ...(Array.isArray(prepareResponse?.warnings) ? prepareResponse.warnings : []),
    ...(Array.isArray(exportResponse?.warnings) ? exportResponse.warnings : []),
  ];
  const warningDetails = [
    ...(Array.isArray(prepareResponse?.warningDetails) ? prepareResponse.warningDetails : []),
    ...(Array.isArray(exportResponse?.warningDetails) ? exportResponse.warningDetails : []),
  ];

  return {
    ...exportResponse,
    warnings,
    warningDetails,
  };
}

function isCancelledError(error) {
  return Boolean(error && typeof error === "object" && error.code === "cancelled");
}

function hasVisibleSummary(summary) {
  return Boolean(
    summary &&
      Number.isFinite(summary.totalImages) &&
      Number.isFinite(summary.duplicateImages),
  );
}
