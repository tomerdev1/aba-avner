use crate::errors::{DedupeError, Result};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn normalize_input_dir(path: &Path) -> Result<PathBuf> {
    canonicalize_directory(path, DedupeError::InvalidInputDir)
}

pub fn normalize_output_dir(path: &Path) -> Result<PathBuf> {
    let absolute = absolutize_path(path).map_err(DedupeError::InvalidOutputDir)?;
    let normalized = if absolute.exists() {
        canonicalize_directory(&absolute, DedupeError::InvalidOutputDir)?
    } else {
        canonicalize_with_missing_tail(&absolute).map_err(DedupeError::InvalidOutputDir)?
    };

    if is_filesystem_root(&normalized) {
        return Err(DedupeError::InvalidOutputDir(format!(
            "{} is not a safe output directory",
            normalized.display()
        )));
    }

    Ok(normalized)
}

fn absolutize_path(path: &Path) -> std::result::Result<PathBuf, String> {
    let normalized = normalize_path(path);
    if normalized.is_absolute() {
        return Ok(normalized);
    }

    env::current_dir()
        .map(|cwd| normalize_path(&cwd.join(normalized)))
        .map_err(|error| error.to_string())
}

fn canonicalize_directory(path: &Path, error: fn(String) -> DedupeError) -> Result<PathBuf> {
    let metadata = fs::metadata(path).map_err(|_| error(path.display().to_string()))?;
    if !metadata.is_dir() {
        return Err(error(path.display().to_string()));
    }

    fs::canonicalize(path).map_err(|_| error(path.display().to_string()))
}

fn canonicalize_with_missing_tail(path: &Path) -> std::result::Result<PathBuf, String> {
    let mut existing = path;
    let mut missing_tail = Vec::<OsString>::new();

    while !existing.exists() {
        let name = existing
            .file_name()
            .ok_or_else(|| path.display().to_string())?;
        missing_tail.push(name.to_os_string());
        existing = existing
            .parent()
            .ok_or_else(|| path.display().to_string())?;
    }

    let mut canonical = fs::canonicalize(existing).map_err(|error| error.to_string())?;
    for segment in missing_tail.iter().rev() {
        canonical.push(segment);
    }

    Ok(normalize_path(&canonical))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(
                    normalized.components().next_back(),
                    Some(Component::Normal(_))
                ) {
                    normalized.pop();
                } else {
                    normalized.push(component.as_os_str());
                }
            }
            other => normalized.push(other.as_os_str()),
        }
    }

    normalized
}

fn is_filesystem_root(path: &Path) -> bool {
    let mut components = path.components();
    match (components.next(), components.next()) {
        (Some(Component::RootDir), None) => true,
        (Some(Component::Prefix(_)), None) => true,
        (Some(Component::Prefix(_)), Some(Component::RootDir)) if components.next().is_none() => {
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_input_dir, normalize_output_dir};
    use crate::errors::DedupeError;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn normalize_input_dir_returns_canonical_directory() {
        let temp = tempdir().unwrap();
        let nested = temp.path().join("input");
        fs::create_dir_all(&nested).unwrap();

        let normalized = normalize_input_dir(&nested).unwrap();

        assert_eq!(normalized, fs::canonicalize(&nested).unwrap());
    }

    #[test]
    fn normalize_output_dir_rejects_root_directory() {
        let error = normalize_output_dir(Path::new("/")).unwrap_err();

        match error {
            DedupeError::InvalidOutputDir(message) => {
                assert!(message.contains("not a safe output directory"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn normalize_output_dir_resolves_missing_segments_against_existing_ancestor() {
        let temp = tempdir().unwrap();
        let output = temp.path().join("nested/output");

        let normalized = normalize_output_dir(&output).unwrap();

        assert_eq!(normalized, temp.path().join("nested/output"));
    }
}
