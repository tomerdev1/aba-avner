const rawState = new URLSearchParams(window.location.search).get("visual-test-state");
const isRtl = rawState?.endsWith("-rtl") ?? false;
const state = isRtl ? rawState.slice(0, -"-rtl".length) : rawState;

const RUN_DEDUPE_COMMAND = "run_dedupe";
const PREPARE_DEDUPE_REVIEW_COMMAND = "prepare_dedupe_review";
const EXPORT_DEDUPE_REVIEW_COMMAND = "export_dedupe_review";
const CANCEL_DEDUPE_COMMAND = "cancel_dedupe";

function createSwatchDataUri(label, fill) {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 160 160"><rect width="160" height="160" rx="18" fill="${fill}"/><text x="50%" y="54%" dominant-baseline="middle" text-anchor="middle" font-family="Arial" font-size="28" fill="#ffffff">${label}</text></svg>`;
  return `data:image/svg+xml;charset=UTF-8,${encodeURIComponent(svg)}`;
}

const reviewPreviewMap = new Map([
  ["/mock/review/group-1/photo-a.jpg", createSwatchDataUri("A", "#3f6f61")],
  ["/mock/review/group-1/photo-b.jpg", createSwatchDataUri("B", "#7b4a3d")],
  ["/mock/review/group-1/photo-c.jpg", createSwatchDataUri("C", "#4e5f9a")],
  ["/mock/review/group-2/photo-d.jpg", createSwatchDataUri("D", "#8c6b2a")],
  ["/mock/review/group-2/photo-e.jpg", createSwatchDataUri("E", "#5b4e8a")],
]);

function createReport({
  totalImages,
  uniqueImages,
  duplicateImages,
  groupedUniqueImages = uniqueImages,
  groupedDuplicateImages = duplicateImages,
  storageSavedBytes,
  storageSavedHuman,
  folderBreakdown,
}) {
  return {
    storageSavedBytes,
    storageSavedHuman,
    summary: {
      totalImages,
      uniqueImages: groupedUniqueImages,
      duplicateImages: groupedDuplicateImages,
    },
    folderBreakdown,
  };
}

function createWarnings(count) {
  return Array.from(
    { length: count },
    (_, index) => `Skipped /input/batch-${String(index + 1).padStart(2, "0")}.jpg: unreadable metadata`,
  );
}

function createWarningResponse() {
  return {
    totalImages: 86,
    uniqueImages: 51,
    duplicateImages: 35,
    report: createReport({
      totalImages: 86,
      uniqueImages: 51,
      duplicateImages: 35,
      groupedUniqueImages: 54,
      groupedDuplicateImages: 32,
      storageSavedBytes: 1932735283,
      storageSavedHuman: "1.8 GB",
      folderBreakdown: [
        {
          path: "/input/wedding/raw-dumps/day-01",
          duplicates: 14,
          wastedBytes: 882900992,
        },
        {
          path: "/input/phone-backup/2024/exports",
          duplicates: 11,
          wastedBytes: 540016640,
        },
        {
          path: "/input/social-media/reels-stills",
          duplicates: 7,
          wastedBytes: 334495744,
        },
      ],
    }),
    outputDir:
      "/output/2026/visual-regression/session-alpha/review/exports/final-curation/duplicates-kept",
    warnings: createWarnings(14),
  };
}

function createCompleteResponse() {
  return {
    totalImages: 18,
    uniqueImages: 11,
    duplicateImages: 7,
    report: createReport({
      totalImages: 18,
      uniqueImages: 11,
      duplicateImages: 7,
      groupedUniqueImages: 13,
      groupedDuplicateImages: 5,
      storageSavedBytes: 734003200,
      storageSavedHuman: "700 MB",
      folderBreakdown: [
        {
          path: "/input/wedding/raw-dumps/day-01",
          duplicates: 4,
          wastedBytes: 419430400,
        },
        {
          path: "/input/phone-backup/2024/exports",
          duplicates: 3,
          wastedBytes: 314572800,
        },
      ],
    }),
    outputDir: "/output/2026/visual-regression/session-alpha/review/exports/final-curation",
    warnings: [],
  };
}

function createReviewResponse() {
  return {
    groups: [
      {
        groupId: "g1",
        images: [
          "/mock/review/group-1/photo-a.jpg",
          "/mock/review/group-1/photo-b.jpg",
          "/mock/review/group-1/photo-c.jpg",
        ],
        suggested: "/mock/review/group-1/photo-a.jpg",
      },
      {
        groupId: "g2",
        images: ["/mock/review/group-2/photo-d.jpg", "/mock/review/group-2/photo-e.jpg"],
        suggested: "/mock/review/group-2/photo-e.jpg",
      },
    ],
    warnings: [],
  };
}

function neverResolvingPromise() {
  return new Promise(() => {});
}

function setSliderValue(value) {
  const slider = document.getElementById("minSizeSlider");
  slider.value = value;
  slider.dispatchEvent(new Event("input", { bubbles: true }));
}

function setThresholdValue(value) {
  const thresholdInput = document.getElementById("similarityThresholdInput");
  thresholdInput.value = value;
  thresholdInput.dispatchEvent(new Event("input", { bubbles: true }));
}

function createScenario(currentState) {
  switch (currentState) {
    case "canceling":
      return {
        invoke(command) {
          if (command === CANCEL_DEDUPE_COMMAND) {
            return true;
          }

          if (command === PREPARE_DEDUPE_REVIEW_COMMAND || command === RUN_DEDUPE_COMMAND) {
            return neverResolvingPromise();
          }

          throw new Error(`Unexpected command in ${currentState}: ${command}`);
        },
        async onLoad() {
          setSliderValue("2.5");
          window.__ABA_AVNER_TEST_API__.runDedupe();
          await waitFor(() => document.getElementById("startButton").textContent.includes("Cancel"));
          document.getElementById("startButton").click();
        },
      };
    case "progress":
      return {
        invoke(command) {
          if (command === PREPARE_DEDUPE_REVIEW_COMMAND || command === RUN_DEDUPE_COMMAND) {
            return neverResolvingPromise();
          }

          throw new Error(`Unexpected command in ${currentState}: ${command}`);
        },
        async onLoad(context) {
          setSliderValue("2.5");
          window.__ABA_AVNER_TEST_API__.runDedupe();
          await waitFor(() => document.getElementById("progressStage").textContent.includes("Starting"));
          context.emitProgress("scan", 9, 18);
          await waitFor(() => document.getElementById("progressStage").textContent.includes("Scanning"));
        },
      };
    case "review":
      return {
        invoke(command) {
          if (command === PREPARE_DEDUPE_REVIEW_COMMAND) {
            return createReviewResponse();
          }

          throw new Error(`Unexpected command in ${currentState}: ${command}`);
        },
        async onLoad() {
          setSliderValue("2.5");
          await window.__ABA_AVNER_TEST_API__.runDedupe();
        },
      };
    case "complete":
      return {
        invoke(command) {
          if (command === PREPARE_DEDUPE_REVIEW_COMMAND) {
            return createReviewResponse();
          }

          if (command === EXPORT_DEDUPE_REVIEW_COMMAND) {
            return createCompleteResponse();
          }

          throw new Error(`Unexpected command in ${currentState}: ${command}`);
        },
        async onLoad() {
          window.__ABA_AVNER_TEST_API__.setVisualReviewEnabled(false);
          setSliderValue("2.5");
          await window.__ABA_AVNER_TEST_API__.runDedupe();
          document.getElementById("summaryToggleButton")?.click();
        },
      };
    case "warning-flood":
      return {
        invoke(command) {
          if (command === PREPARE_DEDUPE_REVIEW_COMMAND) {
            return createReviewResponse();
          }

          if (command === EXPORT_DEDUPE_REVIEW_COMMAND) {
            return createWarningResponse();
          }

          throw new Error(`Unexpected command in ${currentState}: ${command}`);
        },
        async onLoad() {
          window.__ABA_AVNER_TEST_API__.setVisualReviewEnabled(false);
          setSliderValue("3.4");
          await window.__ABA_AVNER_TEST_API__.runDedupe();
          document.getElementById("summaryToggleButton")?.click();
        },
      };
    case "failure-flood":
      return {
        invoke(command) {
          if (command === PREPARE_DEDUPE_REVIEW_COMMAND || command === RUN_DEDUPE_COMMAND) {
            throw new Error(
              "Output folder is not writable. Could not create /output/session-2026-03-14/report.json",
            );
          }

          throw new Error(`Unexpected command in ${currentState}: ${command}`);
        },
        async onLoad() {
          setSliderValue("4.1");
          await window.__ABA_AVNER_TEST_API__.runDedupe();
        },
      };
    case "settings":
      return {
        invoke(command) {
          if (command === PREPARE_DEDUPE_REVIEW_COMMAND) {
            return createReviewResponse();
          }

          if (command === EXPORT_DEDUPE_REVIEW_COMMAND) {
            return createCompleteResponse();
          }

          if (command === RUN_DEDUPE_COMMAND) {
            return createCompleteResponse();
          }

          throw new Error(`Unexpected command in ${currentState}: ${command}`);
        },
        async onLoad() {
          window.__ABA_AVNER_TEST_API__.openSettings();
          setSliderValue("3.3");
          setThresholdValue("6");
        },
      };
    case "idle":
      return {
        invoke(command) {
          if (command === PREPARE_DEDUPE_REVIEW_COMMAND) {
            return createReviewResponse();
          }

          if (command === EXPORT_DEDUPE_REVIEW_COMMAND) {
            return createCompleteResponse();
          }

          if (command === RUN_DEDUPE_COMMAND) {
            return createCompleteResponse();
          }

          throw new Error(`Unexpected command in ${currentState}: ${command}`);
        },
        async onLoad() {
          setSliderValue("1.2");
        },
      };
    default:
      return null;
  }
}

if (state) {
  const scenario = createScenario(state);
  let activeRunId = null;

  window.__ABA_AVNER_TESTING__ = true;
  window.__TAURI__ = {
    dialog: {
      open: async ({ title }) => (title.includes("source") ? "/input" : "/output"),
    },
    core: {
      convertFileSrc: (path) => reviewPreviewMap.get(path) ?? path,
      invoke: async (command, args) => {
        activeRunId = args?.config?.runId ?? activeRunId;

        if (!scenario) {
          throw new Error(`Unknown visual state: ${state}`);
        }

        return scenario.invoke(command, args);
      },
    },
    event: {
      listen: async (_eventName, callback) => {
        window.__ABA_AVNER_VISUAL_PROGRESS__ = callback;
        return () => {};
      },
    },
  };

  window.addEventListener("load", async () => {
    const issuesList = document.getElementById("issuesList");

    if (!scenario) {
      issuesList.textContent = `Unknown visual state: ${state}`;
      return;
    }

    if (state === "error") {
      window.__TAURI__ = undefined;
      document.getElementById("startButton").click();
      return;
    }

    if (isRtl) {
      window.__ABA_AVNER_TEST_API__.setLocale("he");
    }

    await scenario.onLoad({
      emitProgress(stage, current, total) {
        window.__ABA_AVNER_VISUAL_PROGRESS__?.({
          payload: { runId: activeRunId, stage, current, total },
        });
      },
    });
  });
}

function waitFor(callback, timeout = 2000) {
  const started = Date.now();

  return new Promise((resolve, reject) => {
    function tick() {
      try {
        if (callback()) {
          resolve();
          return;
        }
      } catch (error) {
        reject(error);
        return;
      }

      if (Date.now() - started >= timeout) {
        reject(new Error("Timed out waiting for visual harness condition"));
        return;
      }

      requestAnimationFrame(tick);
    }

    tick();
  });
}
