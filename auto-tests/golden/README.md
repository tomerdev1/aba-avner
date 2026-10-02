Generated golden fixtures derived from `../synthetic-images` (procedural, see `../generate.py`).

Regenerate with:

```bash
python3 auto-tests/golden/generate.py
```

The generator creates:
- `input/`: exact duplicates, near duplicates, distinct images, and one corrupt file
- `output_seed/`: a pre-existing output image used to test existing-output skipping
- `manifest.json`: expected grouping and count invariants for tests
