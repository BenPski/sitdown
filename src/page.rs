use core::panic;
use std::{fs, path::Path};

use pulldown_cmark::{MetadataBlockKind, Options, Parser, TextMergeStream};
use yaml_rust2::{Yaml, YamlLoader};

/// the page data, both the metadata and the contents of the file
struct Page {
    metadata: Yaml,
    contents: String,
}

impl Page {
    fn new<T: AsRef<Path>>(path: T, options: Options) -> Self {
        let text = fs::read_to_string(path.as_ref()).unwrap();
        let parser = Parser::new_ext(&text, options);

        let iterator = TextMergeStream::new(parser).peekable();

        let meta = if let Some(Start(MetadataBlockKind(_))) = iterator.peek() {
            iterator.next();
            let info = if let Some(Text(Borrowed(s))) = iterator.next() {
                let docs = YamlLoader::load_from_str(s).unwrap();
                let res = docs[0];
                iterator.next();
                res
            } else {
                panic!("Missing yaml from metadata block");
            };
            info
        } else {
            Yaml::Null
        };

        let mut contents = String::new();
        pulldown_cmark::html::push_html(&mut contents, iterator);

        Self {
            metadata: meta,
            contents,
        }
    }
}
