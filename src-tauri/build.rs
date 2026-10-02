use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const BUILD_INPUTS: &[&str] = &[
    "index.html",
    "app.js",
    "styles.css",
    "visual-harness.js",
    "src",
    "styles",
];

fn main() {
    if std::env::var_os("CARGO_FEATURE_GUI").is_some() {
        prepare_ui_dist().expect("failed to prepare ui/dist");
        tauri_build::build();
    }
}

fn prepare_ui_dist() -> io::Result<()> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("missing CARGO_MANIFEST_DIR"));
    let ui_dir = manifest_dir.join("../ui");
    let dist_dir = ui_dir.join("dist");

    println!("cargo:rerun-if-changed={}", manifest_dir.join("tauri.conf.json").display());
    for entry in BUILD_INPUTS {
        println!("cargo:rerun-if-changed={}", ui_dir.join(entry).display());
    }

    for entry in BUILD_INPUTS {
        let source = ui_dir.join(entry);
        if !source.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("missing required UI build input: {}", source.display()),
            ));
        }
    }

    clear_dir(&dist_dir)?;
    for entry in BUILD_INPUTS {
        copy_entry(&ui_dir.join(entry), &dist_dir.join(entry))?;
    }

    Ok(())
}

fn clear_dir(dir: &Path) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn copy_entry(source: &Path, target: &Path) -> io::Result<()> {
    if source.is_dir() {
        fs::create_dir_all(target)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_entry(&entry.path(), &target.join(entry.file_name()))?;
        }
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, target)?;
    Ok(())
}
