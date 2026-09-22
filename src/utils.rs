use std::path::Path;

pub fn module_id(path: &Path) -> String {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .expect("Failed to get current directory")
            .join(path)
    };

    let mut id = absolute.to_string_lossy().replace('\\', "/");

    // Remove Windows verbatim path prefix.
    if let Some(stripped) = id.strip_prefix("//?/") {
        id = stripped.to_string();
    }

    id
}
