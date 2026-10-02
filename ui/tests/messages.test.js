import { describe, expect, it } from "vitest";

import { buildFailureIssues, buildWarningsIssues } from "../src/app/messages.js";

function t(key, params = {}) {
  return Object.entries(params).reduce(
    (message, [paramKey, value]) => message.replace(`{${paramKey}}`, String(value)),
    key,
  );
}

describe("warning issues", () => {
  it("localizes known structured warnings while keeping count and order", () => {
    const issues = buildWarningsIssues(t, {
      warnings: [
        "/input/a.png: permission denied",
        "Skipped /input/b.png because a similar image already exists in the output folder.",
      ],
      warningDetails: [
        {
          code: "file_issue",
          path: "/input/a.png",
          detail: "permission denied",
        },
        {
          code: "similar_image_in_output",
          path: "/input/b.png",
        },
      ],
    });

    expect(issues.title).toBe("issues.warningsTitle");
    expect(issues.items).toEqual([
      "issues.warningItems.fileIssue",
      "issues.warningItems.similarImageInOutput",
    ]);
  });

  it("falls back to raw warnings for unknown structured warning codes", () => {
    const issues = buildWarningsIssues(t, {
      warnings: ["original backend warning"],
      warningDetails: [
        {
          code: "unknown_warning",
          path: "/input/a.png",
          detail: "permission denied",
        },
      ],
    });

    expect(issues.items).toEqual(["/input/a.png: permission denied"]);
  });

  it("falls back to raw warnings when structured warning details are absent", () => {
    const issues = buildWarningsIssues(t, {
      warnings: ["warning 1", "warning 2"],
    });

    expect(issues.items).toEqual(["warning 1", "warning 2"]);
  });

  it("preserves extra raw warnings that do not have structured details", () => {
    const issues = buildWarningsIssues(t, {
      warnings: ["warning 1", "unstructured leftover"],
      warningDetails: [
        {
          code: "file_issue",
          path: "/input/a.png",
          detail: "permission denied",
        },
      ],
    });

    expect(issues.items).toEqual([
      "issues.warningItems.fileIssue",
      "unstructured leftover",
    ]);
  });

  it("renders a dedicated cancelled issue state", () => {
    const issues = buildFailureIssues(t, {
      code: "cancelled",
      message: "Dedupe run cancelled",
    });

    expect(issues).toEqual({
      title: "issues.runCancelledTitle",
      hint: "issues.runCancelledHint",
      items: ["issues.errors.cancelled"],
    });
  });

  it("maps known backend validation codes to localized messages", () => {
    const issues = buildFailureIssues(t, {
      code: "input_output_conflict",
      message: "Input and output directories conflict: /input is inside /output",
    });

    expect(issues).toEqual({
      title: "issues.runFailedTitle",
      hint: "issues.runFailedHint",
      items: ["issues.errors.inputOutputConflict"],
    });
  });

  it("maps run-in-progress errors to localized messages", () => {
    const issues = buildFailureIssues(t, {
      code: "run_in_progress",
      message: "A dedupe run is already in progress",
    });

    expect(issues).toEqual({
      title: "issues.runFailedTitle",
      hint: "issues.runFailedHint",
      items: ["issues.errors.runInProgress"],
    });
  });
});
