use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Example {
    title: Option<String>,
    date: Option<String>,
    #[serde(flatten)]
    meta: HashMap<String, String>,
}

fn main() {
    let yaml = r#"
    name: Something
    date: 2020-01-01
    "#;

    let example: Example = serde_yml::from_str(&yaml).unwrap();
    println!("{example:?}");

    println!("{:?}", serde_yml::to_string(&example));
}
