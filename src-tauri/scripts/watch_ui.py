#!/usr/bin/env python3
import os
import subprocess
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent
UI_SRC = (ROOT_DIR / "../ui").resolve()

EXCLUDES = {
    "node_modules",
    "tests",
    "package.json",
    "package-lock.json",
    "vitest.config.js",
    "dist",
}


def should_skip(path: Path) -> bool:
    for part in path.parts:
        if part in EXCLUDES:
            return True
    return False


def snapshot() -> dict[str, tuple[int, int]]:
    state: dict[str, tuple[int, int]] = {}
    for root, _dirs, files in os.walk(UI_SRC):
        root_path = Path(root)
        if should_skip(root_path):
            _dirs[:] = []
            continue
        for name in files:
            if name in EXCLUDES:
                continue
            file_path = root_path / name
            if should_skip(file_path):
                continue
            try:
                stat = file_path.stat()
            except FileNotFoundError:
                continue
            state[str(file_path)] = (stat.st_mtime_ns, stat.st_size)
    return state


def run_prepare() -> None:
    subprocess.run(
        ["npm", "--prefix", str(UI_SRC), "run", "build"],
        check=True,
        cwd=str(ROOT_DIR),
    )


def main() -> None:
    print("Watching UI changes. Press Ctrl+C to stop.")
    run_prepare()
    last_state = snapshot()

    while True:
        time.sleep(0.8)
        current_state = snapshot()
        if current_state != last_state:
            print("Changes detected. Rebuilding ui/dist...")
            run_prepare()
            last_state = current_state


if __name__ == "__main__":
    main()
