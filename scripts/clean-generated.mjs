import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const defaultRepoRoot = path.resolve(__dirname, "..");

export const GENERATED_PATHS = [
  "coverage",
  "ui/coverage",
  "ui/dist",
  "ui/test-results",
  "src-tauri/target",
];

function resolveTarget(repoRoot, relativePath) {
  const targetPath = path.resolve(repoRoot, relativePath);
  const relativeToRoot = path.relative(repoRoot, targetPath);

  if (
    relativeToRoot === "" ||
    relativeToRoot.startsWith("..") ||
    path.isAbsolute(relativeToRoot)
  ) {
    throw new Error(`Refusing to remove path outside repo root: ${relativePath}`);
  }

  return targetPath;
}

export async function cleanGenerated({
  repoRoot = defaultRepoRoot,
  targets = GENERATED_PATHS,
} = {}) {
  await Promise.all(
    targets.map((target) =>
      fs.rm(resolveTarget(repoRoot, target), { recursive: true, force: true }),
    ),
  );
}

const invokedDirectly = process.argv[1] && path.resolve(process.argv[1]) === __filename;

if (invokedDirectly) {
  cleanGenerated().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
