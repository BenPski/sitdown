use figment::{Error, Figment, Metadata, Provider};
use pulldown_cmark::Options;
use serde::{Deserialize, Serialize};

/// default directory values
pub const IN_DIR: &str = "content";
pub const OUT_DIR: &str = "_site";
pub const ASSET_DIR: &str = "assets";
pub const TEMPLATE_DIR: &str = "templates";
pub const WORK_DIR: &str = "_work";

/// config for managing the site
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub structure: ConfigStructure,
    pub options: ConfigOptions,
    pub defaults: ConfigDefaults,
}

/// config for defining the layout of the site
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigStructure {
    /// the directory that hold all the markdown files
    pub content: String,
    /// static files that don't need processing like css, js, and media
    pub assets: String,
    /// the jinja templates
    pub template: String,
    /// the dir that metadata gets written to and read from
    pub work: String,
    /// the output directory that is used for serving the pages
    pub site: String,
}

/// config defining the defaults to be used in the site generation
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigDefaults {
    /// the default template for a page
    pub page: String,
}

/// config options for the markdown parsing
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigOptions {
    /// enable math mode
    pub math: bool,
}

impl Default for ConfigStructure {
    fn default() -> Self {
        Self {
            content: IN_DIR.into(),
            assets: ASSET_DIR.into(),
            template: TEMPLATE_DIR.into(),
            work: WORK_DIR.into(),
            site: OUT_DIR.into(),
        }
    }
}

impl Default for ConfigDefaults {
    fn default() -> Self {
        Self {
            page: "content".into(),
        }
    }
}

impl Default for ConfigOptions {
    fn default() -> Self {
        Self { math: true }
    }
}

impl Config {
    pub fn figment() -> Figment {
        Figment::from(Self::default())
    }
    pub fn from<T: Provider>(provider: T) -> Result<Self, Error> {
        Figment::from(provider).extract()
    }
}

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("Sitdown config")
    }
    fn data(&self) -> Result<figment::value::Map<figment::Profile, figment::value::Dict>, Error> {
        figment::providers::Serialized::defaults(Self::default()).data()
    }
}

// impl ConfigStructure {
//     fn figment() -> Figment {
//         Figment::from(Self::default())
//     }
//
//     fn from<T: Provider>(provider: T) -> Result<Self, Error> {
//         Figment::from(provider).extract()
//     }
// }

impl Provider for ConfigStructure {
    fn metadata(&self) -> figment::Metadata {
        Metadata::named("Sitdown file structure")
    }
    fn data(&self) -> Result<figment::value::Map<figment::Profile, figment::value::Dict>, Error> {
        figment::providers::Serialized::defaults(Self::default()).data()
    }
}

// impl ConfigDefaults {
//     fn figment() -> Figment {
//         Figment::from(Self::default())
//     }
//
//     fn from<T: Provider>(provider: T) -> Result<Self, Error> {
//         Figment::from(provider).extract()
//     }
// }

impl Provider for ConfigDefaults {
    fn metadata(&self) -> figment::Metadata {
        Metadata::named("Sitdown defaults")
    }
    fn data(&self) -> Result<figment::value::Map<figment::Profile, figment::value::Dict>, Error> {
        figment::providers::Serialized::defaults(Self::default()).data()
    }
}

impl ConfigOptions {
    // fn figment() -> Figment {
    //     Figment::from(Self::default())
    // }
    // fn from<T: Provider>(provider: T) -> Result<Self, Error> {
    //     Figment::from(provider).extract()
    // }
    pub fn options(&self) -> Options {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        if self.math {
            options.insert(Options::ENABLE_MATH);
        }
        options
    }
}

impl Provider for ConfigOptions {
    fn metadata(&self) -> Metadata {
        Metadata::named("Sitdown parser options")
    }
    fn data(&self) -> Result<figment::value::Map<figment::Profile, figment::value::Dict>, Error> {
        figment::providers::Serialized::defaults(Self::default()).data()
    }
}
