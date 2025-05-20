

pub mod waldirconfigs {
    use walkdir::WalkDir;
    use std::path::PathBuf;

    pub fn getsrc_files() -> Vec<PathBuf> {
        WalkDir::new("./")
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;  // If error, return None
                if entry.file_type().is_file() {
                    Some(entry.into_path())
                } else {
                    None
                }
            })
            .collect()
    }
}