use std::{collections::HashMap, path::PathBuf};

pub struct CCacheItem {
    pub src_path: PathBuf,
    pub context_hash: String,
    pub deps: HashMap<PathBuf, String>,
    pub qstrs: Vec<String>,
    pub moduledefs: Vec<String>,
    pub root_pointers: Vec<String>,
}
