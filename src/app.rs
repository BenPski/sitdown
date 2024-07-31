use figment::providers::{Format, Toml};
use minijinja::Environment;

use crate::{config::Config, files::Dir, templates, OUT_DIR};

/// The app represents the state of the site to generate

pub struct Info<'a> {
    config: Config,
    templates: Environment<'a>,
}

impl<'a> Info<'a> {
    fn new() -> Self {
        let config: Config = Config::figment()
            .merge(Toml::file("sitdown.toml"))
            .extract()
            .unwrap();
        let templates = templates::get_env(&config.template_dir).unwrap();
        println!("config: {:?}", config);
        println!("templates: {:?}", templates);
        Self { config, templates }
    }
}

pub struct App<'a> {
    info: Info<'a>,
    content: Dir,
    assets: Dir,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let info = Info::new();
        let content = Dir::new(&info.config.content_dir);
        let assets = Dir::new(&info.config.asset_dir);

        App {
            info,
            content,
            assets,
        }
    }

    fn copy_assets(&self) -> std::io::Result<()> {
        self.assets.copy_to(OUT_DIR)
    }

    fn create_pages(&self) -> std::io::Result<()> {
        let docs = self.content.documents(&self.info.config);
        for doc in docs {
            doc.create(&self.info.templates)?;
        }
        Ok(())
    }

    pub fn create(&self) -> std::io::Result<()> {
        self.copy_assets().and_then(|_| self.create_pages())
    }
}
