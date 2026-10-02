from __future__ import annotations

import json
import shutil
from pathlib import Path

from PIL import Image, ImageEnhance, ImageFilter


ROOT = Path(__file__).resolve().parent
SOURCE_DIR = ROOT.parent / "synthetic-images"
INPUT_DIR = ROOT / "input"
OUTPUT_SEED_DIR = ROOT / "output_seed"
MANIFEST_PATH = ROOT / "manifest.json"
THRESHOLD = 10
SOURCE_FILES = [
    "image_02.jpg",
    "image_03.jpg",
    "image_04.jpg",
    "image_05.jpg",
    "image_06.jpg",
]


def dhash(image: Image.Image) -> int:
    resized = image.convert("L").resize((9, 8), Image.Resampling.BILINEAR)
    value = 0
    for y in range(8):
        for x in range(8):
            left = resized.getpixel((x, y))
            right = resized.getpixel((x + 1, y))
            if left > right:
                value |= 1 << (y * 8 + x)
    return value


def hamming_distance(a: int, b: int) -> int:
    return (a ^ b).bit_count()


def source_paths() -> list[Path]:
    paths = [SOURCE_DIR / name for name in SOURCE_FILES]
    missing = [path.name for path in paths if not path.exists()]
    if missing:
        raise RuntimeError(f"missing source files: {', '.join(missing)}")
    return paths


def candidate_variants(image: Image.Image) -> list[Image.Image]:
    width, height = image.size
    crop_x = max(2, width // 40)
    crop_y = max(2, height // 40)
    variants = []

    variants.append(
        image.crop((crop_x, crop_y, width - crop_x, height - crop_y)).resize(
            (width, height), Image.Resampling.LANCZOS
        )
    )
    variants.append(
        ImageEnhance.Brightness(image).enhance(1.06).filter(ImageFilter.GaussianBlur(radius=0.4))
    )
    variants.append(
        ImageEnhance.Contrast(image).enhance(1.08).resize(
            (max(32, width - 6), max(32, height - 6)), Image.Resampling.LANCZOS
        ).resize((width, height), Image.Resampling.LANCZOS)
    )
    variants.append(
        image.rotate(1.2, resample=Image.Resampling.BICUBIC, expand=False).filter(
            ImageFilter.UnsharpMask(radius=1, percent=70)
        )
    )
    return variants


def build_near_variant(source_path: Path) -> Image.Image:
    with Image.open(source_path) as image:
        base = image.convert("RGB")
        base_hash = dhash(base)
        for variant in candidate_variants(base):
            distance = hamming_distance(base_hash, dhash(variant))
            if 1 <= distance <= THRESHOLD:
                return variant
    raise RuntimeError(f"unable to build near variant within threshold for {source_path.name}")


def reset_output() -> None:
    for path in (INPUT_DIR, OUTPUT_SEED_DIR):
        if path.exists():
            shutil.rmtree(path)
        path.mkdir(parents=True, exist_ok=True)


def copy_file(source: Path, destination: Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)


def write_corrupt_file(source: Path, destination: Path) -> None:
    raw = source.read_bytes()
    destination.write_bytes(raw[: max(64, len(raw) // 20)])


def main() -> None:
    if not SOURCE_DIR.exists():
        raise SystemExit(f"missing source directory: {SOURCE_DIR}")

    reset_output()
    anchor_a, anchor_b, anchor_c, anchor_d, anchor_e = source_paths()

    copy_file(anchor_a, INPUT_DIR / "group_a_original.jpg")
    copy_file(anchor_a, INPUT_DIR / "group_a_exact.jpg")
    build_near_variant(anchor_a).save(INPUT_DIR / "group_a_near.jpg", quality=92)

    copy_file(anchor_b, INPUT_DIR / "group_b_original.jpg")
    build_near_variant(anchor_b).save(INPUT_DIR / "group_b_near.jpg", quality=92)

    copy_file(anchor_c, INPUT_DIR / "distinct_c.jpg")
    copy_file(anchor_d, INPUT_DIR / "distinct_d.jpg")

    copy_file(anchor_e, INPUT_DIR / "existing_input.jpg")
    copy_file(anchor_e, OUTPUT_SEED_DIR / "existing_seed.jpg")

    write_corrupt_file(anchor_b, INPUT_DIR / "corrupt_input.jpg")

    manifest = {
        "similarity_threshold": THRESHOLD,
        "expected_hashed_images": 8,
        "expected_unique_after_existing_filter": 4,
        "expected_duplicate_images": 4,
        "expected_groups": [
            ["group_a_exact.jpg", "group_a_near.jpg", "group_a_original.jpg"],
            ["group_b_near.jpg", "group_b_original.jpg"],
            ["distinct_c.jpg"],
            ["distinct_d.jpg"],
            ["existing_input.jpg"],
        ],
        "expected_output_prefixes": ["group_a_", "group_b_", "distinct_c", "distinct_d"],
        "expected_warning_files": ["corrupt_input.jpg", "existing_input.jpg"],
    }
    MANIFEST_PATH.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
