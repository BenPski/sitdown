use serde::Serialize;

use crate::info::Info;

#[derive(Debug, Serialize)]
pub struct Header {
    link: String,
    title: String,
}

impl From<&Info> for Header {
    fn from(value: &Info) -> Self {
        Self {
            link: value.link.clone(),
            title: value.title.clone(),
        }
    }
}
