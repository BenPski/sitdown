use core::panic;
use minijinja::{value::Object, Environment, Value};
use pulldown_cmark::{
    CowStr::Borrowed,
    Event::{Start, Text},
    Options, Parser, Tag, TextMergeStream,
};
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    config::ConfigDefaults,
    metadata::{DefaultMetadata, FileMetadata, Metadata},
    path::PathData,
    OUT_DIR,
};

/// the page data, both the metadata and the contents of the file
#[derive(Debug, Serialize)]
pub struct Document {
    #[serde(flatten)]
    pub metadata: Metadata,
    pub contents: String,
}

impl Object for &Document {
    fn get_value(self: &std::sync::Arc<Self>, key: &minijinja::Value) -> Option<minijinja::Value> {
        match key.as_str()? {
            "title" => Some(Value::from(self.metadata.title.clone())),
            "location" => self.metadata.location.to_str().map(|s| Value::from(s)),
            "content" => Some(Value::from(self.contents.clone())),
            s => {
                if let Some(v) = self.metadata.meta.get(s) {
                    Value::from_serialize(v).get_item(key).ok()
                } else {
                    None
                }
            }
        }
    }
}

impl Document {
    pub fn new<T: AsRef<Path>>(options: &Options, config: &ConfigDefaults, path: T) -> Self {
        let path_data = PathData::from(path.as_ref());
        let def_metadata = DefaultMetadata::new(config, path_data);
        let text = fs::read_to_string(path.as_ref()).unwrap();
        let parser = Parser::new_ext(&text, *options);

        let mut iterator = TextMergeStream::new(parser).peekable();

        let meta = if let Some(Start(Tag::MetadataBlock(_))) = iterator.peek() {
            iterator.next();
            let info = if let Some(Text(Borrowed(s))) = iterator.next() {
                let res: FileMetadata = serde_yml::from_str(s).unwrap();
                println!("parsed meta: {:?}", res);
                iterator.next();
                Metadata::new(def_metadata, res)
            } else {
                panic!("Missing yaml from metadata block");
            };
            info
        } else {
            def_metadata.into()
        };

        println!("final meta: {:?}", meta);

        let mut contents = String::new();
        pulldown_cmark::html::push_html(&mut contents, iterator);

        Self {
            metadata: meta,
            contents,
        }
    }

    pub fn create<'a>(&self, templates: &Environment<'a>) -> std::io::Result<()> {
        let layout = self.metadata.layout.as_ref();
        println!("layout: {layout:?}");
        let template = templates.get_template(&layout).unwrap();
        println!("values being passed: {:?}", Value::from_serialize(self));
        let content = template.render(Value::from_serialize(self)).unwrap();
        let path = PathBuf::from(OUT_DIR).join(&self.metadata.location);
        println!("Creating: {path:?}");
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, content)?;
        Ok(())
    }
}
