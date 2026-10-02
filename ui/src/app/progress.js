const PIPELINE_STAGE_ORDER = ["scan", "hash", "group", "review", "copy"];

export function setProgress(stage, current, total, elements, options = {}) {
  const percent = calculateProgressPercent(
    options.progressKey || stage,
    current,
    total,
    options,
  );

  if (!elements.progressBar || !elements.progressStage || !elements.progressPercent) {
    return;
  }

  setStyleWidthIfChanged(elements.progressBar, `${percent}%`);
  setTextContentIfChanged(elements.progressStage, stage);
  setTextContentIfChanged(elements.progressPercent, `${percent}%`);
  setAttributeIfChanged(elements.progressBar, "aria-valuenow", String(percent));
  setAttributeIfChanged(elements.progressBar, "aria-valuetext", `${stage} ${percent}%`);
}

export function calculateProgressPercent(stage, current, total, options = {}) {
  if (options.completed) {
    return 100;
  }

  const safeTotal = Math.max(total, 1);
  const clampedCurrent = Math.min(Math.max(current, 0), safeTotal);
  const stageIndex = PIPELINE_STAGE_ORDER.indexOf(stage);

  if (stageIndex === -1) {
    return Math.min(100, Math.round((clampedCurrent / safeTotal) * 100));
  }

  const stageRatio = clampedCurrent / safeTotal;
  const overallRatio = (stageIndex + stageRatio) / PIPELINE_STAGE_ORDER.length;
  return Math.min(100, Math.round(overallRatio * 100));
}

export function formatMegabytes(value, t = defaultTranslator) {
  if (!Number.isFinite(value)) {
    return `0 ${t("units.megabytes")}`;
  }

  if (Number.isInteger(value)) {
    return `${value} ${t("units.megabytes")}`;
  }

  return `${value.toFixed(1)} ${t("units.megabytes")}`;
}

export function updateMinSizeLabel(value, elements, t = defaultTranslator) {
  if (!elements.minSizeValue) {
    return;
  }

  elements.minSizeValue.textContent = formatMegabytes(value, t);
}

function defaultTranslator() {
  return "MB";
}

function setAttributeIfChanged(element, name, value) {
  if (element.getAttribute(name) !== value) {
    element.setAttribute(name, value);
  }
}

function setTextContentIfChanged(element, value) {
  if (element.textContent !== value) {
    element.textContent = value;
  }
}

function setStyleWidthIfChanged(element, value) {
  if (element.style.width !== value) {
    element.style.width = value;
  }
}
