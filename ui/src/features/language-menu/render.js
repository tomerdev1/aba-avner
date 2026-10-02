export function renderLanguageMenu(state, elements, t) {
  if (elements.languageButton) {
    elements.languageButton.setAttribute("aria-label", t("app.openLanguageMenu"));
    elements.languageButton.setAttribute("aria-expanded", String(state.isLanguageMenuOpen));
    elements.languageButton.textContent = getLocaleFlag(state.locale);
  }
  if (elements.languageMenu) {
    elements.languageMenu.hidden = !state.isLanguageMenuOpen;
    elements.languageMenu.setAttribute("aria-hidden", String(!state.isLanguageMenuOpen));
  }
  if (elements.localeEnButton) {
    elements.localeEnButton.textContent = `${getLocaleFlag("en")} ${t("locales.en")}`;
    elements.localeEnButton.setAttribute("aria-pressed", String(state.locale === "en"));
    elements.localeEnButton.setAttribute("tabindex", state.isLanguageMenuOpen ? "0" : "-1");
  }
  if (elements.localeHeButton) {
    elements.localeHeButton.textContent = `${getLocaleFlag("he")} ${t("locales.he")}`;
    elements.localeHeButton.setAttribute("aria-pressed", String(state.locale === "he"));
    elements.localeHeButton.setAttribute("tabindex", state.isLanguageMenuOpen ? "0" : "-1");
  }
}

function getLocaleFlag(locale) {
  return locale === "he" ? "🇮🇱" : "🇺🇸";
}
