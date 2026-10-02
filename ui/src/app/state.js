export function createInitialState(
  minSizeMb = 0,
  similarityThreshold = 10,
  locale = "en",
  filterExistingOutput = true,
  visualReviewEnabled = true,
  exportSimilarImageGroups = true,
) {
  return {
    view: "main",
    runStatus: "idle",
    minSizeMb,
    similarityThreshold,
    filterExistingOutput,
    visualReviewEnabled,
    exportSimilarImageGroups,
    locale,
    isLanguageMenuOpen: false,
    isSummaryExpanded: false,
    progress: {
      stage: "Idle",
      current: 0,
      total: 1,
    },
    review: createEmptyReview(),
    summary: createEmptySummary(),
    issues: {
      title: "",
      hint: "",
      items: [],
    },
  };
}

export function normalizeProgress(payload) {
  const data = payload?.payload || payload || {};

  return {
    stage: data.stage || "Working",
    current: data.current || 0,
    total: data.total || 1,
  };
}

export function reduceAppState(state, action) {
  switch (action.type) {
    case "view/set":
      return {
        ...state,
        view: action.view === "settings" ? "settings" : "main",
        isLanguageMenuOpen: false,
      };
    case "run/start":
      return {
        ...state,
        runStatus: "running",
        view: "main",
        isLanguageMenuOpen: false,
        isSummaryExpanded: false,
        progress: {
          stage: "Idle",
          current: 0,
          total: 1,
        },
        review: createEmptyReview(),
        summary: createEmptySummary(),
        issues: {
          title: "",
          hint: "",
          items: [],
        },
      };
    case "run/cancelling":
      return {
        ...state,
        runStatus: "cancelling",
      };
    case "run/review-export-start":
      return {
        ...state,
        runStatus: "running",
        isLanguageMenuOpen: false,
        isSummaryExpanded: false,
        progress: {
          stage: "Idle",
          current: 0,
          total: 1,
        },
        summary: createEmptySummary(),
        issues: {
          title: "",
          hint: "",
          items: [],
        },
      };
    case "run/complete":
      return {
        ...state,
        runStatus: normalizeRunStatus(action.status),
      };
    case "progress/set":
      return reduceProgressUpdate(state, action.payload);
    case "issues/set":
      return {
        ...state,
        issues: {
          title: action.issues?.title || "",
          hint: action.issues?.hint || "",
          items: action.issues?.items || [],
        },
      };
    case "summary/set":
      return {
        ...state,
        isSummaryExpanded: false,
        review: createEmptyReview(),
        summary: normalizeSummary(action.summary),
      };
    case "review/set":
      return {
        ...state,
        isSummaryExpanded: false,
        summary: createEmptySummary(),
        review: normalizeReview(action.review),
      };
    case "review/select":
      return selectReviewImage(state, action.groupId, action.path);
    case "review/clear":
      return {
        ...state,
        review: createEmptyReview(),
      };
    case "minSize/set":
      return {
        ...state,
        minSizeMb: action.value,
      };
    case "similarityThreshold/set":
      return {
        ...state,
        similarityThreshold: action.value,
      };
    case "filterExistingOutput/set":
      return {
        ...state,
        filterExistingOutput: Boolean(action.value),
      };
    case "visualReviewEnabled/set":
      return {
        ...state,
        visualReviewEnabled: Boolean(action.value),
      };
    case "exportSimilarImageGroups/set":
      return {
        ...state,
        exportSimilarImageGroups: Boolean(action.value),
      };
    case "locale/set":
      return {
        ...state,
        locale: action.locale,
        isLanguageMenuOpen: false,
      };
    case "languageMenu/toggle":
      return {
        ...state,
        isLanguageMenuOpen: !state.isLanguageMenuOpen,
      };
    case "languageMenu/close":
      return {
        ...state,
        isLanguageMenuOpen: false,
      };
    case "summary/toggle":
      return {
        ...state,
        isSummaryExpanded: !state.isSummaryExpanded,
      };
    default:
      return state;
  }
}

function reduceProgressUpdate(state, payload) {
  const progress = normalizeProgress(payload);

  if (
    state.progress.stage === progress.stage &&
    state.progress.current === progress.current &&
    state.progress.total === progress.total
  ) {
    return state;
  }

  return {
    ...state,
    progress,
  };
}

export function isBusyStatus(runStatus) {
  return runStatus === "running" || runStatus === "cancelling";
}

function normalizeRunStatus(runStatus) {
  const validStatuses = new Set([
    "idle",
    "running",
    "cancelling",
    "review",
    "success",
    "success_with_warnings",
    "error",
  ]);

  return validStatuses.has(runStatus) ? runStatus : "idle";
}

function createEmptySummary() {
  return {
    totalImages: null,
    uniqueImages: null,
    duplicateImages: null,
    report: {
      storageSavedBytes: 0,
      storageSavedHuman: "0 B",
      summary: {
        totalImages: 0,
        uniqueImages: 0,
        duplicateImages: 0,
      },
      folderBreakdown: [],
    },
    outputDir: "",
  };
}

function createEmptyReview() {
  return {
    groups: [],
    selections: {},
  };
}

function normalizeSummary(summary) {
  const value = summary && typeof summary === "object" ? summary : {};

  return {
    totalImages: Number.isFinite(value.totalImages) ? value.totalImages : null,
    uniqueImages: Number.isFinite(value.uniqueImages) ? value.uniqueImages : null,
    duplicateImages: Number.isFinite(value.duplicateImages) ? value.duplicateImages : null,
    report: normalizeReport(value.report),
    outputDir: typeof value.outputDir === "string" ? value.outputDir : "",
  };
}

function normalizeReport(report) {
  const value = report && typeof report === "object" ? report : {};
  const summary = value.summary && typeof value.summary === "object" ? value.summary : {};

  return {
    storageSavedBytes: Number.isFinite(value.storageSavedBytes) ? value.storageSavedBytes : 0,
    storageSavedHuman:
      typeof value.storageSavedHuman === "string" ? value.storageSavedHuman : "0 B",
    summary: {
      totalImages: Number.isFinite(summary.totalImages) ? summary.totalImages : 0,
      uniqueImages: Number.isFinite(summary.uniqueImages) ? summary.uniqueImages : 0,
      duplicateImages: Number.isFinite(summary.duplicateImages) ? summary.duplicateImages : 0,
    },
    folderBreakdown: Array.isArray(value.folderBreakdown)
      ? value.folderBreakdown
          .filter(
            (entry) =>
              entry &&
              typeof entry === "object" &&
              typeof entry.path === "string" &&
              Number.isFinite(entry.duplicates) &&
              Number.isFinite(entry.wastedBytes),
          )
          .map((entry) => ({
            path: entry.path,
            duplicates: entry.duplicates,
            wastedBytes: entry.wastedBytes,
          }))
      : [],
  };
}

function normalizeReview(review) {
  const value = review && typeof review === "object" ? review : {};
  const groups = Array.isArray(value.groups)
    ? value.groups
        .filter(
          (group) =>
            group &&
            typeof group === "object" &&
            typeof group.groupId === "string" &&
            Array.isArray(group.images) &&
            group.images.every((image) => typeof image === "string") &&
            typeof group.suggested === "string",
        )
        .map((group) => ({
          groupId: group.groupId,
          images: [...group.images],
          suggested: group.suggested,
        }))
    : [];

  const selections = {};
  for (const group of groups) {
    const selected = group.images.includes(group.suggested) ? group.suggested : group.images[0];
    if (selected) {
      selections[group.groupId] = selected;
    }
  }

  return {
    groups,
    selections,
  };
}

function selectReviewImage(state, groupId, path) {
  if (typeof groupId !== "string" || typeof path !== "string") {
    return state;
  }

  const group = state.review.groups.find((entry) => entry.groupId === groupId);
  if (!group || !group.images.includes(path) || state.review.selections[groupId] === path) {
    return state;
  }

  return {
    ...state,
    review: {
      ...state.review,
      selections: {
        ...state.review.selections,
        [groupId]: path,
      },
    },
  };
}
