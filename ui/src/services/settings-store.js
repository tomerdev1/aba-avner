export const DEFAULT_MIN_SIZE_MB = 0;
export const DEFAULT_SIMILARITY_THRESHOLD = 10;
export const DEFAULT_FILTER_EXISTING_OUTPUT = true;
export const DEFAULT_VISUAL_REVIEW_ENABLED = true;
export const DEFAULT_EXPORT_SIMILAR_IMAGE_GROUPS = true;

const MIN_SIZE_STORAGE_KEY = "aba_avner.min_size_mb";
const SIMILARITY_THRESHOLD_STORAGE_KEY = "aba_avner.similarity_threshold";
const FILTER_EXISTING_OUTPUT_STORAGE_KEY = "aba_avner.filter_existing_output";
const VISUAL_REVIEW_ENABLED_STORAGE_KEY = "aba_avner.visual_review_enabled";
const EXPORT_SIMILAR_IMAGE_GROUPS_STORAGE_KEY = "aba_avner.export_similar_image_groups";

export function createSettingsStore(options = {}) {
  const storage = options.storage ?? globalThis.localStorage;

  return {
    getMinSizeMb() {
      try {
        const raw = storage?.getItem?.(MIN_SIZE_STORAGE_KEY);
        if (raw == null) {
          return DEFAULT_MIN_SIZE_MB;
        }

        const value = Number.parseFloat(raw);
        return Number.isFinite(value) ? value : DEFAULT_MIN_SIZE_MB;
      } catch {
        return DEFAULT_MIN_SIZE_MB;
      }
    },

    setMinSizeMb(value) {
      try {
        storage?.setItem?.(MIN_SIZE_STORAGE_KEY, String(value));
      } catch {
        // Ignore storage failures and keep settings functional in-memory.
      }
    },

    getSimilarityThreshold() {
      try {
        const raw = storage?.getItem?.(SIMILARITY_THRESHOLD_STORAGE_KEY);
        if (raw == null) {
          return DEFAULT_SIMILARITY_THRESHOLD;
        }

        const value = Number.parseInt(raw, 10);
        return Number.isInteger(value) ? value : DEFAULT_SIMILARITY_THRESHOLD;
      } catch {
        return DEFAULT_SIMILARITY_THRESHOLD;
      }
    },

    setSimilarityThreshold(value) {
      try {
        storage?.setItem?.(SIMILARITY_THRESHOLD_STORAGE_KEY, String(value));
      } catch {
        // Ignore storage failures and keep settings functional in-memory.
      }
    },

    getFilterExistingOutput() {
      try {
        const raw = storage?.getItem?.(FILTER_EXISTING_OUTPUT_STORAGE_KEY);
        if (raw == null) {
          return DEFAULT_FILTER_EXISTING_OUTPUT;
        }

        return raw !== "false";
      } catch {
        return DEFAULT_FILTER_EXISTING_OUTPUT;
      }
    },

    setFilterExistingOutput(value) {
      try {
        storage?.setItem?.(FILTER_EXISTING_OUTPUT_STORAGE_KEY, String(Boolean(value)));
      } catch {
        // Ignore storage failures and keep settings functional in-memory.
      }
    },

    getVisualReviewEnabled() {
      try {
        const raw = storage?.getItem?.(VISUAL_REVIEW_ENABLED_STORAGE_KEY);
        if (raw == null) {
          return DEFAULT_VISUAL_REVIEW_ENABLED;
        }

        return raw !== "false";
      } catch {
        return DEFAULT_VISUAL_REVIEW_ENABLED;
      }
    },

    setVisualReviewEnabled(value) {
      try {
        storage?.setItem?.(VISUAL_REVIEW_ENABLED_STORAGE_KEY, String(Boolean(value)));
      } catch {
        // Ignore storage failures and keep settings functional in-memory.
      }
    },

    getExportSimilarImageGroups() {
      try {
        const raw = storage?.getItem?.(EXPORT_SIMILAR_IMAGE_GROUPS_STORAGE_KEY);
        if (raw == null) {
          return DEFAULT_EXPORT_SIMILAR_IMAGE_GROUPS;
        }

        return raw !== "false";
      } catch {
        return DEFAULT_EXPORT_SIMILAR_IMAGE_GROUPS;
      }
    },

    setExportSimilarImageGroups(value) {
      try {
        storage?.setItem?.(EXPORT_SIMILAR_IMAGE_GROUPS_STORAGE_KEY, String(Boolean(value)));
      } catch {
        // Ignore storage failures and keep settings functional in-memory.
      }
    },
  };
}
