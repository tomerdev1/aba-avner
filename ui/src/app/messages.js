import { isStructuredTauriError } from "../services/tauri-client.js";

const errorMessageKeys = {
  cancelled: "issues.errors.cancelled",
  invalid_input_dir: "issues.errors.invalidInputDir",
  invalid_output_dir: "issues.errors.invalidOutputDir",
  invalid_selection: "issues.errors.invalidSelection",
  input_output_conflict: "issues.errors.inputOutputConflict",
  run_in_progress: "issues.errors.runInProgress",
  read_dir_failed: "issues.errors.readDirFailed",
  load_image_failed: "issues.errors.loadImageFailed",
  copy_image_failed: "issues.errors.copyImageFailed",
};

const warningMessageKeys = {
  file_issue: "issues.warningItems.fileIssue",
  output_dir_scan_failed: "issues.warningItems.outputDirScanFailed",
  similar_image_in_output: "issues.warningItems.similarImageInOutput",
};

export function buildTauriUnavailableIssues(t) {
  return {
    title: t("issues.tauriUnavailableTitle"),
    hint: t("issues.tauriUnavailableHint"),
    items: [t("issues.tauriUnavailableItem")],
  };
}

export function buildWarningsIssues(t, result) {
  const items = resolveWarningItems(t, result);

  if (!items.length) {
    return {
      title: "",
      hint: "",
      items: [],
    };
  }

  return {
    title: t("issues.warningsTitle", { count: items.length }),
    hint: t("issues.warningsHint"),
    items,
  };
}

export function buildFailureIssues(t, error) {
  if (isStructuredTauriError(error) && error.code === "cancelled") {
    return {
      title: t("issues.runCancelledTitle"),
      hint: t("issues.runCancelledHint"),
      items: [t("issues.errors.cancelled")],
    };
  }

  const message = resolveFailureMessage(t, error);

  return {
    title: t("issues.runFailedTitle"),
    hint: t("issues.runFailedHint"),
    items: [message],
  };
}

function resolveFailureMessage(t, error) {
  if (!isStructuredTauriError(error)) {
    return t("issues.errors.unexpected", { code: "unstructured_error" });
  }

  const knownMessageKey = errorMessageKeys[error.code];
  if (knownMessageKey) {
    return t(knownMessageKey);
  }

  return t("issues.errors.unexpected", { code: error.message });
}

function resolveWarningItems(t, result) {
  if (Array.isArray(result)) {
    return result;
  }

  const warnings = Array.isArray(result?.warnings) ? result.warnings : [];
  const warningDetails = Array.isArray(result?.warningDetails) ? result.warningDetails : [];

  if (!warningDetails.length) {
    return warnings;
  }

  const structuredItems = warningDetails.map((warning) => {
    const localized = localizeWarningItem(t, warning);
    if (localized) {
      return localized;
    }

    return fallbackWarningMessage(warning);
  });
  const extraRawWarnings = warnings.slice(warningDetails.length);

  return [...structuredItems, ...extraRawWarnings];
}

function localizeWarningItem(t, warning) {
  if (!warning || typeof warning.code !== "string") {
    return "";
  }

  const messageKey = warningMessageKeys[warning.code];
  if (!messageKey) {
    return "";
  }

  return t(messageKey, {
    path: warning.path || "",
    detail: warning.detail || "",
  });
}

function fallbackWarningMessage(warning) {
  if (!warning || typeof warning !== "object") {
    return "";
  }

  if (typeof warning.path === "string" && typeof warning.detail === "string") {
    return `${warning.path}: ${warning.detail}`;
  }

  if (typeof warning.path === "string") {
    return warning.path;
  }

  return "";
}
