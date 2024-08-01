use std::{fs, path::PathBuf};

use minijinja::{Environment, Value};
use pulldown_cmark::Options;
use serde::{Deserialize, Serialize};

use crate::{config::ConfigDefaults, document::Document, files::Dir, metadata::Metadata, OUT_DIR};

/// a representation of the metadata in the filetree
/// that should be simple to access for the metadata in the rendering
///
/// in order to be convenient to access need to flatten the structure a bit
/// and that is why a directory and a file share the same structure

#[derive(Default, Debug, Serialize)]
pub struct FileMeta {
    #[serde(flatten)]
    pub page: Document,
    pub entries: Vec<FileMeta>,
}

impl FileMeta {
    pub fn from_dir(dir: &Dir, options: &Options, config: &ConfigDefaults) -> Self {
        let entries = dir
            .pages()
            .map(|p| Document::new(options, config, p))
            .map(|d| FileMeta::from(d))
            .chain(dir.dirs().map(|d| FileMeta::from_dir(d, options, config)))
            .collect();
        Self {
            page: Document::default(),
            entries,
        }
    }
    pub fn from_doc(doc: Document) -> Self {
        Self {
            page: doc,
            entries: Vec::new(),
        }
    }

    pub fn traverse<'a>(&'a self, templates: &Environment<'a>) {
        let mut meta = FullMeta {
            root: self,
            parent: self,
            page: &FileMeta::default(),
        };
        self.traverse_meta(templates, &mut meta);
    }

    fn traverse_meta<'a>(&'a self, templates: &Environment<'a>, meta: &mut FullMeta<'a>) {
        println!("processing: {:?}", self);
        if self.entries.is_empty() {
            // a page
            meta.page = self;
            let layout = self.page.metadata.layout.as_ref();
            let template = templates.get_template(&layout).unwrap();
            let content = template.render(Value::from_serialize(meta)).unwrap();
            let path = PathBuf::from(OUT_DIR).join(&self.page.metadata.location);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, content).unwrap();
        } else {
            // directory
            meta.parent = self;
            for entry in &self.entries {
                entry.traverse_meta(templates, meta);
            }
        }
    }
}

impl From<Document> for FileMeta {
    fn from(value: Document) -> Self {
        Self {
            page: value,
            entries: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize)]
struct FullMeta<'a> {
    root: &'a FileMeta,
    parent: &'a FileMeta,
    #[serde(flatten)]
    page: &'a FileMeta,
}
