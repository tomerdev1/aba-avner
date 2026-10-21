import { LOCALES } from "../../localization/service.js";

export function renderLanguageMenu(state, elements, t) {
  if (elements.languageButton) {
    elements.languageButton.setAttribute("aria-label", t("app.openLanguageMenu"));
    elements.languageButton.setAttribute("aria-expanded", String(state.isLanguageMenuOpen));
    elements.languageButton.textContent = LOCALES.find(({ code }) => code === state.locale)?.flag ?? "🇺🇸";
  }
  if (elements.languageMenu) {
    elements.languageMenu.hidden = !state.isLanguageMenuOpen;
    elements.languageMenu.setAttribute("aria-hidden", String(!state.isLanguageMenuOpen));
  }
  for (const button of elements.localeButtons ?? []) {
    const locale = LOCALES.find(({ code }) => code === button.dataset.locale);
    button.textContent = `${locale.flag} ${locale.name}`;
    button.setAttribute("aria-pressed", String(state.locale === locale.code));
    button.setAttribute("tabindex", state.isLanguageMenuOpen ? "0" : "-1");
  }
}
