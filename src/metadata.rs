use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

use crate::{config::ConfigDefaults, path::PathData};

/// the metadata associated with a page
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    layout: Option<String>,
    title: Option<String>,
    location: Option<PathBuf>,
    // technically making more assumptions than is strictly valid by putting the remaining yaml
    // into just a hashmap
    #[serde(flatten)]
    meta: HashMap<String, serde_yml::Value>,
}

/// metadata derived from the config and the filepath
pub struct DefaultMetadata {
    layout: String,
    title: String,
    location: PathBuf,
}

/// metadata that defines the template and parameters to give to the template
#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub layout: String,
    pub title: String,
    pub location: PathBuf,
    #[serde(flatten)]
    pub meta: HashMap<String, serde_yml::Value>,
}

impl DefaultMetadata {
    pub fn new(config: &ConfigDefaults, path: PathData) -> Self {
        Self {
            title: path.title,
            location: path.location,
            layout: config.page.clone(),
        }
    }
}

impl From<DefaultMetadata> for Metadata {
    fn from(value: DefaultMetadata) -> Self {
        Self {
            title: value.title,
            location: value.location,
            layout: value.layout,
            meta: HashMap::new(),
        }
    }
}

impl Metadata {
    pub fn new(def: DefaultMetadata, file: FileMetadata) -> Self {
        let title = file.title.unwrap_or(def.title);
        let location = file.location.unwrap_or(def.location);
        let layout = file.layout.unwrap_or(def.layout);
        Self {
            title,
            location,
            layout,
            meta: file.meta,
        }
    }
}
