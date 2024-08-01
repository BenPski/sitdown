use minijinja::value::Object;

use crate::files::Dir;

/// the metadata references for higher up the document tree, includes the root
/// of the filesystem and the immediate parent of the page

#[derive(Debug)]
pub struct DirMeta<'a> {
    pub root: &'a Dir,
    pub parent: &'a Dir,
}

impl<'a> Object for DirMeta<'a> {}
