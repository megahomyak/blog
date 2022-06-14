use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use crate::context::Context;
pub mod hotwatch;

pub enum FileEvent<'event> {
    Changed {
        path: &'event Path,
    },
    Removed {
        path: &'event Path,
    },
    Renamed {
        from: &'event Path,
        to: &'event Path,
    },
    Created {
        path: &'event Path,
    },
}

/// When dropped, must stop the running watcher thread
pub trait WatchGuard<EventHandler, FilePath, Watcher: FileWatcher<EventHandler, FilePath, Self>> {
    /// Starts the new watcher of the same type
    fn new(self, path: FilePath, event_handler: EventHandler) -> Self {
        Watcher::new(path, event_handler)
    }
}

pub trait FileWatcher<EventHandler, FilePath, WatchGuard>
where
    EventHandler: FnMut(&FileEvent) + Send + 'static,
    FilePath: AsRef<Path>,
    WatchGuard: self::WatchGuard<EventHandler, FilePath, Self>,
{
    /// Runs the watcher in a new thread, returning a handle to stop it
    fn new(path: FilePath, event_handler: EventHandler) -> WatchGuard;
}

pub trait FileNameShortcut {
    fn file_name_arc_str(&self) -> Arc<str>;
}

impl FileNameShortcut for Path {
    fn file_name_arc_str(&self) -> Arc<str> {
        self.file_name().unwrap().to_string_lossy().into()
    }
}

pub fn handle_event<ArticlesWatchGuard, ConfigWatchGuard>(
    context: &mut Arc<Mutex<Context<ArticlesWatchGuard, ConfigWatchGuard>>>,
    event: &FileEvent,
) where
    ArticlesWatchGuard: WatchGuard,
{
    match event {
        FileEvent::Removed { path } => {
            context
                .lock()
                .unwrap()
                .remove_article(&path.file_name_arc_str());
        }
        FileEvent::Renamed { from, to } => {
            context
                .lock()
                .unwrap()
                .rename_article(&from.file_name_arc_str(), to.file_name_arc_str());
        }
        FileEvent::Changed { path } | FileEvent::Created { path } => {
            context
                .lock()
                .unwrap()
                .update_article(&path.file_name_arc_str());
        }
    }
}
