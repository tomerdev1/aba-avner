# Aba Avner Core Business Logic

## Purpose

Aba Avner scans an image folder, detects exact and near-duplicate images by visual similarity, and builds a clean output folder that contains only one representative image from each similarity group.

The app is a desktop GUI workflow built around one operational goal:

- reduce a noisy image collection into a deduplicated backup/export folder

## User-Facing Features

The current app exposes these user-facing features that affect behavior:

- choose an input folder and an output folder through desktop folder-pickers
- run the dedupe process from the main screen
- cancel an active run
- set a minimum image size threshold in MB
- set a similarity threshold for near-duplicate matching
- choose whether the output folder should be treated as an already-deduplicated destination
- choose whether visual duplicate review runs before export
- choose whether the duplicate-group reference folder is exported
- switch UI language between English and Hebrew

The minimum image size, similarity threshold, "skip images already in output", and visual-review setting change runtime behavior. The similar-group export setting changes output side effects but does not change grouping or representative selection. Language and view settings change the UI only.

## End-to-End Flow

1. The user starts a run from the UI.
2. The app prompts for an input folder, then an output folder.
3. The backend trims and normalizes the submitted paths.
4. The backend validates that the input folder exists and is a directory.
5. The backend validates that the output folder is safe to use.
6. The backend rejects unsafe path relationships:
   - input and output cannot resolve to the same directory
   - output cannot be inside input
   - input cannot be inside output
   - output cannot be the filesystem root
7. The app recursively scans the input folder for supported image files only.
8. Files smaller than the configured minimum size are excluded before hashing.
9. Remaining images are loaded and converted into perceptual hashes (`dHash`).
10. The backend compares hashes by Hamming distance to find visually similar images.
11. Similar images are merged into groups.
12. The backend returns the grouped images to the UI as review groups with a stable `group_id`, all group image paths, and a suggested keep-path.
13. If the visual review setting is enabled, the UI enters a review step before export.
14. During review, the user keeps exactly one image per group:
   - each group starts with the backend suggestion selected
   - clicking a different image in that group replaces the selection
   - if the user does not change a group, the suggested path is used
15. If the visual review setting is disabled, the app skips the review screen and immediately exports using the backend suggestion for each group.
16. When export starts, the backend validates every submitted selection:
   - the selected path must belong to that group
   - missing group selections fall back to the suggested path
   - invalid paths fail the export with a clear error
17. If the "skip images already in output" setting is enabled, the app also hashes images already present in the output folder and removes any selected representative that already has a similar match there.
18. The remaining representative images are copied into the output folder with collision-safe filenames.
19. If the "export similar image groups folder" setting is enabled, the app also creates or refreshes a sidecar folder that contains duplicate-group reference copies.
20. If that setting is disabled, the app does not create, refresh, or remove that sidecar folder during the run.
21. The app returns summary counts, warning details, and the resolved output folder path.

## Similarity Rule

This app does not use byte-for-byte comparison. It uses perceptual similarity.

- every kept candidate image is converted into a 64-bit `dHash`
- two images are treated as similar when the Hamming distance between hashes is less than or equal to the configured threshold
- the frontend persists this threshold in settings and currently defaults it to `10`

This allows the app to group:

- exact duplicates
- renamed duplicates
- lightly edited copies
- visually close near-duplicates

## Grouping Rule

Similarity grouping is transitive.

If image A is similar to B, and B is similar to C, they can end up in the same duplicate group even if A and C are not directly the closest pair.

The backend uses:

- a BK-tree for efficient similarity candidate lookup
- union-find style clustering to merge related matches into groups

## Representative Selection Rule

For each similarity group, the app defines exactly one suggested representative:

- the first entry in that group after grouping

Before export, the user may override that suggestion by selecting another image from the same group.

If the visual review step is disabled, the suggested representative is used automatically for every group.

At export time:

- exactly one path is resolved per group
- missing selections use the suggested representative
- paths outside the group are rejected

All non-selected images in the group are counted as duplicates and are not copied.

## Existing Output Filtering

This behavior is configurable and is enabled by default in the current UI.

When enabled:

- the output folder is scanned for supported image files
- those output images are hashed for similarity comparison without applying the user-selected minimum size threshold
- each selected representative is compared against the existing output set
- if a similar output image already exists, that representative is skipped and a warning is recorded

When disabled:

- representatives are copied without comparing them to images already present in the output folder

If scanning the output folder fails during this optional filtering step, the run continues and records a warning instead of failing immediately.

## File Handling Rules

- input scanning is recursive
- only supported image extensions are considered: `jpg`, `jpeg`, `png`, `gif`, `bmp`, `tiff`, `tif`, `webp`
- non-image files are ignored
- symlinks are not followed during directory walking
- the output directory is created if it does not already exist
- the optional `similar image groups` sidecar folder is only managed when that export setting is enabled
- if a copied filename already exists, the app appends `-1`, `-2`, and so on to avoid overwriting files

## Filtering And Warning Rules

- files below the minimum image size threshold are skipped quietly and do not count toward run totals
- files that cannot be stat'ed, loaded, or hashed produce warnings
- files skipped because a similar image already exists in the output folder produce warnings
- warning details are returned with structured codes so the UI can render targeted messages

Current warning codes are:

- `file_issue`
- `output_dir_scan_failed`
- `similar_image_in_output`

## Result Semantics

At the end of a successful run, the app reports:

- `total_images`: images that successfully entered dedupe comparison after size filtering and hashing prerequisites
- `unique_images`: representative images actually copied into the output folder
- `duplicate_images`: `total_images - unique_images`
- `report.storage_saved_bytes`: total bytes consumed by removed duplicate members inside grouped duplicate sets, excluding kept representatives
- `report.storage_saved_human`: human-readable binary unit rendering of `report.storage_saved_bytes`
- `report.summary`: grouping-based counts computed immediately after grouping
- `report.folder_breakdown`: duplicate-member counts and wasted bytes per source folder, sorted by highest wasted space
- `output_dir`: the normalized output directory path used for the run
- `warnings` / `warning_details`: non-fatal issues and skip reasons

This means:

- images skipped for being too small are excluded from `total_images`
- unreadable or unhashable files are excluded from `total_images` and surfaced as warnings
- representatives skipped because a similar image already exists in the output folder reduce `unique_images` and therefore increase `duplicate_images`
- the report is built from already-collected grouped entries and does not trigger an extra filesystem scan after grouping
- `report.summary.unique_images` is the number of grouping representatives before any output-folder filtering, so it can differ from top-level `unique_images`

## Progress And Cancellation

The app publishes progress events during a run and the UI can cancel an active run.

Current progress stages are:

- `scan`
- `hash`
- `group`
- `review`
- `copy`

Backend progress totals are stage-local:

- `scan` starts at `0/1` and completes at `1/1` for the discovered image list
- `hash` advances from `0/N` to `N/N`, where `N` is the scanned image count
- `group` advances from `0/N` to `N/N`, where `N` is the hashed image count
- `review` is a UI-held stage between grouping and export only when the visual review setting is enabled
- `copy` advances from `0/U` to `U/U`, where `U` is the number of unique representatives selected for export

The UI renders those stage-local updates as one monotonic pipeline:

- known runtime stages are ordered as `scan -> hash -> group -> review -> copy`
- the visible progress bar does not move backward when the backend switches to the next stage
- successful runs render the progress bar as complete even if the last raw stage payload came from an empty or fully filtered copy stage

Cancellation is cooperative:

- the backend checks for cancellation before and between major stages
- input directory scanning also checks cancellation during filesystem traversal so large scans can stop before hashing begins
- the grouping loop also checks cancellation between per-image grouping iterations so large grouping stages stop promptly
- the existing-output filter also checks cancellation between per-image output hashing iterations so large output scans stop promptly before copy work starts
- filesystem export loops also check cancellation between file copies so large exports stop promptly
- a cancelled run stops without completing copy work
- the UI treats cancellation as a non-successful, non-crashing stop rather than a completed export
