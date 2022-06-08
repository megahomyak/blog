use std::path::Path;

use hotwatch::blocking;

use super::{FileEvent, FileWatcher};

#[allow(clippy::module_name_repetitions)]
pub struct HotwatchFileWatcher<EventHandler> {
    watcher: blocking::Hotwatch,
    event_handler: EventHandler,
}

impl<EventHandler> FileWatcher<EventHandler> for HotwatchFileWatcher<EventHandler>
where
    EventHandler: FnMut(&FileEvent) + 'static,
{
    fn new(event_handler: EventHandler) -> Self {
        Self {
            event_handler,
            watcher: blocking::Hotwatch::new().unwrap(),
        }
    }

    fn watch(mut self, path: &Path) -> ! {
        self.watcher
            .watch(path, move |event| {
                match event {
                    hotwatch::Event::Write(path) => {
                        (self.event_handler)(&FileEvent::Changed { path: &path });
                    }
                    hotwatch::Event::Create(path) => {
                        (self.event_handler)(&FileEvent::Created { path: &path });
                    }
                    hotwatch::Event::Remove(path) => {
                        (self.event_handler)(&FileEvent::Removed { path: &path });
                    }
                    hotwatch::Event::Rename(from, to) => {
                        (self.event_handler)(&FileEvent::Renamed {
                            from: &from,
                            to: &to,
                        });
                    }
                    _ => (),
                };
                blocking::Flow::Continue
            })
            .unwrap();
        loop {
            self.watcher.run();
        }
    }
}
