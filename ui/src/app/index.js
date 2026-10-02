import { bindAppInteractions } from "./bindings.js";
import { createAppController } from "./controller.js";
import { getElements, getTauriApi } from "./dom.js";
import { renderApp } from "./render.js";
import { formatMegabytes, setProgress, updateMinSizeLabel } from "./progress.js";
import { setView, syncStageHeight } from "./view.js";
import { createTauriClient } from "../services/tauri-client.js";
import { createUiLogger } from "../services/ui-logger.js";
import { createLocalizationService } from "../localization/service.js";
import { createSettingsStore } from "../services/settings-store.js";
import { checkForAppUpdate } from "../services/update-check.js";

export { formatMegabytes, setProgress, setView, syncStageHeight, updateMinSizeLabel };

export function createApp(elements = getElements(), tauri = getTauriApi()) {
  const localization = createLocalizationService();
  const settingsStore = createSettingsStore();
  const tauriClient = createTauriClient(tauri);
  let latestState = null;
  const render = (state) => {
    latestState = state;
    renderApp(state, elements, localization);
  };
  const app = createAppController(elements, tauriClient, localization, settingsStore, render);

  localization.subscribe(() => {
    if (latestState) {
      renderApp(latestState, elements, localization);
    }
  });

  return {
    ...app,
    getLocale: localization.getLocale,
    tauriClient,
    localization,
  };
}

export function initApp() {
  const uiLogger = createUiLogger();
  uiLogger.installGlobalErrorHandlers();

  const elements = getElements();
  const app = createApp(elements, getTauriApi());

  if (typeof window !== "undefined" && window.__ABA_AVNER_TESTING__) {
    window.__ABA_AVNER_TEST_API__ = app;
  }

  bindAppInteractions(elements, app);

  if (typeof window === "undefined" || !window.__ABA_AVNER_TESTING__) {
    checkForAppUpdate(app.tauriClient, app.localization);
  }

  return app;
}
