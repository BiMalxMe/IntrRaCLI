

pub mod waldirconfigs {
    use walkdir::WalkDir;
    use std::path::PathBuf;
    // ./All/rust/rust-proj/src
    pub fn getsrc_files() -> Vec<PathBuf> {
        WalkDir::new("../")
        // only the top level file not going deep
        .max_depth(1)
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;  // If error, return None
                // entering only files
                if entry.file_type().is_file() {
                    Some(entry.into_path())
                } else {
                    None
                }
            })
            .collect()
    }
}