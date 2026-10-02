import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const defaultRootDir = path.resolve(__dirname, "..");

export const BUILD_INPUTS = [
  "index.html",
  "app.js",
  "styles.css",
  "visual-harness.js",
  "src",
  "styles",
];

async function ensureExists(targetPath) {
  try {
    await fs.access(targetPath);
  } catch {
    throw new Error(`Missing required UI build input: ${targetPath}`);
  }
}

async function clearDir(targetDir) {
  await fs.mkdir(targetDir, { recursive: true });
  const entries = await fs.readdir(targetDir, { withFileTypes: true });

  await Promise.all(
    entries.map((entry) =>
      fs.rm(path.join(targetDir, entry.name), { recursive: true, force: true }),
    ),
  );
}

async function copyEntry(sourcePath, targetPath) {
  const stat = await fs.stat(sourcePath);
  if (stat.isDirectory()) {
    await fs.mkdir(targetPath, { recursive: true });
    const entries = await fs.readdir(sourcePath, { withFileTypes: true });
    await Promise.all(
      entries.map((entry) =>
        copyEntry(path.join(sourcePath, entry.name), path.join(targetPath, entry.name)),
      ),
    );
    return;
  }

  await fs.mkdir(path.dirname(targetPath), { recursive: true });
  await fs.copyFile(sourcePath, targetPath);
}

export async function buildUi({
  rootDir = defaultRootDir,
  distDir = path.join(rootDir, "dist"),
} = {}) {
  await Promise.all(BUILD_INPUTS.map((entry) => ensureExists(path.join(rootDir, entry))));
  await clearDir(distDir);

  await Promise.all(
    BUILD_INPUTS.map((entry) => copyEntry(path.join(rootDir, entry), path.join(distDir, entry))),
  );
}

const invokedDirectly = process.argv[1] && path.resolve(process.argv[1]) === __filename;

if (invokedDirectly) {
  buildUi().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
