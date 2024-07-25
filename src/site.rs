//
// There are a few different types of files that need to be handled
// simplest is the supporting content, things like css, javascript, and media
// these should just be copied verbatim to the site/ directory
//
// next is the markdown files, these should be formatted with the `content` template after
// converting the markdown into html. These then get copied into the site/ directory at the
// equivalent location from the content. eg, content/dir/stuff.md -> site/dir/stuff.html
//
// next is the directories, these should generate an index.html using the `entries` template. This
// should provide links out to the files contained in the directories. content/dir/ ->
// site/dir/index.html
//
// the final odd ball file is the home.md file that should generate index.html. should be
// content/home.md -> site/index.html, for now this should be a regular `content` file.
//
// the links to be used in the files should all treat the site/ directory as the root, so something
// like content/dir/stuff.md -> /dir/stuff.html, content/dir/ -> /dir/index.html
//
// for extracting headers need to include home and all the top level directories. or can be
// hardcoded in the layout template
//
//
//
//
use std::path::Path;
use std::{fs, io, path::PathBuf};

use minijinja::{context, Environment};
use serde::Serialize;

const IN_DIR: &str = "content";
const OUT_DIR: &str = "site";
const CSS_DIR: &str = "css";
const MEDIA_DIR: &str = "media";
const SCRIPTS_DIR: &str = "scripts";

#[derive(Debug)]
pub struct Site {
    home: Page,
    env: Environment<'static>,
    css: PathBuf,
    scripts: PathBuf,
    media: PathBuf,
    top_level: Vec<Dir>,
}

impl Site {
    pub fn new() -> Self {
        let base_dir = PathBuf::from(IN_DIR);
        let mut home = base_dir.clone();
        home.push("home.md");
        let css = PathBuf::from(CSS_DIR);
        let scripts = PathBuf::from(SCRIPTS_DIR);
        let media = PathBuf::from(MEDIA_DIR);

        let mut env = Environment::new();
        env.add_template("layout", include_str!("../templates/layout.jinja"))
            .expect("No layout template, `templates/layout.jinja`");
        env.add_template("home", include_str!("../templates/home.jinja"))
            .expect("No home template, `templates/home.jinja`");
        env.add_template("about", include_str!("../templates/about.jinja"))
            .expect("No about template, `templates/about.jinja");
        env.add_template("entries", include_str!("../templates/entries.jinja"))
            .expect("No entries template, `templates/entries.jinja");
        env.add_template("content", include_str!("../templates/content.jinja"))
            .expect("No content template, `templates/content.jinja");

        let top_level = base_dir
            .read_dir()
            .expect("Failed to read dir")
            .filter_map(|entry| {
                if let Ok(entry) = entry {
                    Dir::from_path(entry.path())
                } else {
                    None
                }
            })
            .collect();

        Site {
            home: Page::from_path(home).expect("No home.md"),
            env,
            css,
            scripts,
            media,
            top_level,
        }
    }

    pub fn prepare(self) -> SitePrepared {
        let support = vec![self.css, self.scripts, self.media];
        let mut headers: Vec<Header> = vec![Header::from(&Info::index(&self.home))];
        headers.extend(self.top_level.iter().map(|d| Header::from(&Info::from(d))));

        let index: PageInfo = PageInfo::index(self.home);

        let mut dirs = Vec::new();
        let mut pages = Vec::new();
        for dir in &self.top_level {
            dir.extract_work(&mut dirs, &mut pages);
        }

        let files = Files {
            index,
            support,
            dirs,
            pages,
        };
        SitePrepared {
            headers,
            files,
            env: self.env,
        }
    }

    pub fn run(self) {
        self.prepare().generate()
    }
}

#[derive(Debug)]
pub struct SitePrepared {
    headers: Vec<Header>,
    files: Files,
    env: Environment<'static>,
}

impl SitePrepared {
    pub fn generate(self) {
        self.files.generate(&self.headers, &self.env);
    }
}

#[derive(Debug)]
struct Files {
    index: PageInfo,
    support: Vec<PathBuf>,
    dirs: Vec<DirInfo>,
    pages: Vec<PageInfo>,
}

impl Files {
    fn generate(self, headers: &[Header], env: &Environment) {
        for f in &self.support {
            copy_dir(f, PathBuf::from(OUT_DIR).join(f)).unwrap();
        }

        self.index.generate(headers, env);

        for dir in self.dirs {
            dir.generate(headers, env);
        }

        for page in self.pages {
            page.generate(headers, env);
        }
    }
}

#[derive(Debug, Clone)]
enum File {
    Dir(Dir),
    Page(Page),
}

impl File {
    fn from_path(path: PathBuf) -> Option<Self> {
        if path.is_dir() {
            Dir::from_path(path).map(File::Dir)
        } else {
            Page::from_path(path).map(File::Page)
        }
    }
}
#[derive(Debug, Clone)]
struct Dir {
    path: PathBuf,
    children: Vec<File>,
}

fn copy_dir(from: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
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

impl Dir {
    fn from_path(path: PathBuf) -> Option<Self> {
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

    fn extract_work(&self, dir_info: &mut Vec<DirInfo>, page_info: &mut Vec<PageInfo>) {
        dir_info.push(DirInfo::from(self));
        for child in &self.children {
            match child {
                File::Dir(d) => {
                    d.extract_work(dir_info, page_info);
                }
                File::Page(p) => {
                    page_info.push(PageInfo::from(p));
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct Page {
    path: PathBuf,
}

impl Page {
    fn from_path(path: PathBuf) -> Option<Self> {
        if !path.is_dir() {
            Some(Self { path })
        } else {
            None
        }
    }
}

// general info derived from the page/dirs path
#[derive(Debug)]
struct Info {
    title: String,
    link: String,
    save_path: PathBuf,
}

impl From<&Page> for Info {
    fn from(value: &Page) -> Self {
        let title = value
            .path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .replace("_", " ");
        let mut link = PathBuf::from("/");
        let mut save_path = PathBuf::from(OUT_DIR);
        for item in value.path.components().skip(1) {
            link.push(item);
            save_path.push(item);
        }
        link.set_extension("html");
        save_path.set_extension("html");

        Self {
            title,
            link: link.to_str().unwrap().to_string(),
            save_path,
        }
    }
}

impl From<&Dir> for Info {
    fn from(value: &Dir) -> Self {
        let title = value
            .path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .replace("_", " ");
        let mut link = PathBuf::from("/");
        let mut save_path = PathBuf::from(OUT_DIR);
        for item in value.path.components().skip(1) {
            link.push(item);
            save_path.push(item);
        }
        link.push("index.html");
        save_path.push("index.html");

        Self {
            title,
            link: link.to_str().unwrap().to_string(),
            save_path,
        }
    }
}

impl Info {
    fn index(_page: &Page) -> Self {
        Self {
            title: "Home".into(),
            link: "/index.html".into(),
            save_path: PathBuf::from(OUT_DIR).join("index.html"),
        }
    }
}

// pair DirInfo and PageInfo with the template and any other supporting info to generate the page
// TODO: include a reference to headers in the info so then everything can just be serializing the
// structs
#[derive(Debug, Serialize)]
struct DirInfo {
    title: String,
    entries: Vec<LinkTree>,
    save_path: PathBuf,
}

impl From<&Dir> for DirInfo {
    fn from(value: &Dir) -> Self {
        let info = Info::from(value);
        let entries = value.children.iter().map(|c| c.into()).collect();
        Self {
            title: info.title,
            entries,
            save_path: info.save_path,
        }
    }
}

impl DirInfo {
    fn generate(self, headers: &[Header], env: &Environment) {
        fs::create_dir_all(self.save_path.parent().unwrap()).unwrap();
        let template = env.get_template("entries").unwrap();
        let contents = template
            .render(context! { headers => headers, title => self.title, entries => self.entries})
            .unwrap();
        fs::write(self.save_path, contents).unwrap();
    }
}

#[derive(Debug, Serialize)]
struct LinkTree {
    link: String,
    title: String,
    children: Vec<LinkTree>,
}

impl From<&File> for LinkTree {
    fn from(value: &File) -> Self {
        match value {
            File::Page(p) => p.into(),
            File::Dir(d) => d.into(),
        }
    }
}

impl From<&Page> for LinkTree {
    fn from(value: &Page) -> Self {
        let info = Info::from(value);
        Self {
            link: info.link,
            title: info.title,
            children: Vec::new(),
        }
    }
}

impl From<&Dir> for LinkTree {
    fn from(value: &Dir) -> Self {
        let info = Info::from(value);
        Self {
            link: info.link,
            title: info.title,
            children: value.children.iter().map(|c| c.into()).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct PageInfo {
    title: String,
    content: String,
    save_path: PathBuf,
}

impl From<&Page> for PageInfo {
    fn from(value: &Page) -> Self {
        let info = Info::from(value);
        let content = fs::read_to_string(value.path.clone()).unwrap();
        Self {
            title: info.title,
            content,
            save_path: info.save_path,
        }
    }
}

impl PageInfo {
    fn index(page: Page) -> Self {
        let info = Info::index(&page);
        let content = fs::read_to_string(page.path).unwrap();
        Self {
            title: info.title,
            content,
            save_path: info.save_path,
        }
    }

    fn generate(self, headers: &[Header], env: &Environment) {
        fs::create_dir_all(self.save_path.parent().unwrap()).unwrap();
        let template = env.get_template("content").unwrap();
        let content = markdown::to_html(&self.content);
        let contents = template
            .render(context! { headers => headers, title => self.title, content => content })
            .unwrap();
        fs::write(self.save_path, contents).unwrap();
    }
}

#[derive(Debug, Serialize)]
struct Header {
    link: String,
    title: String,
}

impl From<&Info> for Header {
    fn from(value: &Info) -> Self {
        Self {
            link: value.link.clone(),
            title: value.title.clone(),
        }
    }
}
