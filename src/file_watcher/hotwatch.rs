use std::path::Path;

use hotwatch::Hotwatch;

use super::{FileEvent, FileWatcher};

pub struct WatchGuard {
    _watcher: Hotwatch,
}

impl WatchGuard {
    pub const fn new(watcher: Hotwatch) -> Self {
        Self { _watcher: watcher }
    }
}

impl super::WatchGuard for WatchGuard {}

pub struct Watcher {}

impl<EventHandler, FilePath> FileWatcher<EventHandler, FilePath, WatchGuard> for Watcher
where
    EventHandler: FnMut(&FileEvent) + Send + 'static,
    FilePath: AsRef<Path> + Send,
{
    fn new(path: FilePath, mut event_handler: EventHandler) -> WatchGuard {
        let mut watcher = Hotwatch::new().unwrap();
        watcher
            .watch(&path, move |event| {
                match event {
                    hotwatch::Event::Write(path) => {
                        event_handler(&FileEvent::Changed { path: &path });
                    }
                    hotwatch::Event::Create(path) => {
                        event_handler(&FileEvent::Created { path: &path });
                    }
                    hotwatch::Event::Remove(path) => {
                        event_handler(&FileEvent::Removed { path: &path });
                    }
                    hotwatch::Event::Rename(from, to) => {
                        event_handler(&FileEvent::Renamed {
                            from: &from,
                            to: &to,
                        });
                    }
                    _ => (),
                };
            })
            .unwrap();
        WatchGuard::new(watcher)
    }
}
