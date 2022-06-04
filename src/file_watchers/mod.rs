use std::path::Path;

use crate::types::TraitFuture;

enum FileEventKind {
    Changed,
    Created,
    Removed,
}

struct FileEvent<'file_path> {
    kind: FileEventKind,
    file_path: &'file_path Path,
}

impl<'file_event> FileEvent<'file_event> {
    pub const fn new(file_path: &'file_event Path, kind: FileEventKind) -> Self {
        Self { kind, file_path }
    }

    pub const fn file_path(&self) -> &'file_event Path {
        self.file_path
    }

    pub const fn kind(&'file_event self) -> &'file_event FileEventKind {
        &self.kind
    }
}

pub trait FileWatcher {
    fn next_event(&self) -> TraitFuture<FileEvent>;
}
