use figment::{
    providers::{Env, Format, Toml},
    Error, Figment, Metadata, Provider,
};
use serde::{Deserialize, Serialize};

/// config for generating the site
/// includes the directories to target and the names of the default templates
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// the directory that all the markdown files are contained in
    content_dir: String,
    /// the static assets like css, javascript, or media
    asset_dir: String,
    /// the directory with the template files
    template_dir: String,
    /// name of the default page template
    page_template: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            content_dir: "content".into(),
            asset_dir: "assets".into(),
            template_dir: "templates".into(),
            page_template: "default".into(),
        }
    }
}

impl Config {
    pub fn figment() -> Figment {
        Figment::from(Config::default()).merge(Env::prefixed("SITDOWN_"))
    }

    pub fn from<T: Provider>(provider: T) -> Result<Self, Error> {
        Figment::from(provider).extract()
    }
}

impl Provider for Config {
    fn metadata(&self) -> figment::Metadata {
        Metadata::named("Sitdown config")
    }
    fn data(&self) -> Result<figment::value::Map<figment::Profile, figment::value::Dict>, Error> {
        figment::providers::Serialized::defaults(Config::default()).data()
    }
}
