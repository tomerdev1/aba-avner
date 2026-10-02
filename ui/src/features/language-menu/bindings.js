export function bindLanguageMenuInteractions(elements, app, options = {}) {
  const documentRef = options.documentRef ?? globalThis.document;
  const selectedLocaleButton = () =>
    elements.localeHeButton?.getAttribute("aria-pressed") === "true"
      ? elements.localeHeButton
      : elements.localeEnButton;
  const localeButtons = () => [elements.localeEnButton, elements.localeHeButton].filter(Boolean);

  function focusLanguageTrigger() {
    elements.languageButton?.focus();
  }

  function focusSelectedLocale() {
    (selectedLocaleButton() ?? localeButtons()[0])?.focus();
  }

  function isLanguageMenuOpen() {
    return elements.languageMenu?.hidden === false;
  }

  function closeLanguageMenuAndRestoreFocus() {
    if (!isLanguageMenuOpen()) {
      return;
    }

    const shouldRestoreFocus = elements.languageMenu?.contains(documentRef?.activeElement);
    app.closeLanguageMenu();
    if (shouldRestoreFocus) {
      focusLanguageTrigger();
    }
  }

  function moveLocaleFocus(direction) {
    const buttons = localeButtons();
    if (!buttons.length) {
      return;
    }

    const currentIndex = buttons.indexOf(documentRef?.activeElement);
    const baseIndex = currentIndex === -1 ? buttons.indexOf(selectedLocaleButton()) : currentIndex;
    const nextIndex = (baseIndex + direction + buttons.length) % buttons.length;
    buttons[nextIndex]?.focus();
  }

  elements.languageButton?.addEventListener("click", () => {
    app.toggleLanguageMenu();
    if (!elements.languageMenu?.hidden) {
      focusSelectedLocale();
    }
  });
  elements.localeEnButton?.addEventListener("click", () => {
    app.setLocale("en");
    focusLanguageTrigger();
  });
  elements.localeHeButton?.addEventListener("click", () => {
    app.setLocale("he");
    focusLanguageTrigger();
  });

  elements.languageButton?.addEventListener("keydown", (event) => {
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") {
      return;
    }

    event.preventDefault();
    if (elements.languageMenu?.hidden) {
      app.toggleLanguageMenu();
    }
    if (!elements.languageMenu?.hidden) {
      focusSelectedLocale();
    }
  });

  elements.languageMenu?.addEventListener("keydown", (event) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      moveLocaleFocus(1);
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      moveLocaleFocus(-1);
      return;
    }

    if (event.key === "Home") {
      event.preventDefault();
      localeButtons()[0]?.focus();
      return;
    }

    if (event.key === "End") {
      event.preventDefault();
      localeButtons().at(-1)?.focus();
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      closeLanguageMenuAndRestoreFocus();
      return;
    }

    if (event.key === "Tab") {
      app.closeLanguageMenu();
    }
  });

  documentRef?.addEventListener("click", (event) => {
    if (!elements.headerControls?.contains(event.target)) {
      closeLanguageMenuAndRestoreFocus();
    }
  });

  documentRef?.addEventListener("keydown", (event) => {
    if (event.defaultPrevented) {
      return;
    }

    if (event.key === "Escape") {
      closeLanguageMenuAndRestoreFocus();
    }
  });
}
