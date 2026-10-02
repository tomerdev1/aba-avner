#!/usr/bin/env python3
from pathlib import Path
import sys


EXPECTED_BUNDLES = {
    "linux": [
        "src-tauri/target/release/bundle/appimage/*.AppImage",
        "src-tauri/target/release/bundle/deb/*.deb",
        "src-tauri/target/release/bundle/rpm/*.rpm",
    ],
    "macos": [
        "src-tauri/target/release/bundle/dmg/*.dmg",
        "src-tauri/target/release/bundle/macos/*.app",
    ],
    "windows": [
        "src-tauri/target/release/*.exe",
        "src-tauri/target/release/bundle/nsis/*.exe",
    ],
}


def main() -> int:
    if len(sys.argv) != 2 or sys.argv[1] not in EXPECTED_BUNDLES:
        valid_platforms = ", ".join(sorted(EXPECTED_BUNDLES))
        print(
            f"Usage: {Path(__file__).name} <platform> "
            f"where <platform> is one of: {valid_platforms}",
            file=sys.stderr,
        )
        return 1

    repo_root = Path(__file__).resolve().parents[2]
    platform = sys.argv[1]
    missing_patterns: list[str] = []
    found_paths: list[Path] = []

    for pattern in EXPECTED_BUNDLES[platform]:
        matches = sorted(repo_root.glob(pattern))
        if matches:
            found_paths.extend(matches)
            continue
        missing_patterns.append(pattern)

    if missing_patterns:
        print(
            f"Bundle verification failed for {platform}: missing outputs for "
            f"{', '.join(missing_patterns)}",
            file=sys.stderr,
        )
        return 1

    print(f"Bundle verification passed for {platform}:")
    for path in found_paths:
        print(path.relative_to(repo_root))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
