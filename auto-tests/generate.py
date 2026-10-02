"""Generate the procedural image fixtures in auto-tests/ (no third-party images).

    python3 auto-tests/generate.py && python3 auto-tests/golden/generate.py
"""
from __future__ import annotations

import random
import shutil
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent


def make_image(seed: int, size: tuple[int, int], noise: int) -> Image.Image:
    """Smooth random color field + random shapes + gaussian noise.

    The coarse color field gives each seed a distinct dHash; noise adds
    entropy so large PNGs stay above the 1 MB size threshold the tests use.
    """
    rng = random.Random(seed)
    grid = Image.new("RGB", (5, 4))
    grid.putdata([tuple(rng.randrange(256) for _ in range(3)) for _ in range(20)])
    image = grid.resize(size, Image.Resampling.BICUBIC)

    draw = ImageDraw.Draw(image)
    w, h = size
    for _ in range(rng.randint(4, 9)):
        x0, y0 = rng.randrange(w), rng.randrange(h)
        x1, y1 = x0 + rng.randint(w // 10, w // 3), y0 + rng.randint(h // 10, h // 3)
        color = tuple(rng.randrange(256) for _ in range(3))
        shape = draw.ellipse if rng.random() < 0.5 else draw.rectangle
        shape((x0, y0, x1, y1), fill=color)

    if noise:
        grain = Image.frombytes("L", size, rng.randbytes(w * h)).convert("RGB")
        image = Image.blend(image, grain, noise / 255)
    return image


def reset(path: Path) -> Path:
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True)
    return path


def regular() -> None:
    # Two originals, each with four byte-identical copies (-1..-4).
    out = reset(ROOT / "regular")
    for i, name in enumerate(["large_a", "large_b"]):
        make_image(100 + i, (1000, 800), 60).save(out / f"{name}.png")
        for n in range(1, 5):
            shutil.copy(out / f"{name}.png", out / f"{name}-{n}.png")


def small_images_limiter() -> None:
    # Large (>1 MB) PNGs mixed with small 128x128 ones below the size threshold.
    out = reset(ROOT / "small_images_limiter")
    for i in range(11):
        make_image(200 + i, (128, 128), 40).save(out / f"small_{i:02}.png")
    for i, name in enumerate(["large_a", "large_b", "large_c"]):
        make_image(100 + i, (1000, 800), 60).save(out / f"{name}.png")
    for name in ["large_a", "large_b"]:
        shutil.copy(out / f"{name}.png", out / f"{name}-1.png")


def synthetic_images() -> None:
    # 50 distinct JPEGs plus 3 exact copies, mixed sizes.
    out = reset(ROOT / "synthetic-images")
    sizes = [(640, 480), (800, 600), (480, 640), (1024, 768)]
    for i in range(50):
        make_image(300 + i, sizes[i % len(sizes)], 20).save(out / f"image_{i:02}.jpg", quality=90)
    for i in [4, 5, 6]:
        shutil.copy(out / f"image_{i:02}.jpg", out / f"image_{i:02} (Copy).jpg")


if __name__ == "__main__":
    regular()
    small_images_limiter()
    synthetic_images()
