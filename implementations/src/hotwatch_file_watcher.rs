use std::path::PathBuf;

use abstractions::file_watcher::{self, FileWatcher};
use hotwatch::blocking;

pub struct HotwatchFileWatcher {
    hotwatch: blocking::Hotwatch,
    path: PathBuf,
}

impl FileWatcher for HotwatchFileWatcher {
    fn new(path: PathBuf) -> Option<Self> {
        let hotwatch = match blocking::Hotwatch::new() {
            Ok(hotwatch) => hotwatch,
            Err(_error) => return None,
        };
        Some(Self { hotwatch, path })
    }

    fn run(&mut self, callback: impl Fn(file_watcher::Event) + 'static) {
        let _watching_result = self.hotwatch.watch(&self.path, move |event| {
            callback(match event {
                hotwatch::Event::Rename(from, to) => file_watcher::Event::Renamed { from, to },
                hotwatch::Event::Write(path) => file_watcher::Event::Changed { path },
                hotwatch::Event::Create(path) => file_watcher::Event::Created { path },
                hotwatch::Event::Error(_error, _optional_path) => return blocking::Flow::Exit,
                hotwatch::Event::Remove(path) => file_watcher::Event::Removed { path },
                _ => return blocking::Flow::Continue,
            });
            blocking::Flow::Continue
        });
    }
}
