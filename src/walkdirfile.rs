

pub mod waldirconfigs {
    use walkdir::WalkDir;
    use std::path:: PathBuf;
    // ./All/rust/rust-proj/src
    pub fn getsrc_files(path : PathBuf) -> Vec<PathBuf> {
        WalkDir::new(path)
        // only the top level file not going deep
        .max_depth(1)
            .into_iter()
            .filter_map(|entry| {
                entry.ok().map(|e| e.into_path())

            })
            .collect()
    }

    pub fn get_dir_datas(path : PathBuf) -> Vec<PathBuf> {
        WalkDir::new(path)
        .max_depth(1)
        .into_iter()
        .filter_map( | entry | {
            entry.ok().map(|e| e.into_path())
        } )
        .collect()
    }
}