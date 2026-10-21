import { describe, expect, it } from "vitest";

import { enMessages } from "../src/localization/messages/en.js";
import { heMessages } from "../src/localization/messages/he.js";
import { esMessages } from "../src/localization/messages/es.js";
import { zhMessages } from "../src/localization/messages/zh.js";
import { filMessages } from "../src/localization/messages/fil.js";
import { swMessages } from "../src/localization/messages/sw.js";
import { tpiMessages } from "../src/localization/messages/tpi.js";

const translations = { he: heMessages, es: esMessages, zh: zhMessages, fil: filMessages, sw: swMessages, tpi: tpiMessages };

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
  it.each(Object.entries(translations))("keep english and %s catalogs structurally aligned", (_code, messages) => {
    expect(flattenMessageKeys(messages).sort()).toEqual(flattenMessageKeys(enMessages).sort());
  });

  it("keep all localized values as strings", () => {
    const allValues = [enMessages, ...Object.values(translations)].flatMap(flattenMessageValues);

    expect(allValues.every((value) => typeof value === "string")).toBe(true);
  });
});
