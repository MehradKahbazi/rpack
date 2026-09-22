use std::path::{Path, PathBuf};

const EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx"];

pub fn resolve(importer: &Path, dependency: &str) -> Result<PathBuf, String> {
    if !dependency.starts_with('.') {
        return Err(format!(
            "Unsupported package import '{}' from '{}'",
            dependency,
            importer.display()
        ));
    }

    let importer_dir = importer.parent().ok_or_else(|| {
        format!(
            "Cannot determine parent directory of '{}'",
            importer.display()
        )
    })?;

    let requested = importer_dir.join(dependency);

    resolve_path(&requested).ok_or_else(|| {
        format!(
            "Cannot resolve '{}' from '{}'",
            dependency,
            importer.display()
        )
    })
}

fn resolve_path(path: &Path) -> Option<PathBuf> {
    // 1. Exact file
    if path.is_file() {
        return canonicalize(path);
    }

    // 2. File with supported extension
    if path.extension().is_none() {
        for extension in EXTENSIONS {
            let candidate = path.with_extension(extension);

            if candidate.is_file() {
                return canonicalize(&candidate);
            }
        }
    }

    // 3. Directory index file
    if path.is_dir() {
        for extension in EXTENSIONS {
            let candidate = path.join(format!("index.{extension}"));

            if candidate.is_file() {
                return canonicalize(&candidate);
            }
        }
    }

    None
}

fn canonicalize(path: &Path) -> Option<PathBuf> {
    path.canonicalize().ok()
}
