import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { buildUi } from "../scripts/build-ui.mjs";

async function withTempDir(callback) {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "aba-avner-ui-build-"));
  try {
    await callback(tempDir);
  } finally {
    await fs.rm(tempDir, { recursive: true, force: true });
  }
}

async function writeFile(rootDir, relativePath, contents = "") {
  const filePath = path.join(rootDir, relativePath);
  await fs.mkdir(path.dirname(filePath), { recursive: true });
  await fs.writeFile(filePath, contents);
}

async function listFiles(rootDir) {
  const entries = [];

  async function visit(currentDir) {
    const children = await fs.readdir(currentDir, { withFileTypes: true });
    for (const child of children) {
      const childPath = path.join(currentDir, child.name);
      if (child.isDirectory()) {
        await visit(childPath);
      } else {
        entries.push(path.relative(rootDir, childPath));
      }
    }
  }

  await visit(rootDir);
  return entries.sort();
}

describe("buildUi", () => {
  afterEach(() => {
    delete process.env.ABA_AVNER_UI_SRC;
    delete process.env.ABA_AVNER_UI_DIST;
  });

  it("copies only the explicit production assets into dist", async () => {
    await withTempDir(async (tempDir) => {
      const srcDir = path.join(tempDir, "ui");
      const distDir = path.join(srcDir, "dist");

      await writeFile(srcDir, "index.html", "<!doctype html>");
      await writeFile(srcDir, "app.js", "console.log('app');");
      await writeFile(srcDir, "styles.css", "@import './styles/base.css';");
      await writeFile(srcDir, "visual-harness.js", "window.__TEST__ = true;");
      await writeFile(srcDir, "src/app/index.js", "export const ok = true;");
      await writeFile(srcDir, "styles/base.css", "body {}");
      await writeFile(srcDir, "tests/app.test.js", "should not ship");
      await writeFile(srcDir, "coverage/index.html", "should not ship");
      await writeFile(srcDir, "scripts/bootstrap-dev.sh", "should not ship");
      await writeFile(srcDir, "playwright.config.js", "should not ship");
      await writeFile(distDir, "stale.txt", "remove me");

      await buildUi({ rootDir: srcDir, distDir });

      await expect(listFiles(distDir)).resolves.toEqual([
        "app.js",
        "index.html",
        "src/app/index.js",
        "styles.css",
        "styles/base.css",
        "visual-harness.js",
      ]);
    });
  });

  it("fails when a required production asset is missing", async () => {
    await withTempDir(async (tempDir) => {
      const srcDir = path.join(tempDir, "ui");
      const distDir = path.join(srcDir, "dist");

      await writeFile(srcDir, "index.html", "<!doctype html>");
      await writeFile(srcDir, "app.js", "console.log('app');");
      await writeFile(srcDir, "styles.css", "@import './styles/base.css';");
      await writeFile(srcDir, "src/app/index.js", "export const ok = true;");
      await writeFile(srcDir, "styles/base.css", "body {}");

      await expect(buildUi({ rootDir: srcDir, distDir })).rejects.toThrow(
        /Missing required UI build input: .*visual-harness\.js/,
      );
    });
  });
});
