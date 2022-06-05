use std::path::PathBuf;

pub enum Event {
    Changed { path: PathBuf },
    Created { path: PathBuf },
    Removed { path: PathBuf },
    Renamed { from: PathBuf, to: PathBuf },
}

pub trait FileWatcher {
    /// # Errors
    /// Returns `None` if an error happened while creating the watcher
    fn new(path: PathBuf) -> Option<Self> where Self: Sized;
    /// Runs the watcher with the callback until the first error
    fn run(&mut self, callback: impl Fn(Event) + 'static);
}
