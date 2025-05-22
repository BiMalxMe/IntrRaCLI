pub mod waldirconfigs {
    use walkdir::WalkDir;
    use std::path::PathBuf;

    pub fn get_dir_datas(path: PathBuf) -> Vec<(String, PathBuf)> {
        WalkDir::new(path)
            .max_depth(1)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                if let Some(name) = entry.file_name().to_str() {
                    !name.starts_with('.') // Exclude hidden
                } else {
                    false
                }
            })
            .map(|entry| {
                let path = entry.path().to_path_buf();
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");

                let prefix = if path.is_dir() {
                    "📁"
                } else {
                    "📄"
                };

                (format!("{} {}", prefix, name), path)
            })
            .collect()
    }
}
