use std::path::{Path, PathBuf};

pub fn resolve(importer: &Path, dependency: &str) -> Result<PathBuf, String> {
    let importer_dir = importer.parent().ok_or_else(|| {
        format!(
            "Cannot determine parent directory of '{}'",
            importer.display()
        )
    })?;

    let requested = importer_dir.join(dependency);

    if requested.exists() && requested.is_file() {
        return Ok(requested);
    }

    if requested.extension().is_none() {
        let with_js = requested.with_extension("js");

        if with_js.exists() && with_js.is_file() {
            return Ok(with_js);
        }
    }

    Err(format!(
        "Cannot resolve '{}' from '{}'",
        dependency,
        importer.display()
    ))
}
