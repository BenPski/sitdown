use std::path::PathBuf;

use minijinja::Environment;

use crate::{
    config::{Config, ConfigStructure},
    error::Result,
    templates,
    tree::{load_contents, Dir, DirInfo, PageInfo},
};

/// The app represents the state of the site to generate
pub struct App<'a> {
    structure: &'a ConfigStructure,
    // options: Options,
    // defaults: ConfigDefaults,
    templates: Environment<'a>,
    content: Dir<DirInfo, PageInfo>,
    assets: Dir<PathBuf, PathBuf>,
}

impl<'a> App<'a> {
    pub fn new(config: &'a Config) -> Self {
        let structure = &config.structure;
        let options = config.options.options();
        let defaults = &config.defaults;

        let templates = templates::get_env(&structure.template).unwrap();
        let content = load_contents(&structure.content)
            .unwrap()
            .annotate(&defaults, &options)
            .unwrap();
        let assets = load_contents(&structure.assets).unwrap();

        App {
            structure,
            // options,
            // defaults,
            templates,
            content,
            assets,
        }
    }

    fn copy_assets(&self) -> Result<()> {
        let res = self.assets.copy_to(&self.structure.site)?;
        Ok(res)
    }

    fn create_pages(&self) -> Result<()> {
        let meta_tree = self.content.write_metadata(&self.structure.work)?;
        println!("tree after writing metadata: {:?}", meta_tree);
        meta_tree.create(&self.structure.site, &self.templates)?;
        Ok(())
    }

    pub fn create(&self) -> Result<()> {
        self.copy_assets().and_then(|_| self.create_pages())
    }
}
