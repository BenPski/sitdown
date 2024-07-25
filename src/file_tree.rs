use std::path::PathBuf;

#[derive(Debug)]
pub enum File {
    Dir(Dir),
    Page(Page),
}

impl File {
    pub fn from_path(path: PathBuf) -> Option<Self> {
        if path.is_dir() {
            Dir::from_path(path).map(File::Dir)
        } else {
            Page::from_path(path).map(File::Page)
        }
    }
}
#[derive(Debug)]
pub struct Dir {
    pub path: PathBuf,
    pub children: Vec<File>,
}

impl Dir {
    pub fn from_path(path: PathBuf) -> Option<Self> {
        if path.is_dir() {
            let children = path
                .read_dir()
                .expect("Unable to read directory")
                .filter_map(|c| c.ok().and_then(|p| File::from_path(p.path())))
                .collect();
            Some(Dir { path, children })
        } else {
            None
        }
    }

    // extract the the info for the contents of the directory
    // fn extract(&self) -> (Vec<DirInfo>, Vec<PageInfo>) {
    //     let mut dir_info = Vec::new();
    //     let mut page_info = Vec::new();
    //     self.extract_work(&mut dir_info, &mut page_info);
    //     (dir_info, page_info)
    // }
}

#[derive(Debug)]
pub struct Page {
    pub path: PathBuf,
}

impl Page {
    pub fn from_path(path: PathBuf) -> Option<Self> {
        if !path.is_dir() {
            Some(Self { path })
        } else {
            None
        }
    }
}
