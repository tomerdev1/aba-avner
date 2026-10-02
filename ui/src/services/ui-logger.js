const UI_ERROR_HANDLER_FLAG = "__ABA_AVNER_UI_ERROR_HANDLERS_INSTALLED__";

export function createUiLogger({ consoleRef = console, windowRef = window } = {}) {
  return {
    installGlobalErrorHandlers() {
      if (!windowRef?.addEventListener || windowRef[UI_ERROR_HANDLER_FLAG]) {
        return;
      }

      windowRef.addEventListener("error", (event) => {
        logUiFailure(consoleRef, "Unexpected UI error", {
          message: event?.message || "",
          filename: event?.filename || "",
          lineno: event?.lineno || 0,
          colno: event?.colno || 0,
          error: event?.error || null,
        });
      });

      windowRef.addEventListener("unhandledrejection", (event) => {
        logUiFailure(consoleRef, "Unhandled UI promise rejection", {
          reason: event?.reason || null,
        });
      });

      windowRef[UI_ERROR_HANDLER_FLAG] = true;
    },
  };
}

function logUiFailure(consoleRef, message, details) {
  consoleRef.error(message, details);
}
