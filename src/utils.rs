use std::{fs, io, path::Path};

use crate::{
    file_tree::{Dir, File},
    info::{DirInfo, PageInfo},
};

pub fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir(entry.path(), to.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), to.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

pub fn extract_work(dir: &Dir, dir_info: &mut Vec<DirInfo>, page_info: &mut Vec<PageInfo>) {
    dir_info.push(DirInfo::from(dir));
    for child in &dir.children {
        match child {
            File::Dir(d) => {
                extract_work(d, dir_info, page_info);
            }
            File::Page(p) => {
                page_info.push(PageInfo::from(p));
            }
        }
    }
}
