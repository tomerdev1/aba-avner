function buildImagePreviewSrc(path) {
  if (typeof path !== "string" || path.length === 0) {
    return "";
  }

  const maybeConvert = globalThis.window?.__TAURI__?.core?.convertFileSrc;
  if (typeof maybeConvert === "function") {
    return maybeConvert(path);
  }

  if (/^(data:|blob:|https?:|file:)/.test(path)) {
    return path;
  }

  if (/^[a-zA-Z]:[\\/]/.test(path)) {
    return encodeURI(`file:///${path.replace(/\\/g, "/")}`);
  }

  if (path.startsWith("/")) {
    return encodeURI(`file://${path}`);
  }

  return path;
}

function basename(path) {
  if (typeof path !== "string") {
    return "";
  }

  if (/^(data:|blob:)/.test(path)) {
    return "Preview image";
  }

  return path.split(/[\\/]/).pop() || path;
}

export function renderReviewPanel(review, runStatus, elements, localization) {
  if (!elements.reviewPanel) {
    return;
  }

  const groups = Array.isArray(review?.groups) ? review.groups.filter((group) => group.images.length > 1) : [];
  const hasReview = Array.isArray(review?.groups) && review.groups.length > 0;

  elements.reviewPanel.hidden = !hasReview;
  if (!hasReview) {
    if (elements.reviewGroups) {
      elements.reviewGroups.replaceChildren();
    }
    return;
  }

  elements.reviewBadge.textContent = localization.t("review.badge");
  elements.reviewTitle.textContent = localization.t("review.title");
  elements.reviewHint.textContent = localization.t("review.hint");
  elements.reviewApplyButton.textContent = localization.t("review.apply");
  elements.reviewCancelButton.textContent = localization.t("review.cancel");
  elements.reviewApplyButton.disabled = runStatus === "running" || runStatus === "cancelling";
  elements.reviewCancelButton.disabled = runStatus === "running" || runStatus === "cancelling";

  elements.reviewEmpty.hidden = groups.length !== 0;
  elements.reviewEmpty.textContent = localization.t("review.empty");

  const groupNodes = groups.map((group, index) => {
    const section = document.createElement("section");
    section.className = "app-review__group";
    section.setAttribute("data-group-id", group.groupId);

    const title = document.createElement("h3");
    title.className = "app-review__group-title";
    title.textContent = localization.t("review.groupLabel", { index: index + 1 });

    const grid = document.createElement("div");
    grid.className = "app-review__grid";

    for (const imagePath of group.images) {
      const button = document.createElement("button");
      const isSelected = review.selections?.[group.groupId] === imagePath;
      button.className = "app-review__image";
      button.type = "button";
      button.dataset.groupId = group.groupId;
      button.dataset.imagePath = imagePath;
      button.setAttribute("aria-pressed", String(isSelected));
      if (isSelected) {
        button.classList.add("is-selected");
      }

      const preview = document.createElement("img");
      preview.className = "app-review__thumb";
      preview.alt = basename(imagePath);
      preview.src = buildImagePreviewSrc(imagePath);
      preview.loading = "lazy";

      const label = document.createElement("span");
      label.className = "app-review__filename";
      label.textContent = basename(imagePath);
      label.setAttribute("dir", "auto");
      label.title = imagePath;

      button.append(preview, label);
      grid.append(button);
    }

    section.append(title, grid);
    return section;
  });

  elements.reviewGroups.replaceChildren(...groupNodes);
}
