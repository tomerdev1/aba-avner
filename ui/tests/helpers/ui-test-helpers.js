import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { getElements } from "../../src/app/dom.js";
import { createLocalizationService } from "../../src/localization/service.js";

export function loadHtml() {
  const html = readFileSync(resolve(process.cwd(), "index.html"), "utf8");
  document.open();
  document.write(html);
  document.close();
}

export function createRenderHarness(locale = "en") {
  loadHtml();

  const localization = createLocalizationService({
    storage: {
      getItem() {
        return locale;
      },
      setItem() {},
    },
    documentRef: document,
  });

  return {
    elements: getElements(),
    localization,
  };
}

export async function loadAppWithTauri(tauri) {
  window.__ABA_AVNER_TESTING__ = true;
  window.__TAURI__ = tauri;
  const appUrl = pathToFileURL(resolve(process.cwd(), "app.js"));
  appUrl.searchParams.set("t", `${Date.now()}-${Math.random()}`);
  await import(appUrl.toString());
}
