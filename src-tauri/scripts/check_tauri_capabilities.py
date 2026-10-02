#!/usr/bin/env python3
import json
from pathlib import Path
import sys


def main() -> int:
    repo_root = Path(__file__).resolve().parents[2]
    capability_path = repo_root / "src-tauri" / "capabilities" / "default.json"
    expected_permissions = ["core:default", "dialog:default", "updater:default"]

    payload = json.loads(capability_path.read_text())
    permissions = payload.get("permissions")

    if permissions != expected_permissions:
        print(
            "Capability review failed: expected permissions "
            f"{expected_permissions}, found {permissions}",
            file=sys.stderr,
        )
        return 1

    print(f"Capability review passed for {capability_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
