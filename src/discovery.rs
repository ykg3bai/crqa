use std::io;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

use crate::config::Target;
use crate::model::Language;

pub fn collect_files(targets: &[Target]) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for target in targets {
        match target {
            Target::File(path) => {
                if Language::from_path(path).is_some() {
                    files.push(path.clone());
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("unsupported source file: {}", path.display()),
                    ));
                }
            }
            Target::Directory(path) => collect_supported_files(path, &mut files)?,
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

fn collect_supported_files(path: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    let walker = WalkBuilder::new(path)
        .hidden(false)
        .standard_filters(true)
        .filter_entry(|entry| {
            entry
                .file_name()
                .to_str()
                .is_none_or(|name| !matches!(name, ".git" | "build" | "target"))
        })
        .build();

    for result in walker {
        let entry = result.map_err(io::Error::other)?;
        let path = entry.path();
        if path.is_file() && Language::from_path(path).is_some() {
            files.push(path.to_path_buf());
        }
    }
    Ok(())
}
