const DEDUPE_PROGRESS_EVENT = "dedupe_progress";
const RUN_DEDUPE_COMMAND = "run_dedupe";
const PREPARE_DEDUPE_REVIEW_COMMAND = "prepare_dedupe_review";
const EXPORT_DEDUPE_REVIEW_COMMAND = "export_dedupe_review";
const CANCEL_DEDUPE_COMMAND = "cancel_dedupe";
const IS_MSIX_INSTALL_COMMAND = "is_msix_install";
const UNEXPECTED_TAURI_ERROR_CODE = "unexpected_tauri_error";

export function isStructuredTauriError(error) {
  return Boolean(
    error &&
      typeof error === "object" &&
      typeof error.code === "string" &&
      typeof error.message === "string",
  );
}

function isStructuredWarningDetail(warning) {
  return Boolean(
    warning &&
      typeof warning === "object" &&
      typeof warning.code === "string" &&
      typeof warning.path === "string" &&
      (warning.detail === undefined || typeof warning.detail === "string"),
  );
}

function isStructuredProgressPayload(payload) {
  return Boolean(
    payload &&
      typeof payload === "object" &&
      typeof payload.runId === "string" &&
      typeof payload.stage === "string" &&
      Number.isFinite(payload.current) &&
      Number.isFinite(payload.total),
  );
}

function normalizeRunDedupeError(error) {
  if (isStructuredTauriError(error)) {
    return {
      code: error.code,
      message: error.message,
    };
  }

  console.error("Unexpected Tauri error payload", error);

  return {
    code: UNEXPECTED_TAURI_ERROR_CODE,
    message: buildDeveloperTraceableMessage(error),
  };
}

function buildDeveloperTraceableMessage(error) {
  if (error instanceof Error) {
    return `${UNEXPECTED_TAURI_ERROR_CODE}:${error.name}`;
  }

  if (typeof error === "string" && error) {
    return `${UNEXPECTED_TAURI_ERROR_CODE}:string`;
  }

  if (error && typeof error === "object") {
    return `${UNEXPECTED_TAURI_ERROR_CODE}:object`;
  }

  return `${UNEXPECTED_TAURI_ERROR_CODE}:unknown`;
}

function normalizeRunDedupeResult(result) {
  const response = result && typeof result === "object" ? result : {};

  return {
    ...response,
    warnings: Array.isArray(response.warnings)
      ? response.warnings.filter((warning) => typeof warning === "string")
      : [],
    warningDetails: Array.isArray(response.warningDetails)
      ? response.warningDetails.filter(isStructuredWarningDetail)
      : [],
  };
}

function isStructuredReviewGroup(group) {
  return Boolean(
    group &&
      typeof group === "object" &&
      typeof group.groupId === "string" &&
      Array.isArray(group.images) &&
      group.images.every((image) => typeof image === "string") &&
      typeof group.suggested === "string",
  );
}

function normalizeReviewResult(result) {
  const response = result && typeof result === "object" ? result : {};

  return {
    groups: Array.isArray(response.groups) ? response.groups.filter(isStructuredReviewGroup) : [],
    warnings: Array.isArray(response.warnings)
      ? response.warnings.filter((warning) => typeof warning === "string")
      : [],
    warningDetails: Array.isArray(response.warningDetails)
      ? response.warningDetails.filter(isStructuredWarningDetail)
      : [],
  };
}

const UPDATER_CHECK_COMMAND = "plugin:updater|check";
const UPDATER_DOWNLOAD_AND_INSTALL_COMMAND = "plugin:updater|download_and_install";

export function createTauriClient(tauri = {}) {
  const { core, dialog, event } = tauri;

  return {
    async chooseDirectory(title) {
      if (!dialog?.open) {
        return null;
      }

      return dialog.open({
        directory: true,
        multiple: false,
        title,
      });
    },

    async listenToProgress(listener) {
      if (!event?.listen) {
        return null;
      }

      return event.listen(DEDUPE_PROGRESS_EVENT, (eventPayload) => {
        const payload = eventPayload?.payload ?? eventPayload;
        if (!isStructuredProgressPayload(payload)) {
          return;
        }

        listener(payload);
      });
    },

    async runDedupe({
      inputDir,
      outputDir,
      minSizeMb,
      similarityThreshold = 10,
      filterExistingOutput = true,
      locale = "en",
      runId,
    }) {
      if (!core?.invoke) {
        throw new Error("Tauri APIs not available.");
      }

      try {
        const response = await core.invoke(RUN_DEDUPE_COMMAND, {
          config: {
            inputDir,
            outputDir,
            similarityThreshold,
            minImageSizeBytes: Math.round(minSizeMb * 1024 * 1024),
            filterExistingOutput,
            runId,
            locale,
          },
        });

        return normalizeRunDedupeResult(response);
      } catch (error) {
        throw normalizeRunDedupeError(error);
      }
    },

    async prepareDedupeReview({
      inputDir,
      outputDir,
      minSizeMb,
      similarityThreshold = 10,
      filterExistingOutput = true,
      locale = "en",
      runId,
    }) {
      if (!core?.invoke) {
        throw new Error("Tauri APIs not available.");
      }

      try {
        const response = await core.invoke(PREPARE_DEDUPE_REVIEW_COMMAND, {
          config: {
            inputDir,
            outputDir,
            similarityThreshold,
            minImageSizeBytes: Math.round(minSizeMb * 1024 * 1024),
            filterExistingOutput,
            runId,
            locale,
          },
        });

        return normalizeReviewResult(response);
      } catch (error) {
        throw normalizeRunDedupeError(error);
      }
    },

    async exportDedupeReview({
      outputDir,
      similarityThreshold = 10,
      filterExistingOutput = true,
      exportSimilarImageGroups = true,
      locale = "en",
      groups = [],
      selections = {},
      runId,
    }) {
      if (!core?.invoke) {
        throw new Error("Tauri APIs not available.");
      }

      try {
        const response = await core.invoke(EXPORT_DEDUPE_REVIEW_COMMAND, {
          config: {
            outputDir,
            similarityThreshold,
            filterExistingOutput,
            exportSimilarImageGroups,
            groups,
            selections,
            runId,
            locale,
          },
        });

        return normalizeRunDedupeResult(response);
      } catch (error) {
        throw normalizeRunDedupeError(error);
      }
    },

    async cancelDedupe() {
      if (!core?.invoke) {
        return false;
      }

      try {
        return Boolean(await core.invoke(CANCEL_DEDUPE_COMMAND));
      } catch (error) {
        throw normalizeRunDedupeError(error);
      }
    },

    async isMsixInstall() {
      if (!core?.invoke) {
        return false;
      }

      try {
        return Boolean(await core.invoke(IS_MSIX_INSTALL_COMMAND));
      } catch {
        return false;
      }
    },

    async checkForUpdate() {
      if (!core?.invoke) {
        return null;
      }

      const metadata = await core.invoke(UPDATER_CHECK_COMMAND, {});
      if (!metadata) {
        return null;
      }

      return {
        rid: metadata.rid,
        version: metadata.version,
        currentVersion: metadata.currentVersion,
        body: metadata.body ?? null,
      };
    },

    async installUpdate(rid) {
      if (!core?.invoke || !core?.Channel) {
        return false;
      }

      await core.invoke(UPDATER_DOWNLOAD_AND_INSTALL_COMMAND, {
        rid,
        onEvent: new core.Channel(),
      });
      return true;
    },

    isAvailable() {
      return Boolean(core?.invoke && dialog?.open);
    },
  };
}
