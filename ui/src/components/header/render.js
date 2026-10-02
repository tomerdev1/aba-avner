import { isBusyStatus } from "../../app/state.js";

export function renderHeader(state, elements, t) {
  if (elements.appEyebrow) {
    elements.appEyebrow.textContent = t("app.eyebrow");
  }
  if (elements.appTitle) {
    elements.appTitle.textContent = t("app.title");
  }
  if (elements.appSubtitle) {
    elements.appSubtitle.textContent = t("app.subtitle");
  }
  if (elements.startButton) {
    elements.startButton.textContent = isBusyStatus(state.runStatus) ? t("app.cancelRun") : t("app.startRun");
  }
  if (elements.settingsButton) {
    elements.settingsButton.setAttribute("aria-label", t("app.openSettings"));
  }
}
