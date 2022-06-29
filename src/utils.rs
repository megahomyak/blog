use std::{sync::Arc, path::Path};

pub trait FileNameShortcut {
    fn file_name_arc_str(&self) -> Arc<str>;
}

impl FileNameShortcut for Path {
    fn file_name_arc_str(&self) -> Arc<str> {
        self.file_name().unwrap().to_string_lossy().into()
    }
}
