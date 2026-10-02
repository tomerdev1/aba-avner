export async function checkForAppUpdate(tauriClient, localization, confirm = globalThis.confirm) {
  if (!tauriClient?.checkForUpdate || typeof confirm !== "function") {
    return;
  }

  try {
    // MSIX (Store) installs update via Windows Update automatically - this
    // self-update flow only applies to the unpackaged NSIS distribution.
    if (await tauriClient.isMsixInstall?.()) {
      return;
    }

    const update = await tauriClient.checkForUpdate();
    if (!update) {
      return;
    }

    const shouldInstall = confirm(
      localization.t("updates.confirmInstall", { version: update.version }),
    );
    if (!shouldInstall) {
      return;
    }

    await tauriClient.installUpdate(update.rid);
  } catch (error) {
    console.error("Update check failed", error);
  }
}
