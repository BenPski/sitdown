use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

use crate::{config::Config, path::PathData};

/// the metadata associated with a page
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub location: Option<PathBuf>,
    pub layout: Option<String>,
    // technically making more assumptions than is strictly valid by putting the remaining yaml
    // into just a hashmap
    #[serde(flatten)]
    pub meta: HashMap<String, serde_yml::Value>,
}

impl Metadata {
    fn title(mut self, title: impl Into<Option<String>>) -> Self {
        self.title = title.into();
        self
    }
    fn location(mut self, location: impl Into<Option<PathBuf>>) -> Self {
        self.location = location.into();
        self
    }
    fn layout(mut self, layout: impl Into<Option<String>>) -> Self {
        self.layout = layout.into();
        self
    }
    fn meta(mut self, meta: impl Into<HashMap<String, serde_yml::Value>>) -> Self {
        self.meta = meta.into();
        self
    }
    pub fn merge(mut self, other: impl Into<Metadata>) -> Self {
        let other = other.into();
        self.title = self.title.or(other.title);
        self.location = self.location.or(other.location);
        self.layout = self.layout.or(other.layout);
        self.meta.extend(other.meta);

        self
    }
}

impl From<PathData> for Metadata {
    fn from(value: PathData) -> Self {
        Self {
            title: value.title.into(),
            location: value.location.into(),
            ..Default::default()
        }
        // Self::default().title(value.title).location(value.location)
    }
}

impl From<&Config> for Metadata {
    fn from(value: &Config) -> Self {
        Self {
            layout: value.page_template.clone().into(),
            ..Default::default()
        }
    }
}
