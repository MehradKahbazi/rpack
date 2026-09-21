use std::path::{Path, PathBuf};

pub fn resolve(importer: &Path, dependency: &str) -> Result<PathBuf, String> {
    let importer_dir = importer.parent().ok_or_else(|| {
        format!(
            "Cannot determine parent directory of '{}'",
            importer.display()
        )
    })?;

    let resolved = importer_dir.join(dependency);

    if !resolved.exists() {
        return Err(format!(
            "Cannot resolve '{}' from '{}'",
            dependency,
            importer.display()
        ));
    }

    Ok(resolved)
}
