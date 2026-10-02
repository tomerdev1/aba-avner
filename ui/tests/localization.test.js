import { describe, expect, it, vi } from "vitest";

import { createLocalizationService } from "../src/localization/service.js";

function createStorage(initial = {}) {
  const values = new Map(Object.entries(initial));

  return {
    getItem(key) {
      return values.has(key) ? values.get(key) : null;
    },
    setItem(key, value) {
      values.set(key, value);
    },
  };
}

function createDocumentRef() {
  return {
    title: "",
    documentElement: {
      lang: "",
      dir: "",
    },
  };
}

describe("localization service", () => {
  it("defaults to english and updates document metadata", () => {
    const documentRef = createDocumentRef();
    const service = createLocalizationService({
      storage: createStorage(),
      documentRef,
    });

    expect(service.getLocale()).toBe("en");
    expect(service.t("app.eyebrow")).toBe("Visual duplicate remover");
    expect(documentRef.documentElement.lang).toBe("en");
    expect(documentRef.documentElement.dir).toBe("ltr");
    expect(documentRef.title).toBe("Aba Avner");
  });

  it("restores a persisted locale and supports rtl metadata", () => {
    const documentRef = createDocumentRef();
    const service = createLocalizationService({
      storage: createStorage({ "aba_avner.locale": "he" }),
      documentRef,
    });

    expect(service.getLocale()).toBe("he");
    expect(service.t("settings.title")).toBe("הגדרות");
    expect(documentRef.documentElement.lang).toBe("he");
    expect(documentRef.documentElement.dir).toBe("rtl");
  });

  it("persists locale changes and notifies subscribers", () => {
    const storage = createStorage();
    const documentRef = createDocumentRef();
    const service = createLocalizationService({
      storage,
      documentRef,
    });
    const listener = vi.fn();

    service.subscribe(listener);
    service.setLocale("he");

    expect(storage.getItem("aba_avner.locale")).toBe("he");
    expect(listener).toHaveBeenCalledWith("he");
    expect(documentRef.title).toBe("אבא אבנר");
  });

  it("falls back to english when the persisted locale is unsupported", () => {
    const documentRef = createDocumentRef();
    const service = createLocalizationService({
      storage: createStorage({ "aba_avner.locale": "fr" }),
      documentRef,
    });

    expect(service.getLocale()).toBe("en");
    expect(service.t("settings.title")).toBe("Settings");
    expect(documentRef.documentElement.lang).toBe("en");
    expect(documentRef.documentElement.dir).toBe("ltr");
  });
});
