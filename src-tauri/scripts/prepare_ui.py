#!/usr/bin/env python3
import subprocess
import sys
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent
UI_DIR = (ROOT_DIR / "../ui").resolve()


def main() -> None:
    subprocess.run(
        ["npm", "--prefix", str(UI_DIR), "run", "build"],
        check=True,
        cwd=str(ROOT_DIR),
    )


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        sys.exit(error.returncode)
