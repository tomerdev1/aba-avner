import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, it } from "vitest";
import { cleanGenerated, GENERATED_PATHS } from "../../scripts/clean-generated.mjs";

async function withTempDir(callback) {
  const tempDir = await fs.mkdtemp(path.join(os.tmpdir(), "aba-avner-clean-"));
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

async function pathExists(targetPath) {
  try {
    await fs.access(targetPath);
    return true;
  } catch {
    return false;
  }
}

describe("cleanGenerated", () => {
  it("removes only the generated directories from the known cleanup set", async () => {
    await withTempDir(async (repoRoot) => {
      for (const generatedPath of GENERATED_PATHS) {
        await writeFile(repoRoot, path.join(generatedPath, "artifact.txt"), generatedPath);
      }

      await writeFile(repoRoot, "README.md", "keep");
      await writeFile(repoRoot, "ui/src/app/index.js", "keep");
      await writeFile(repoRoot, "src-tauri/src/main.rs", "keep");

      await cleanGenerated({ repoRoot });

      for (const generatedPath of GENERATED_PATHS) {
        await expect(pathExists(path.join(repoRoot, generatedPath))).resolves.toBe(false);
      }

      await expect(fs.readFile(path.join(repoRoot, "README.md"), "utf8")).resolves.toBe("keep");
      await expect(fs.readFile(path.join(repoRoot, "ui/src/app/index.js"), "utf8")).resolves.toBe("keep");
      await expect(fs.readFile(path.join(repoRoot, "src-tauri/src/main.rs"), "utf8")).resolves.toBe("keep");
    });
  });

  it("tolerates missing generated directories", async () => {
    await withTempDir(async (repoRoot) => {
      await writeFile(repoRoot, "README.md", "keep");

      await expect(cleanGenerated({ repoRoot })).resolves.toBeUndefined();
      await expect(fs.readFile(path.join(repoRoot, "README.md"), "utf8")).resolves.toBe("keep");
    });
  });
});
