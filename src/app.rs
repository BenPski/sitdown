use figment::providers::{Format, Toml};
use minijinja::Environment;
use pulldown_cmark::Options;

use crate::{
    config::{Config, ConfigDefaults, ConfigOptions, ConfigStructure},
    files::Dir,
    templates, OUT_DIR,
};

/// The app represents the state of the site to generate
pub struct App<'a> {
    structure: ConfigStructure,
    options: Options,
    defaults: ConfigDefaults,
    templates: Environment<'a>,
    content: Dir,
    assets: Dir,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let config: Config = Config::figment()
            .merge(Toml::file("sitdown.yaml"))
            .extract()
            .unwrap();
        let structure = config.structure;
        let options = config.options.options();
        let defaults = config.defaults;

        let templates = templates::get_env(&structure.template).unwrap();
        let content = Dir::new(&structure.content);
        let assets = Dir::new(&structure.assest);

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
        let docs = self.content.documents(&self.options, &self.defaults);
        for doc in docs {
            doc.create(&self.templates)?;
        }
        Ok(())
    }

    pub fn create(&self) -> std::io::Result<()> {
        self.copy_assets().and_then(|_| self.create_pages())
    }
}
