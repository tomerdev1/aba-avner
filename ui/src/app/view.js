function getActiveCard(elements) {
  if (!elements.cardStage || !elements.mainCard || !elements.settingsCard) {
    return null;
  }

  return elements.cardStage.dataset.view === "settings"
    ? elements.settingsCard
    : elements.mainCard;
}

export function syncStageHeight(elements) {
  if (!elements.cardStage) {
    return;
  }

  const activeCard = getActiveCard(elements);
  if (!activeCard) {
    return;
  }

  requestAnimationFrame(() => {
    const nextHeight = Math.ceil(activeCard.getBoundingClientRect().height);
    elements.cardStage.style.height = `${nextHeight}px`;
  });
}

export function setView(view, elements) {
  if (!elements.cardStage || !elements.settingsButton || !elements.settingsCard || !elements.mainCard) {
    return;
  }

  const isSettings = view === "settings";
  elements.cardStage.dataset.view = isSettings ? "settings" : "main";
  elements.settingsButton.setAttribute("aria-expanded", String(isSettings));
  elements.mainCard.setAttribute("aria-hidden", String(isSettings));
  elements.settingsCard.setAttribute("aria-hidden", String(!isSettings));
}
