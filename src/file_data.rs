use std::{collections::HashMap, path::PathBuf};

use crate::{config::Config, document::Document, metadata, path::PathData};

/// the data associated with a file

struct PageInfo {
    layout: String,
    title: String,
    location: PathBuf,
    meta: HashMap<String, serde_yml::Value>,
    content: String,
}

struct DirInfo {}
