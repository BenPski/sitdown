use std::{fs, path::PathBuf};

use minijinja::Environment;
use pulldown_cmark::Options;

use crate::{
    config::{Config, ConfigDefaults, ConfigStructure},
    error::Result,
    templates,
    tree::{load_contents, Dir},
};

/// The app represents the state of the site to generate
pub struct App<'a> {
    structure: &'a ConfigStructure,
    options: Options,
    defaults: &'a ConfigDefaults,
    templates: Environment<'a>,
    content: Dir<PathBuf, PathBuf>,
    assets: Dir<PathBuf, PathBuf>,
}

impl<'a> App<'a> {
    pub fn new(config: &'a Config) -> Self {
        let structure = &config.structure;
        let options = config.options.options();
        let defaults = &config.defaults;

        let templates = templates::get_env(&structure.template).unwrap();
        let content = load_contents(&structure.content).unwrap();
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

    fn clear_dirs(&self) -> Result<()> {
        fs::remove_dir_all(&self.structure.site)
            .and_then(|_| fs::remove_dir_all(&self.structure.work))?;
        Ok(())
    }

    fn copy_assets(&self) -> Result<()> {
        let res = self.assets.copy_to(&self.structure.site)?;
        Ok(res)
    }

    fn create_pages(&self) -> Result<()> {
        let parsed_tree = self.content.annotate(&self.defaults, &self.options)?;
        let meta_tree = parsed_tree.write_metadata(&self.structure.work)?;
        meta_tree.create(&self.structure.site, &self.templates)?;
        Ok(())
    }

    pub fn create(&self) -> Result<()> {
        self.clear_dirs()
            .and_then(|_| self.copy_assets().and_then(|_| self.create_pages()))
    }
}
