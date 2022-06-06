use std::{sync::{Arc, Mutex}, collections::{HashMap, BTreeMap}, time::SystemTime};

struct FileWatcher {
    articles: Arc<Mutex<Articles>>
}

impl FileWatcher {
    pub fn new(directory: PathBuf) {

    }

    fn set()
}
