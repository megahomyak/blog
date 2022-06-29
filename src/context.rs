use std::{
    path::Path,
    sync::{
        mpsc::{self, Receiver, RecvError, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration, mem,
};

use actix_web::dev::ServerHandle;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};

use crate::{watch_articles, website::Website};

type Milliseconds = u64;

pub struct WatcherInfo<'watcher_info> {
    pub path: &'watcher_info Path,
    pub delay: Milliseconds,
    pub recursive_mode: RecursiveMode,
}

pub fn watch(
    path: &Path,
    delay: Milliseconds,
    recursive_mode: RecursiveMode,
    event_sender: &Sender<DebouncedEvent>,
) -> RecommendedWatcher {
    let mut watcher: RecommendedWatcher =
        Watcher::new(event_sender.clone(), Duration::from_millis(delay)).unwrap();
    watcher.watch(path, recursive_mode);
    watcher
}

pub struct WatcherContext {
    watcher: RecommendedWatcher,
    event_receiver: Receiver<DebouncedEvent>,
}

impl WatcherContext {
    pub fn new(watcher_info: &WatcherInfo) -> Self {
        let (event_sender, event_receiver) = mpsc::channel();
        let watcher: RecommendedWatcher =
            Watcher::new(event_sender, Duration::from_millis(watcher_info.delay)).unwrap();
        watcher.watch(watcher_info.path, watcher_info.recursive_mode);
        Self {
            watcher,
            event_receiver,
        }
    }

    pub fn update_from(&mut self, watcher_info: &WatcherInfo) {
        let watcher = Self::new(watcher_info);
        mem::replace(&mut self, &mut watcher);
    }

    pub fn receive_event(&self) -> Result<DebouncedEvent, RecvError> {
        self.event_receiver.recv()
    }
}

pub struct Watchers {
    pub articles_watcher_context: WatcherContext,
    pub config_watcher_context: WatcherContext,
}

pub struct Context<'watchers> {
    website: Arc<Mutex<Website>>,
    server_handle: ServerHandle,
    watchers: &'watchers mut Watchers,
}

impl<'watchers> Context<'watchers> {
    pub fn new(
        website: Arc<Mutex<Website>>,
        server_handle: ServerHandle,
        watchers: &'watchers mut Watchers,
    ) -> Self {
        Self {
            website,
            server_handle,
            watchers,
        }
    }

    pub async fn stop_server(&mut self) {
        self.server_handle.stop(true).await;
    }

    pub fn reload_articles_watcher(&mut self) {
        self.watchers.articles_watcher_context = watch_articles(self.website.clone());
    }
}
