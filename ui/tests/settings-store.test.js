import { describe, expect, it } from "vitest";

import { createSettingsStore } from "../src/services/settings-store.js";

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

describe("settings store", () => {
  it("defaults to zero min size when nothing is stored", () => {
    const store = createSettingsStore({
      storage: createStorage(),
    });

    expect(store.getMinSizeMb()).toBe(0);
    expect(store.getSimilarityThreshold()).toBe(10);
    expect(store.getFilterExistingOutput()).toBe(true);
    expect(store.getVisualReviewEnabled()).toBe(true);
    expect(store.getExportSimilarImageGroups()).toBe(true);
  });

  it("restores and persists the min size value", () => {
    const storage = createStorage({ "aba_avner.min_size_mb": "2.5" });
    const store = createSettingsStore({ storage });

    expect(store.getMinSizeMb()).toBe(2.5);

    store.setMinSizeMb(3.4);

    expect(storage.getItem("aba_avner.min_size_mb")).toBe("3.4");
  });

  it("restores and persists the existing-output filter flag", () => {
    const storage = createStorage({ "aba_avner.filter_existing_output": "false" });
    const store = createSettingsStore({ storage });

    expect(store.getFilterExistingOutput()).toBe(false);

    store.setFilterExistingOutput(true);

    expect(storage.getItem("aba_avner.filter_existing_output")).toBe("true");
  });

  it("restores and persists the similarity threshold", () => {
    const storage = createStorage({ "aba_avner.similarity_threshold": "7" });
    const store = createSettingsStore({ storage });

    expect(store.getSimilarityThreshold()).toBe(7);

    store.setSimilarityThreshold(12);

    expect(storage.getItem("aba_avner.similarity_threshold")).toBe("12");
  });

  it("restores and persists the visual review flag", () => {
    const storage = createStorage({ "aba_avner.visual_review_enabled": "false" });
    const store = createSettingsStore({ storage });

    expect(store.getVisualReviewEnabled()).toBe(false);

    store.setVisualReviewEnabled(true);

    expect(storage.getItem("aba_avner.visual_review_enabled")).toBe("true");
  });

  it("restores and persists the similar-group export flag", () => {
    const storage = createStorage({ "aba_avner.export_similar_image_groups": "false" });
    const store = createSettingsStore({ storage });

    expect(store.getExportSimilarImageGroups()).toBe(false);

    store.setExportSimilarImageGroups(true);

    expect(storage.getItem("aba_avner.export_similar_image_groups")).toBe("true");
  });
});
