use crate::application::{AppLocale, CancellationToken, NoopCancellationToken};
use crate::domain::ImageEntry;
use crate::errors::{DedupeError, Result};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn ensure_output_dir(path: &Path) -> Result<()> {
    if path.exists() && !path.is_dir() {
        return Err(DedupeError::InvalidOutputDir(path.display().to_string()));
    }
    fs::create_dir_all(path)
        .map_err(|e| DedupeError::InvalidOutputDir(format!("{}: {}", path.display(), e)))?;
    ensure_output_dir_is_writable(path)?;
    Ok(())
}

pub fn copy_unique_images(paths: &[PathBuf], output_dir: &Path) -> Result<Vec<PathBuf>> {
    copy_unique_images_with_progress(paths, output_dir, |_| {})
}

pub fn copy_non_unique_groups(
    groups: &[Vec<ImageEntry>],
    selected_paths: &[PathBuf],
    output_dir: &Path,
    locale: AppLocale,
    export_similar_image_groups: bool,
) -> Result<()> {
    let cancellation = NoopCancellationToken;
    copy_non_unique_groups_with_cancellation(
        groups,
        selected_paths,
        output_dir,
        locale,
        export_similar_image_groups,
        &cancellation,
    )
}

pub fn copy_non_unique_groups_with_cancellation(
    groups: &[Vec<ImageEntry>],
    selected_paths: &[PathBuf],
    output_dir: &Path,
    locale: AppLocale,
    export_similar_image_groups: bool,
    cancellation: &dyn CancellationToken,
) -> Result<()> {
    if !export_similar_image_groups {
        return Ok(());
    }

    let duplicate_groups: Vec<&Vec<ImageEntry>> =
        groups.iter().filter(|group| group.len() > 1).collect();
    let groups_dir = output_dir.join(non_unique_images_dir_name(locale));

    if duplicate_groups.is_empty() {
        if groups_dir.exists() {
            fs::remove_dir_all(&groups_dir)
                .map_err(|e| DedupeError::CopyImage(format!("{}: {}", groups_dir.display(), e)))?;
        }
        return Ok(());
    }

    let staging_dir = temp_work_dir(output_dir, "similar-groups-staging");
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir)
            .map_err(|e| DedupeError::CopyImage(format!("{}: {}", staging_dir.display(), e)))?;
    }
    fs::create_dir_all(&staging_dir)
        .map_err(|e| DedupeError::CopyImage(format!("{}: {}", staging_dir.display(), e)))?;

    let selected_paths: HashSet<&PathBuf> = selected_paths.iter().collect();
    for (group_index, group) in duplicate_groups.iter().enumerate() {
        ensure_not_cancelled(cancellation)?;
        let group_dir = staging_dir.join(format!("group-{:03}", group_index + 1));
        fs::create_dir_all(&group_dir)
            .map_err(|e| DedupeError::CopyImage(format!("{}: {}", group_dir.display(), e)))?;
        if let Err(error) =
            copy_group_images(group, &selected_paths, &group_dir, locale, cancellation)
        {
            let _ = fs::remove_dir_all(&staging_dir);
            return Err(error);
        }
    }

    replace_directory_atomically(&staging_dir, &groups_dir)?;
    Ok(())
}

pub fn copy_unique_images_with_progress<F>(
    paths: &[PathBuf],
    output_dir: &Path,
    mut on_progress: F,
) -> Result<Vec<PathBuf>>
where
    F: FnMut(usize),
{
    let cancellation = NoopCancellationToken;
    copy_unique_images_with_progress_and_cancellation(
        paths,
        output_dir,
        &cancellation,
        &mut on_progress,
    )
}

pub fn copy_unique_images_with_progress_and_cancellation<F>(
    paths: &[PathBuf],
    output_dir: &Path,
    cancellation: &dyn CancellationToken,
    mut on_progress: F,
) -> Result<Vec<PathBuf>>
where
    F: FnMut(usize),
{
    let mut used_names = HashSet::new();
    let mut outputs = Vec::new();

    for (idx, path) in paths.iter().enumerate() {
        if let Err(error) = ensure_not_cancelled(cancellation) {
            rollback_created_files(&outputs);
            return Err(error);
        }
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("image");
        let mut target = output_dir.join(filename);
        let mut counter = 1;

        while used_names.contains(&target) || target.exists() {
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let new_name = if ext.is_empty() {
                format!("{}-{}", stem, counter)
            } else {
                format!("{}-{}.{}", stem, counter, ext)
            };
            target = output_dir.join(new_name);
            counter += 1;
        }

        if let Err(error) = fs::copy(path, &target) {
            rollback_created_files(&outputs);
            return Err(DedupeError::CopyImage(format!(
                "{}: {}",
                path.display(),
                error
            )));
        }
        used_names.insert(target.clone());
        outputs.push(target);
        on_progress(idx + 1);
    }

    Ok(outputs)
}

fn copy_group_images(
    group: &[ImageEntry],
    selected_paths: &HashSet<&PathBuf>,
    output_dir: &Path,
    locale: AppLocale,
    cancellation: &dyn CancellationToken,
) -> Result<()> {
    let mut used_names = HashSet::new();

    for entry in group {
        ensure_not_cancelled(cancellation)?;
        let preferred_name =
            build_group_filename(&entry.path, selected_paths.contains(&entry.path), locale);
        let target = unique_target_path(output_dir, &preferred_name, &mut used_names);

        fs::copy(&entry.path, &target)
            .map_err(|e| DedupeError::CopyImage(format!("{}: {}", entry.path.display(), e)))?;
    }

    Ok(())
}

fn build_group_filename(
    path: &Path,
    is_selected_representative: bool,
    locale: AppLocale,
) -> String {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("image");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let base = if is_selected_representative {
        format!("{stem}{}", selected_representative_suffix(locale))
    } else {
        stem.to_string()
    };

    if ext.is_empty() {
        base
    } else {
        format!("{base}.{ext}")
    }
}

fn unique_target_path(
    output_dir: &Path,
    preferred_name: &str,
    used_names: &mut HashSet<PathBuf>,
) -> PathBuf {
    let preferred_path = Path::new(preferred_name);
    let stem = preferred_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image");
    let ext = preferred_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let mut target = output_dir.join(preferred_name);
    let mut counter = 1;

    while used_names.contains(&target) || target.exists() {
        let next_name = if ext.is_empty() {
            format!("{stem}-{counter}")
        } else {
            format!("{stem}-{counter}.{ext}")
        };
        target = output_dir.join(next_name);
        counter += 1;
    }

    used_names.insert(target.clone());
    target
}

fn non_unique_images_dir_name(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::En => "similar image groups",
        AppLocale::He => "קבוצות תמונות דומות",
    }
}

fn selected_representative_suffix(locale: AppLocale) -> &'static str {
    match locale {
        AppLocale::En => "__kept",
        AppLocale::He => "__נשמרה",
    }
}

fn ensure_output_dir_is_writable(path: &Path) -> Result<()> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let probe = path.join(format!(
        ".aba-avner-write-test-{}-{}",
        std::process::id(),
        unique
    ));

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
        .map_err(|error| DedupeError::InvalidOutputDir(format!("{}: {}", path.display(), error)))?;
    fs::remove_file(&probe)
        .map_err(|error| DedupeError::InvalidOutputDir(format!("{}: {}", path.display(), error)))?;
    Ok(())
}

fn rollback_created_files(paths: &[PathBuf]) {
    for path in paths.iter().rev() {
        let _ = fs::remove_file(path);
    }
}

fn ensure_not_cancelled(cancellation: &dyn CancellationToken) -> Result<()> {
    if cancellation.is_cancelled() {
        Err(DedupeError::Cancelled)
    } else {
        Ok(())
    }
}

fn temp_work_dir(output_dir: &Path, label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    output_dir.join(format!(
        ".aba-avner-{}-{}-{}",
        label,
        std::process::id(),
        unique
    ))
}

fn replace_directory_atomically(staging_dir: &Path, destination_dir: &Path) -> Result<()> {
    let backup_dir = temp_work_dir(
        destination_dir.parent().unwrap_or_else(|| Path::new(".")),
        "similar-groups-backup",
    );
    let had_existing_destination = destination_dir.exists();

    if had_existing_destination {
        fs::rename(destination_dir, &backup_dir).map_err(|error| {
            DedupeError::CopyImage(format!("{}: {}", destination_dir.display(), error))
        })?;
    }

    match fs::rename(staging_dir, destination_dir) {
        Ok(()) => {
            if had_existing_destination {
                fs::remove_dir_all(&backup_dir).map_err(|error| {
                    DedupeError::CopyImage(format!("{}: {}", backup_dir.display(), error))
                })?;
            }
            Ok(())
        }
        Err(error) => {
            if had_existing_destination {
                let _ = fs::rename(&backup_dir, destination_dir);
            }
            let _ = fs::remove_dir_all(staging_dir);
            Err(DedupeError::CopyImage(format!(
                "{}: {}",
                destination_dir.display(),
                error
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::AppLocale;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn duplicate_filenames_get_suffixed_predictably() {
        let temp = tempdir().unwrap();
        let output_dir = temp.path().join("output");
        let src_a = temp.path().join("a");
        let src_b = temp.path().join("b");
        fs::create_dir_all(src_a.join("nested")).unwrap();
        fs::create_dir_all(src_b.join("nested")).unwrap();

        let first = src_a.join("nested/photo.png");
        let second = src_b.join("nested/photo.png");
        fs::write(&first, b"one").unwrap();
        fs::write(&second, b"two").unwrap();

        ensure_output_dir(&output_dir).unwrap();
        let copied = copy_unique_images(&[first, second], &output_dir).unwrap();

        assert_eq!(copied[0], output_dir.join("photo.png"));
        assert_eq!(copied[1], output_dir.join("photo-1.png"));
    }

    #[test]
    fn existing_files_in_output_dir_are_not_overwritten() {
        let temp = tempdir().unwrap();
        let output_dir = temp.path().join("output");
        let source = temp.path().join("photo.png");

        ensure_output_dir(&output_dir).unwrap();
        fs::write(output_dir.join("photo.png"), b"existing").unwrap();
        fs::write(&source, b"new").unwrap();

        let copied = copy_unique_images(&[source], &output_dir).unwrap();

        assert_eq!(fs::read(output_dir.join("photo.png")).unwrap(), b"existing");
        assert_eq!(copied, vec![output_dir.join("photo-1.png")]);
        assert_eq!(fs::read(output_dir.join("photo-1.png")).unwrap(), b"new");
    }

    #[test]
    fn paths_without_extensions_still_copy_safely() {
        let temp = tempdir().unwrap();
        let output_dir = temp.path().join("output");
        let first = temp.path().join("plain");
        let second_dir = temp.path().join("nested");
        let second = second_dir.join("plain");

        ensure_output_dir(&output_dir).unwrap();
        fs::create_dir_all(&second_dir).unwrap();
        fs::write(&first, b"one").unwrap();
        fs::write(&second, b"two").unwrap();

        let copied = copy_unique_images(&[first, second], &output_dir).unwrap();

        assert_eq!(copied[0], output_dir.join("plain"));
        assert_eq!(copied[1], output_dir.join("plain-1"));
    }

    #[test]
    fn missing_source_file_returns_copy_error() {
        let temp = tempdir().unwrap();
        let output_dir = temp.path().join("output");
        let missing = temp.path().join("missing.png");

        ensure_output_dir(&output_dir).unwrap();

        let error = copy_unique_images(&[missing], &output_dir).unwrap_err();

        match error {
            DedupeError::CopyImage(message) => assert!(message.contains("missing.png")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn unique_copy_rolls_back_files_created_in_current_operation_after_failure() {
        let temp = tempdir().unwrap();
        let output_dir = temp.path().join("output");
        let first = temp.path().join("first.png");
        let missing = temp.path().join("missing.png");

        ensure_output_dir(&output_dir).unwrap();
        fs::write(&first, b"one").unwrap();

        let error = copy_unique_images(&[first, missing], &output_dir).unwrap_err();

        match error {
            DedupeError::CopyImage(message) => assert!(message.contains("missing.png")),
            other => panic!("unexpected error: {other:?}"),
        }
        assert!(!output_dir.join("first.png").exists());
    }

    #[test]
    fn ensure_output_dir_creates_nested_directories() {
        let temp = tempdir().unwrap();
        let nested = temp.path().join("deep/nested/output");

        ensure_output_dir(&nested).unwrap();

        assert!(nested.is_dir());
    }

    #[test]
    fn ensure_output_dir_confirms_directory_is_writable() {
        let temp = tempdir().unwrap();
        let output_dir = temp.path().join("output");

        ensure_output_dir(&output_dir).unwrap();

        assert_eq!(fs::read_dir(&output_dir).unwrap().count(), 0);
    }

    #[test]
    fn copy_non_unique_groups_exports_only_duplicate_groups_and_marks_selected_representative() {
        let temp = tempdir().unwrap();
        let output_dir = temp.path().join("output");
        let source_dir = temp.path().join("input");

        ensure_output_dir(&output_dir).unwrap();
        fs::create_dir_all(&source_dir).unwrap();

        let group_a_selected = source_dir.join("group-a-selected.png");
        let group_a_duplicate = source_dir.join("group-a-duplicate.png");
        let group_b_first = source_dir.join("group-b.png");
        let group_b_second = source_dir.join("nested/group-b.png");
        let unique = source_dir.join("unique.png");

        fs::create_dir_all(group_b_second.parent().unwrap()).unwrap();
        fs::write(&group_a_selected, b"a-selected").unwrap();
        fs::write(&group_a_duplicate, b"a-duplicate").unwrap();
        fs::write(&group_b_first, b"b-first").unwrap();
        fs::write(&group_b_second, b"b-second").unwrap();
        fs::write(&unique, b"unique").unwrap();

        let groups = vec![
            vec![
                ImageEntry {
                    path: group_a_selected.clone(),
                    hash: 1,
                    size_bytes: 0,
                },
                ImageEntry {
                    path: group_a_duplicate.clone(),
                    hash: 1,
                    size_bytes: 0,
                },
            ],
            vec![
                ImageEntry {
                    path: group_b_first.clone(),
                    hash: 2,
                    size_bytes: 0,
                },
                ImageEntry {
                    path: group_b_second.clone(),
                    hash: 2,
                    size_bytes: 0,
                },
            ],
            vec![ImageEntry {
                path: unique,
                hash: 3,
                size_bytes: 0,
            }],
        ];

        copy_non_unique_groups(
            &groups,
            &[group_a_selected.clone(), group_b_first.clone()],
            &output_dir,
            AppLocale::En,
            true,
        )
        .unwrap();

        let non_unique_dir = output_dir.join(non_unique_images_dir_name(AppLocale::En));
        assert!(non_unique_dir.is_dir());
        assert!(!non_unique_dir.join("group-003").exists());

        let group_a_dir = non_unique_dir.join("group-001");
        assert_eq!(
            fs::read(group_a_dir.join("group-a-selected__kept.png")).unwrap(),
            b"a-selected"
        );
        assert_eq!(
            fs::read(group_a_dir.join("group-a-duplicate.png")).unwrap(),
            b"a-duplicate"
        );

        let group_b_dir = non_unique_dir.join("group-002");
        assert_eq!(
            fs::read(group_b_dir.join("group-b__kept.png")).unwrap(),
            b"b-first"
        );
        assert_eq!(
            fs::read(group_b_dir.join("group-b.png")).unwrap(),
            b"b-second"
        );
    }

    #[test]
    fn copy_non_unique_groups_removes_stale_generated_folder() {
        let temp = tempdir().unwrap();
        let output_dir = temp.path().join("output");
        let source_a = temp.path().join("a/duplicate.png");
        let source_b = temp.path().join("b/duplicate.png");
        let stale = output_dir
            .join(non_unique_images_dir_name(AppLocale::En))
            .join("stale.txt");

        ensure_output_dir(&output_dir).unwrap();
        fs::create_dir_all(source_a.parent().unwrap()).unwrap();
        fs::create_dir_all(source_b.parent().unwrap()).unwrap();
        fs::create_dir_all(stale.parent().unwrap()).unwrap();
        fs::write(&stale, b"stale").unwrap();
        fs::write(&source_a, b"dup-a").unwrap();
        fs::write(&source_b, b"dup-b").unwrap();

        copy_non_unique_groups(
            &[vec![
                ImageEntry {
                    path: source_a,
                    hash: 1,
                    size_bytes: 0,
                },
                ImageEntry {
                    path: source_b,
                    hash: 1,
                    size_bytes: 0,
                },
            ]],
            &[],
            &output_dir,
            AppLocale::En,
            true,
        )
        .unwrap();

        assert!(!stale.exists());
        assert!(output_dir
            .join(non_unique_images_dir_name(AppLocale::En))
            .join("group-001")
            .is_dir());
    }

    #[test]
    fn copy_non_unique_groups_uses_hebrew_names_for_hebrew_locale() {
        let temp = tempdir().unwrap();
        let output_dir = temp.path().join("output");
        let source_a = temp.path().join("a/photo.png");
        let source_b = temp.path().join("b/photo.png");

        ensure_output_dir(&output_dir).unwrap();
        fs::create_dir_all(source_a.parent().unwrap()).unwrap();
        fs::create_dir_all(source_b.parent().unwrap()).unwrap();
        fs::write(&source_a, b"one").unwrap();
        fs::write(&source_b, b"two").unwrap();

        copy_non_unique_groups(
            &[vec![
                ImageEntry {
                    path: source_a.clone(),
                    hash: 1,
                    size_bytes: 0,
                },
                ImageEntry {
                    path: source_b,
                    hash: 1,
                    size_bytes: 0,
                },
            ]],
            &[source_a],
            &output_dir,
            AppLocale::He,
            true,
        )
        .unwrap();

        let group_dir = output_dir
            .join(non_unique_images_dir_name(AppLocale::He))
            .join("group-001");
        assert!(group_dir.join("photo__נשמרה.png").exists());
        assert!(group_dir.join("photo.png").exists());
    }

    #[test]
    fn copy_non_unique_groups_does_not_touch_existing_folder_when_disabled() {
        let temp = tempdir().unwrap();
        let output_dir = temp.path().join("output");
        let source_a = temp.path().join("a/photo.png");
        let source_b = temp.path().join("b/photo.png");
        let stale = output_dir
            .join(non_unique_images_dir_name(AppLocale::En))
            .join("group-001")
            .join("stale.txt");

        ensure_output_dir(&output_dir).unwrap();
        fs::create_dir_all(source_a.parent().unwrap()).unwrap();
        fs::create_dir_all(source_b.parent().unwrap()).unwrap();
        fs::create_dir_all(stale.parent().unwrap()).unwrap();
        fs::write(&source_a, b"one").unwrap();
        fs::write(&source_b, b"two").unwrap();
        fs::write(&stale, b"stale").unwrap();

        copy_non_unique_groups(
            &[vec![
                ImageEntry {
                    path: source_a,
                    hash: 1,
                    size_bytes: 0,
                },
                ImageEntry {
                    path: source_b,
                    hash: 1,
                    size_bytes: 0,
                },
            ]],
            &[],
            &output_dir,
            AppLocale::En,
            false,
        )
        .unwrap();

        assert_eq!(fs::read(&stale).unwrap(), b"stale");
    }

    #[test]
    fn non_unique_group_refresh_preserves_previous_export_when_new_export_fails() {
        let temp = tempdir().unwrap();
        let output_dir = temp.path().join("output");
        let source_a = temp.path().join("a/photo.png");
        let missing = temp.path().join("b/photo.png");
        let stale = output_dir
            .join(non_unique_images_dir_name(AppLocale::En))
            .join("group-001")
            .join("stale.txt");

        ensure_output_dir(&output_dir).unwrap();
        fs::create_dir_all(source_a.parent().unwrap()).unwrap();
        fs::create_dir_all(stale.parent().unwrap()).unwrap();
        fs::write(&source_a, b"one").unwrap();
        fs::write(&stale, b"stale").unwrap();

        let error = copy_non_unique_groups(
            &[vec![
                ImageEntry {
                    path: source_a,
                    hash: 1,
                    size_bytes: 0,
                },
                ImageEntry {
                    path: missing.clone(),
                    hash: 1,
                    size_bytes: 0,
                },
            ]],
            &[],
            &output_dir,
            AppLocale::En,
            true,
        )
        .unwrap_err();

        match error {
            DedupeError::CopyImage(message) => assert!(message.contains("photo.png")),
            other => panic!("unexpected error: {other:?}"),
        }
        assert_eq!(fs::read(&stale).unwrap(), b"stale");
    }
}
