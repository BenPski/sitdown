use minijinja::{value::Object, Environment};
use pulldown_cmark::{
    CowStr::Borrowed,
    Event::{Start, Text},
    Options, Parser, Tag, TextMergeStream,
};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use serde_yml::Value;

use crate::error::Result;
use crate::{config::ConfigDefaults, error::Error, META_FILE};

// -----
// The tree datastructure
// -----

/// the general file tree that should contain the structure of the site
/// a directory or directory-like thing
#[derive(Debug)]
pub struct Dir<DirData, PageData> {
    pub data: DirData,
    pub pages: Vec<Page<PageData>>,
    pub dirs: Vec<Dir<DirData, PageData>>,
}

/// a file or file-like thing
#[derive(Debug)]
pub struct Page<PageData> {
    pub data: PageData,
}

/// the initial parsing of the file system should result in a file system with
/// path to the actual file system

impl<D, P> Dir<D, P> {
    pub fn pages<'a>(&'a self) -> PagesIter<'a, P> {
        PagesIter {
            pages: &self.pages,
            index: 0,
        }
    }
    pub fn dirs<'a>(&'a self) -> DirsIter<'a, D, P> {
        DirsIter {
            dirs: &self.dirs,
            index: 0,
        }
    }
    pub fn entries<'a>(&'a self) -> impl Iterator<Item = File<'a, D, P>> {
        self.pages()
            .map(|p| File::Page(p))
            .chain(self.dirs().map(|d| File::Dir(d)))
    }
}

/// iterator for the pages in a directory
pub struct PagesIter<'a, PageData> {
    pages: &'a [Page<PageData>],
    index: usize,
}

impl<'a, P> Iterator for PagesIter<'a, P> {
    type Item = &'a Page<P>;
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
pub struct DirsIter<'a, D, P> {
    dirs: &'a [Dir<D, P>],
    index: usize,
}

impl<'a, D, P> Iterator for DirsIter<'a, D, P> {
    type Item = &'a Dir<D, P>;
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

/// allows for unifying the pages and dirs into a single type if that ends up being useful
#[derive(Debug)]
pub enum File<'a, D, P> {
    Page(&'a Page<P>),
    Dir(&'a Dir<D, P>),
}

// -----
// The file sytem stored in the tree
// -----

/// read the contents of a directory
pub fn load_contents<T: AsRef<Path>>(path: T) -> std::io::Result<Dir<PathBuf, PathBuf>> {
    let path = path.as_ref();
    let mut pages = Vec::new();
    let mut dirs = Vec::new();
    for entry in path.read_dir()? {
        let entry = entry?;
        if entry.path().is_dir() {
            dirs.push(load_contents(entry.path())?);
        } else {
            pages.push(Page { data: entry.path() })
        }
    }
    Ok(Dir {
        data: path.into(),
        pages,
        dirs,
    })
}

/// annotate the directory structure so it has all the data needed to write out
/// the files
impl Dir<PathBuf, PathBuf> {
    pub fn annotate(
        &self,
        config: &ConfigDefaults,
        options: &Options,
    ) -> Result<Dir<DirInfo, PageInfo>> {
        let pages: Result<Vec<_>> = self.pages().map(|p| p.annotate(config, options)).collect();
        let dirs: Result<Vec<_>> = self.dirs().map(|d| d.annotate(config, options)).collect();
        Ok(Dir {
            data: DirInfo::new(&self.data)?,
            pages: pages?,
            dirs: dirs?,
        })
    }

    pub fn copy_to<T: AsRef<Path>>(&self, to: T) -> std::io::Result<()> {
        fs::create_dir_all(to.as_ref().join(&self.data))?;
        for dir in self.dirs() {
            dir.copy_to(to.as_ref())?;
        }
        for page in self.pages() {
            fs::copy(&page.data, to.as_ref().join(&page.data))?;
        }
        Ok(())
    }
}

impl Page<PathBuf> {
    pub fn annotate(&self, config: &ConfigDefaults, options: &Options) -> Result<Page<PageInfo>> {
        PageInfo::new(&self.data, config, options).map(|x| Page { data: x })
    }
}

// -----
// The tree after parsing the files in the file system
// -----

/// write the metadata out to a working directory so that it can be read as needed when
/// generating the actual pages, then keep track of the paths to the metadata or directory
/// paths
impl Dir<DirInfo, PageInfo> {
    pub fn write_metadata<T: AsRef<Path>>(&self, work_dir: T) -> Result<Dir<DirPath, PagePath>> {
        let dir_path = work_dir.as_ref().join(&self.data.save);
        fs::create_dir_all(&dir_path)?;
        let contents = serde_yml::to_string(&self.data)?;
        fs::write(dir_path.join(META_FILE), contents)?;
        let data = DirPath {
            path: dir_path,
            orig: self.data.save.clone(),
        };

        let dirs: Result<Vec<_>> = self
            .dirs()
            .map(|d| d.write_metadata(work_dir.as_ref()))
            .collect();
        let pages: Result<Vec<_>> = self
            .pages()
            .map(|p| p.write_metadata(work_dir.as_ref()))
            .collect();

        Ok(Dir {
            data,
            dirs: dirs?,
            pages: pages?,
        })
    }
}

impl Page<PageInfo> {
    fn write_metadata<T: AsRef<Path>>(&self, work_dir: T) -> Result<Page<PagePath>> {
        let mut page_path = work_dir.as_ref().join(&self.data.save);
        page_path.set_extension("yaml");
        let content = serde_yml::to_string(&self.data)?;
        fs::write(&page_path, content)?;
        let data = PagePath {
            path: page_path,
            orig: self.data.save.clone(),
            template: self.data.template.clone(),
        };
        Ok(Page { data })
    }
}

/// data for creating the output directory
#[derive(Debug, Serialize, Deserialize)]
pub struct DirInfo {
    /// location to save the directory to
    save: PathBuf,
    /// name of the directory
    title: String,
}

impl DirInfo {
    fn new<T: AsRef<Path>>(path: T) -> Result<Self> {
        PathInfo::dir(path).map(|x| DirInfo {
            title: x.title,
            save: x.save,
        })
    }
}

/// data for creating the output file
#[derive(Debug, Serialize, Deserialize)]
pub struct PageInfo {
    /// the name of the template to use
    template: String,
    /// location to save file to
    save: PathBuf,
    /// the title for the file
    title: String,
    /// the data to be provided to the template
    #[serde(flatten)]
    meta: Metadata,
}

impl PageInfo {
    fn new<T: AsRef<Path>>(path: T, config: &ConfigDefaults, options: &Options) -> Result<Self> {
        let path = path.as_ref();
        let info = PathInfo::page(&path)?;
        let mut page_content = PageContent::read(&path, options)?;
        let title = if let Some(t) = page_content.meta.remove("title") {
            t.as_str().unwrap_or(&info.title).into()
        } else {
            info.title
        };
        let template = if let Some(t) = page_content.meta.remove("template") {
            t.as_str().unwrap_or(&config.page).into()
        } else {
            config.page.clone()
        };
        let save = info.save;
        let meta = Metadata {
            contents: page_content.contents,
            meta: page_content.meta,
        };

        Ok(Self {
            template,
            save,
            title,
            meta,
        })
    }
}

struct PageContent {
    contents: String,
    meta: HashMap<String, Value>,
}

impl PageContent {
    fn read<T: AsRef<Path>>(path: T, options: &Options) -> Result<PageContent> {
        let text = fs::read_to_string(path)?;
        let parser = Parser::new_ext(&text, *options);
        let mut iterator = TextMergeStream::new(parser).peekable();

        let meta = if let Some(Start(Tag::MetadataBlock(_))) = iterator.peek() {
            iterator.next();
            let info = if let Some(Text(Borrowed(s))) = iterator.peek() {
                let res = serde_yml::from_str(s)?;
                iterator.next(); // skip the parsed data
                iterator.next(); // skip the End token
                res
            } else {
                HashMap::new()
            };
            info
        } else {
            HashMap::new()
        };
        let mut contents = String::new();
        pulldown_cmark::html::push_html(&mut contents, iterator);

        Ok(Self { contents, meta })
    }
}

/// data that can be extracted from the original path
struct PathInfo {
    /// the title of the file
    title: String,
    /// save location of the file
    save: PathBuf,
}

/// based on how the path is retrieved this should really be infallible
impl PathInfo {
    /// the generated page info from it's path
    fn page<T: AsRef<Path>>(path: T) -> Result<Self> {
        let path = path.as_ref();
        let title = path
            .file_stem()
            .ok_or_else(|| Error::PageError(path.into()))?
            .to_str()
            .ok_or_else(|| Error::PageError(path.into()))?
            .replace("_", " ")
            .into();
        let mut save = PathBuf::new();
        for component in path.components().skip(1) {
            save.push(component);
        }
        save.set_extension("html");
        Ok(Self { title, save })
    }
    /// the generated dir info from it's path
    fn dir<T: AsRef<Path>>(path: T) -> Result<Self> {
        let path = path.as_ref();
        let title = path
            .file_name()
            .ok_or_else(|| Error::DirError(path.into()))?
            .to_str()
            .ok_or_else(|| Error::DirError(path.into()))?
            .replace("_", " ")
            .into();
        let mut save = PathBuf::new();
        for component in path.components().skip(1) {
            save.push(component);
        }
        Ok(Self { title, save })
    }
}

/// generic metadata for a file that includes the defaults from the config and the defaults from
/// the file path
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Metadata {
    contents: String,
    #[serde(flatten)]
    meta: HashMap<String, Value>,
}

// -----
// The tree for handling putting together all the metadata after it has been written out
// -----

/// with the metadata written out traverse the path tree and pull in metadata as needed
impl Dir<DirPath, PagePath> {
    pub fn create<'a, T: AsRef<Path>>(
        &self,
        out_dir: T,
        templates: &Environment<'a>,
    ) -> Result<()> {
        self.create_with(out_dir, templates, self, self)
    }

    fn create_with<'a, T: AsRef<Path>>(
        &self,
        out_dir: T,
        templates: &Environment<'a>,
        root: &'a Self,
        parent: &'a Self,
    ) -> Result<()> {
        for dir in self.dirs() {
            fs::create_dir_all(out_dir.as_ref().join(&dir.data.orig))?;
            dir.create_with(out_dir.as_ref(), templates, root, dir)?;
        }
        for page in self.pages() {
            page.create_with(out_dir.as_ref(), templates, root, parent)?;
        }
        Ok(())
    }
}

impl Page<PagePath> {
    fn create_with<'a, T: AsRef<Path>>(
        &self,
        out_dir: T,
        templates: &Environment<'a>,
        root: &'a Dir<DirPath, PagePath>,
        parent: &'a Dir<DirPath, PagePath>,
    ) -> Result<()> {
        let contents = fs::read_to_string(&self.data.path)?;
        let metadata: Metadata = serde_yml::from_str(&contents)?;
        let meta = MetaObject {
            root: root.data.clone(),
            parent: parent.data.clone(),
            page: metadata,
        };
        let template = templates.get_template(&self.data.template)?;
        let content = template.render(minijinja::Value::from_object(meta))?;
        fs::write(out_dir.as_ref().join(&self.data.orig), content)?;
        Ok(())
    }
}

#[derive(Debug)]
struct MetaObject {
    root: DirPath,
    parent: DirPath,
    page: Metadata,
}

#[derive(Debug, Clone)]
pub struct DirPath {
    path: PathBuf,
    orig: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PagePath {
    path: PathBuf,
    orig: PathBuf,
    template: String,
}

impl Object for DirPath {
    fn get_value(self: &Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        match key.as_str()? {
            "pages" => {
                let mut pages: Vec<PageInfo> = Vec::new();
                for entry in self.path.read_dir().ok()? {
                    let entry = entry.ok()?;
                    if entry.path().is_file()
                        && entry.path().file_name() != Some(OsStr::new(META_FILE))
                    {
                        let contents = fs::read_to_string(entry.path())
                            .inspect_err(|e| {
                                println!("Failed to read `{:?}` with `{}`", entry.path(), e)
                            })
                            .ok()?;
                        let meta = serde_yml::from_str(&contents)
                            .inspect_err(|err| {
                                println!("Failed to parse `{:?}` with `{}`", entry.path(), err)
                            })
                            .ok()?;
                        pages.push(meta);
                    }
                }
                Some(minijinja::Value::from_serialize(pages))
            }
            "dirs" => {
                let mut dirs: Vec<DirInfo> = Vec::new();
                for entry in self.path.read_dir().ok()? {
                    let entry = entry.ok()?;
                    if entry.path().is_dir() {
                        let meta_path = entry.path().join(META_FILE);
                        let contents = fs::read_to_string(&meta_path)
                            .inspect_err(|e| println!("Failed to read {:?} with {}", &meta_path, e))
                            .ok()?;
                        let meta = serde_yml::from_str(&contents)
                            .inspect_err(|e| {
                                println!("Failed to parse `{:?}` with `{}`", &meta_path, e)
                            })
                            .ok()?;
                        dirs.push(meta);
                    }
                }
                Some(minijinja::Value::from_serialize(dirs))
            }
            _ => None,
        }
    }
}

impl Object for MetaObject {
    fn get_value(self: &Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        match key.as_str()? {
            "root" => Some(minijinja::Value::from_object(self.root.clone())),
            "parent" => Some(minijinja::Value::from_object(self.parent.clone())),
            _ => {
                let meta = minijinja::Value::from_serialize(self.page.clone());
                meta.get_item(key).ok()
            }
        }
    }
}
