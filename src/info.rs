use std::{fs, path::PathBuf};

use minijinja::{context, Environment};
use serde::Serialize;

use crate::{
    file_tree::{Dir, File, Page},
    header::Header,
    OUT_DIR,
};

// general info derived from the page/dirs path
#[derive(Debug)]
pub struct Info {
    pub title: String,
    pub link: String,
    pub save_path: PathBuf,
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
    pub fn index(_page: &Page) -> Self {
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
pub struct DirInfo {
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
    pub fn generate(self, headers: &[Header], env: &Environment) {
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
pub struct PageInfo {
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
    pub fn index(page: Page) -> Self {
        let info = Info::index(&page);
        let content = fs::read_to_string(page.path).unwrap();
        Self {
            title: info.title,
            content,
            save_path: info.save_path,
        }
    }

    pub fn generate(self, headers: &[Header], env: &Environment) {
        fs::create_dir_all(self.save_path.parent().unwrap()).unwrap();
        let template = env.get_template("content").unwrap();
        let content = markdown::to_html(&self.content);
        let contents = template
            .render(context! { headers => headers, title => self.title, content => content })
            .unwrap();
        fs::write(self.save_path, contents).unwrap();
    }
}
