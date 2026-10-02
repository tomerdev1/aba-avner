import { setProgress } from "../../app/progress.js";

export function renderProgressSection(progress, elements, t, runStatus = "idle") {
  if (elements.progressBar) {
    const ariaLabel = t("progress.label");
    if (elements.progressBar.getAttribute("aria-label") !== ariaLabel) {
      elements.progressBar.setAttribute("aria-label", ariaLabel);
    }
  }

  setProgress(localizeProgressStage(progress.stage, t), progress.current, progress.total, elements, {
    completed:
      runStatus === "success" || runStatus === "success_with_warnings",
    progressKey: progress.stage,
  });
}

function localizeProgressStage(stage, t) {
  const knownStages = {
    Idle: t("app.idle"),
    Working: t("app.working"),
    Starting: t("app.starting"),
    Cancelling: t("app.cancelling"),
    scan: t("progress.scan"),
    hash: t("progress.hash"),
    group: t("progress.group"),
    review: t("progress.review"),
    copy: t("progress.copy"),
  };

  return knownStages[stage] || stage;
}
