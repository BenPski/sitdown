use std::{fs, io, path::Path};

use minijinja::Environment;

/// the templates associated with the site

pub fn get_env<'a, T: AsRef<Path>>(template_dir: T) -> io::Result<Environment<'a>> {
    let mut env = Environment::new();
    for entry in template_dir.as_ref().read_dir()? {
        let entry = entry?;
        if entry.path().is_file() {
            let name = entry
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            env.add_template_owned(name, fs::read_to_string(entry.path())?)
                .expect("Unable to read template");
        } else {
            println!("Skipping `{:?}` while traversing templates", entry.path());
        }
    }
    Ok(env)
}
