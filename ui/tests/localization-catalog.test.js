import { describe, expect, it } from "vitest";

import { enMessages } from "../src/localization/messages/en.js";
import { heMessages } from "../src/localization/messages/he.js";

function flattenMessageKeys(messages, prefix = "") {
  return Object.entries(messages).flatMap(([key, value]) => {
    const nextKey = prefix ? `${prefix}.${key}` : key;

    if (value && typeof value === "object" && !Array.isArray(value)) {
      return flattenMessageKeys(value, nextKey);
    }

    return [nextKey];
  });
}

function flattenMessageValues(messages) {
  return Object.values(messages).flatMap((value) => {
    if (value && typeof value === "object" && !Array.isArray(value)) {
      return flattenMessageValues(value);
    }

    return [value];
  });
}

describe("localization catalogs", () => {
  it("keep english and hebrew catalogs structurally aligned", () => {
    expect(flattenMessageKeys(heMessages).sort()).toEqual(flattenMessageKeys(enMessages).sort());
  });

  it("keep all localized values as strings", () => {
    const allValues = [...flattenMessageValues(enMessages), ...flattenMessageValues(heMessages)];

    expect(allValues.every((value) => typeof value === "string")).toBe(true);
  });
});
