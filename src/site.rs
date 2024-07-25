use crate::{
    file_tree::{Dir, Page},
    header::Header,
    info::{DirInfo, Info, PageInfo},
    utils::{copy_dir, extract_work},
    ASSET_DIR, IN_DIR, OUT_DIR, TEMPLATE_DIR,
};
use std::{fs, path::PathBuf};

use minijinja::Environment;

#[derive(Debug)]
pub struct Site {
    home: Page,
    env: Environment<'static>,
    assets: PathBuf,
    top_level: Vec<Dir>,
}

impl Site {
    pub fn new() -> Self {
        let base_dir = PathBuf::from(IN_DIR);
        let mut home = base_dir.clone();
        home.push("home.md");
        let assets = PathBuf::from(ASSET_DIR);

        let templates = PathBuf::from(TEMPLATE_DIR);
        let mut env = Environment::new();
        for entry in templates.read_dir().expect("No templates directory") {
            let entry = entry.unwrap();
            let content = fs::read_to_string(entry.path()).unwrap();
            let name = entry
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            env.add_template_owned(name, content.clone())
                .expect(&format!("Failed to load template `{:?}`", entry.path()));
        }

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
            assets,
            top_level,
        }
    }

    fn prepare(self) -> SitePrepared {
        let assets = self.assets;
        let mut headers: Vec<Header> = vec![Header::from(&Info::index(&self.home))];
        headers.extend(self.top_level.iter().map(|d| Header::from(&Info::from(d))));

        let index: PageInfo = PageInfo::index(self.home);

        let mut dirs = Vec::new();
        let mut pages = Vec::new();
        for dir in &self.top_level {
            extract_work(dir, &mut dirs, &mut pages);
        }

        let files = Files {
            index,
            assets,
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
struct SitePrepared {
    headers: Vec<Header>,
    files: Files,
    env: Environment<'static>,
}

impl SitePrepared {
    fn generate(self) {
        self.files.generate(&self.headers, &self.env);
    }
}

#[derive(Debug)]
struct Files {
    index: PageInfo,
    assets: PathBuf,
    dirs: Vec<DirInfo>,
    pages: Vec<PageInfo>,
}

impl Files {
    fn generate(self, headers: &[Header], env: &Environment) {
        copy_dir(
            self.assets.clone(),
            PathBuf::from(OUT_DIR).join(self.assets),
        )
        .unwrap();

        self.index.generate(headers, env);

        for dir in self.dirs {
            dir.generate(headers, env);
        }

        for page in self.pages {
            page.generate(headers, env);
        }
    }
}
