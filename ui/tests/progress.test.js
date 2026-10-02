import { describe, expect, it } from "vitest";

import { calculateProgressPercent, setProgress } from "../src/app/progress.js";

describe("progress rendering helpers", () => {
  it("skips redundant progress DOM writes for identical values", () => {
    const progressBar = createAttributeElement();

    const elements = {
      progressBar,
      progressStage: createTextElement(),
      progressPercent: createTextElement(),
    };

    setProgress("Hashing", 1, 4, elements);

    const writesAfterFirstRender = {
      width: progressBar.widthWrites,
      stage: elements.progressStage.textWrites,
      percent: elements.progressPercent.textWrites,
      valueNow: progressBar.attributeWrites["aria-valuenow"],
      valueText: progressBar.attributeWrites["aria-valuetext"],
    };

    setProgress("Hashing", 1, 4, elements);

    expect(progressBar.widthWrites).toBe(writesAfterFirstRender.width);
    expect(elements.progressStage.textWrites).toBe(writesAfterFirstRender.stage);
    expect(elements.progressPercent.textWrites).toBe(writesAfterFirstRender.percent);
    expect(progressBar.attributeWrites["aria-valuenow"]).toBe(writesAfterFirstRender.valueNow);
    expect(progressBar.attributeWrites["aria-valuetext"]).toBe(writesAfterFirstRender.valueText);
  });

  it("maps known runtime stages onto monotonic overall pipeline progress", () => {
    expect(calculateProgressPercent("scan", 1, 2)).toBe(10);
    expect(calculateProgressPercent("hash", 1, 2)).toBe(30);
    expect(calculateProgressPercent("group", 1, 2)).toBe(50);
    expect(calculateProgressPercent("copy", 1, 2)).toBe(90);
  });

  it("forces completed runs to render at 100 percent", () => {
    const progressBar = createAttributeElement();
    const elements = {
      progressBar,
      progressStage: createTextElement(),
      progressPercent: createTextElement(),
    };

    setProgress("Copying", 0, 1, elements, { completed: true });

    expect(elements.progressPercent.textContent).toBe("100%");
    expect(progressBar.style.width).toBe("100%");
    expect(progressBar.getAttribute("aria-valuenow")).toBe("100");
  });
});

function createAttributeElement() {
  const attributes = new Map();
  let width = "";
  const element = {
    attributeWrites: {},
    widthWrites: 0,
    getAttribute(name) {
      return attributes.has(name) ? attributes.get(name) : null;
    },
    setAttribute(name, value) {
      this.attributeWrites[name] = (this.attributeWrites[name] || 0) + 1;
      attributes.set(name, value);
    },
    style: {
      get width() {
        return width;
      },
      set width(value) {
        element.widthWrites += 1;
        width = value;
      },
    },
  };

  return element;
}

function createTextElement() {
  let textContent = "";

  return {
    textWrites: 0,
    get textContent() {
      return textContent;
    },
    set textContent(value) {
      this.textWrites += 1;
      textContent = value;
    },
  };
}
