use std::path::{Path, PathBuf};

/// extract default data from a path

#[derive(Debug)]
pub struct PathData {
    pub title: String,
    pub location: PathBuf,
}

impl PathData {
    // doesn't include base directory (aka the location is path rather than _site/path)
    pub fn from<T: AsRef<Path>>(path: T) -> Self {
        let path = path.as_ref();
        let title = path.file_stem().unwrap().to_str().unwrap().into();
        let mut location = PathBuf::new();
        for component in path.components().skip(1) {
            location.push(component);
        }
        location.set_extension("html");
        Self { title, location }
    }
}
