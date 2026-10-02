import { describe, expect, it, vi } from "vitest";

import { createUiLogger } from "../src/services/ui-logger.js";

function createWindowRef() {
  const listeners = new Map();

  return {
    addEventListener(type, listener) {
      listeners.set(type, listener);
    },
    dispatch(type, event) {
      const listener = listeners.get(type);
      if (listener) {
        listener(event);
      }
    },
  };
}

describe("ui logger", () => {
  it("logs unexpected window errors once handlers are installed", () => {
    const consoleRef = { error: vi.fn() };
    const windowRef = createWindowRef();
    const logger = createUiLogger({ consoleRef, windowRef });

    logger.installGlobalErrorHandlers();

    windowRef.dispatch("error", {
      message: "render exploded",
      filename: "app.js",
      lineno: 12,
      colno: 4,
      error: new Error("render exploded"),
    });

    expect(consoleRef.error).toHaveBeenCalledWith(
      "Unexpected UI error",
      expect.objectContaining({
        message: "render exploded",
        filename: "app.js",
        lineno: 12,
        colno: 4,
        error: expect.any(Error),
      }),
    );
  });

  it("logs unexpected unhandled promise rejections", () => {
    const consoleRef = { error: vi.fn() };
    const windowRef = createWindowRef();
    const logger = createUiLogger({ consoleRef, windowRef });

    logger.installGlobalErrorHandlers();

    windowRef.dispatch("unhandledrejection", {
      reason: new Error("async exploded"),
    });

    expect(consoleRef.error).toHaveBeenCalledWith(
      "Unhandled UI promise rejection",
      expect.objectContaining({
        reason: expect.any(Error),
      }),
    );
  });
});
