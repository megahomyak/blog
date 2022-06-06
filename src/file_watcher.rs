use std::{path::PathBuf, rc::Rc, sync::{Arc, Mutex}};

use hotwatch::{blocking, Event};

use crate::articles::Articles;

pub trait WithFileName {
    fn get_file_name(&self) -> String;
}

impl WithFileName for PathBuf {
    fn get_file_name(&self) -> String {
        self.file_name().unwrap().to_string_lossy().to_string()
    }
}

pub fn watch(path: PathBuf, articles: Arc<Mutex<Articles>>) {
    let watcher = blocking::Hotwatch::new().unwrap();
    watcher.watch(path, |event| {
        match event {
            Event::Remove(path) => {
                articles.lock().unwrap().remove(Rc::new(path.get_file_name()));
            },
            Event::Rename(from, to) => {
                articles.lock().unwrap().rename(from.get_file_name(), Rc::new(to.get_file_name()));
            },
            Event::Write(path) | Event::Create(path) => {
                articles.lock().unwrap().update(path.get_file_name());
            },
            _ => (),
        };
        blocking::Flow::Continue
    });
}
