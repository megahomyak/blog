use std::{
    fmt::Display,
    path::Path,
    sync::{Arc, Mutex},
};

use crate::articles::Articles;
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

pub trait FileWatcher<EventHandler>
where
    EventHandler: FnMut(&FileEvent) + 'static,
{
    fn new(event_handler: EventHandler) -> Self;
    fn watch(self, path: &Path) -> !;
}

pub trait FileNameShortcut {
    fn file_name_arc_str(&self) -> Arc<str>;
}

impl FileNameShortcut for Path {
    fn file_name_arc_str(&self) -> Arc<str> {
        self.file_name().unwrap().to_string_lossy().into()
    }
}

pub fn handle_event<AuthorName>(articles: &mut Arc<Mutex<Articles<AuthorName>>>, event: &FileEvent)
where
    AuthorName: Display,
{
    match event {
        FileEvent::Removed { path } => {
            articles
                .lock()
                .unwrap()
                .remove_article(&path.file_name_arc_str());
        }
        FileEvent::Renamed { from, to } => {
            articles
                .lock()
                .unwrap()
                .rename_article(&from.file_name_arc_str(), to.file_name_arc_str());
        }
        FileEvent::Changed { path } | FileEvent::Created { path } => {
            articles
                .lock()
                .unwrap()
                .update_article(&path.file_name_arc_str());
        }
    }
}
