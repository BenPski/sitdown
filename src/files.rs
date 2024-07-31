use std::{
    fs, io,
    path::{Path, PathBuf},
};

use minijinja::Environment;

use crate::{config::Config, document::Document};

/// representation of the file structure
/// make use of Object interface in jinja to make each piece have field accessors

/// some kind of file, either a page/file or a directory
#[derive(Debug, Clone)]
pub enum File<'a> {
    Page(&'a Page),
    Dir(&'a Dir),
}

#[derive(Debug, Clone)]
pub struct Page {
    path: PathBuf,
}

impl AsRef<Path> for Page {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

#[derive(Debug, Clone)]
pub struct Dir {
    path: PathBuf,
    subdirs: Vec<Dir>,
    pages: Vec<Page>,
}

// impl<'a> File<'a> {
//     pub fn new<T: AsRef<Path>>(path: T) -> Self {
//         if path.as_ref().is_dir() {
//             File::Dir(&Dir::new(path.as_ref()))
//         } else {
//             File::Page(&Page::new(path.))
//         }
//     }
// }

impl Page {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        Page {
            path: path.as_ref().into(),
        }
    }
}

impl Dir {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        let path = path.as_ref();
        let mut subdirs = Vec::new();
        let mut pages = Vec::new();
        for entry in path.read_dir().expect("Couldn't read directory") {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                subdirs.push(Dir::new(entry.path()));
            } else {
                pages.push(Page::new(entry.path()));
            }
        }
        Dir {
            path: path.into(),
            subdirs,
            pages,
        }
    }

    // // a dumb way of making the iterator that should be improved
    // pub fn children(self) -> Vec<File> {
    //     let mut files = Vec::new();
    //     for page in self.pages {
    //         files.push(File::Page(page));
    //     }
    //     for dir in self.subdirs {
    //         files.push(File::Dir(dir));
    //     }
    //     files
    // }

    pub fn pages<'a>(&'a self) -> PagesIter<'a> {
        PagesIter {
            pages: &self.pages,
            index: 0,
        }
    }

    pub fn dirs<'a>(&'a self) -> DirsIter<'a> {
        DirsIter {
            dirs: &self.subdirs,
            index: 0,
        }
    }

    pub fn children<'a>(&'a self) -> impl Iterator<Item = File<'a>> {
        self.pages()
            .map(|p| File::Page(p))
            .chain(self.dirs().map(|d| File::Dir(d)))
    }

    pub fn copy_to<T: AsRef<Path>>(&self, to: T) -> io::Result<()> {
        fs::create_dir_all(&to)?;
        for page in &self.pages {
            fs::copy(&page.path, to.as_ref().join(page.path.file_name().unwrap()))?;
        }
        for dir in &self.subdirs {
            dir.copy_to(to.as_ref().join(dir.path.file_name().unwrap()))?;
        }
        Ok(())
    }

    pub fn documents(&self, config: &Config) -> Vec<Document> {
        let mut res: Vec<Document> = self.pages().map(|p| Document::new(config, p)).collect();
        let subdirs = self.dirs().flat_map(|d| d.documents(config));
        res.extend(subdirs);
        res
    }

    // pub fn create_files<'a>(
    //     &self,
    //     config: &Config,
    //     templates: &Environment<'a>,
    // ) -> std::io::Result<()> {
    //     for page in self.pages() {
    //         let doc = Document::new(config, page);
    //         doc.create(templates);
    //     }
    //     Ok(())
    // }
}

/// iterator for the pages in a directory
pub struct PagesIter<'a> {
    pages: &'a [Page],
    index: usize,
}

impl<'a> Iterator for PagesIter<'a> {
    type Item = &'a Page;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.pages.len() {
            let res = &self.pages[self.index];
            self.index += 1;
            Some(res)
        } else {
            None
        }
    }
}

/// iterator for the subdirectories in a directory
pub struct DirsIter<'a> {
    dirs: &'a [Dir],
    index: usize,
}

impl<'a> Iterator for DirsIter<'a> {
    type Item = &'a Dir;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.dirs.len() {
            let res = &self.dirs[self.index];
            self.index += 1;
            Some(res)
        } else {
            None
        }
    }
}
