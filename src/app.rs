use std::path::PathBuf;

use figment::providers::{Format, Toml};
use minijinja::Environment;
use pulldown_cmark::Options;

use crate::{
    config::{Config, ConfigDefaults, ConfigStructure},
    templates,
    tree::{load_contents, Dir, DirInfo, PageInfo},
    OUT_DIR, WORK_DIR,
};

/// The app represents the state of the site to generate
pub struct App<'a> {
    structure: ConfigStructure,
    options: Options,
    defaults: ConfigDefaults,
    templates: Environment<'a>,
    content: Dir<DirInfo, PageInfo>,
    assets: Dir<PathBuf, PathBuf>,
}

impl App<'static> {
    pub fn new() -> Self {
        let config: Config = Config::figment()
            .merge(Toml::file("sitdown.yaml"))
            .extract()
            .unwrap();
        let structure = config.structure;
        let options = config.options.options();
        let defaults = config.defaults;

        let templates = templates::get_env(&structure.template).unwrap();
        let content = load_contents(&structure.content)
            .unwrap()
            .annotate(&defaults, &options)
            .unwrap();
        let assets = load_contents(&structure.assets).unwrap();

        App {
            structure,
            options,
            defaults,
            templates,
            content,
            assets,
        }
    }

    fn copy_assets(&self) -> std::io::Result<()> {
        self.assets.copy_to(OUT_DIR)
    }

    fn create_pages(&self) -> std::io::Result<()> {
        let meta_tree = self.content.write_metadata(WORK_DIR);
        println!("tree after writing metadata: {:?}", meta_tree);
        meta_tree.create(OUT_DIR, &self.templates);
        Ok(())
    }

    pub fn create(&self) -> std::io::Result<()> {
        self.copy_assets().and_then(|_| self.create_pages())
    }
}
