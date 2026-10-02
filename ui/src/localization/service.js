import { enMessages } from "./messages/en.js";
import { heMessages } from "./messages/he.js";

const DEFAULT_LOCALE = "en";
const LOCALE_STORAGE_KEY = "aba_avner.locale";

const catalogs = {
  en: enMessages,
  he: heMessages,
};

const localeDirections = {
  en: "ltr",
  he: "rtl",
};

export function createLocalizationService(options = {}) {
  const storage = options.storage ?? globalThis.localStorage;
  const documentRef = options.documentRef ?? globalThis.document;
  const listeners = new Set();
  let locale = normalizeLocale(readStoredLocale(storage));

  applyDocument(locale, documentRef);

  return {
    getLocale() {
      return locale;
    },

    setLocale(nextLocale) {
      const normalized = normalizeLocale(nextLocale);
      if (normalized === locale) {
        return;
      }

      locale = normalized;
      writeStoredLocale(storage, locale);
      applyDocument(locale, documentRef);

      for (const listener of listeners) {
        listener(locale);
      }
    },

    subscribe(listener) {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },

    t(key, params = {}) {
      const template = resolveMessage(catalogs[locale], key) ?? resolveMessage(catalogs.en, key) ?? key;
      return interpolate(template, params);
    },
  };
}

function normalizeLocale(locale) {
  return catalogs[locale] ? locale : DEFAULT_LOCALE;
}

function readStoredLocale(storage) {
  try {
    return storage?.getItem?.(LOCALE_STORAGE_KEY) || DEFAULT_LOCALE;
  } catch {
    return DEFAULT_LOCALE;
  }
}

function writeStoredLocale(storage, locale) {
  try {
    storage?.setItem?.(LOCALE_STORAGE_KEY, locale);
  } catch {
    // Ignore storage failures and keep localization functional in-memory.
  }
}

function applyDocument(locale, documentRef) {
  if (!documentRef?.documentElement) {
    return;
  }

  documentRef.documentElement.lang = locale;
  documentRef.documentElement.dir = localeDirections[locale] || "ltr";

  if (documentRef.title != null) {
    documentRef.title = interpolate(
      resolveMessage(catalogs[locale], "app.title") || resolveMessage(catalogs.en, "app.title"),
      {},
    );
  }
}

function resolveMessage(catalog, key) {
  return key.split(".").reduce((value, part) => value?.[part], catalog);
}

function interpolate(template, params) {
  if (typeof template !== "string") {
    return "";
  }

  return template.replace(/\{(\w+)\}/g, (_match, token) => String(params[token] ?? ""));
}
