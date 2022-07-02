use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::Duration,
};

use actix_web::{
    dev::{Server, ServerHandle},
    web, App, HttpServer,
};
use clap::{crate_description, Parser, Subcommand};
use config::Config;
use log::{error, warn};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode};
use utils::{set_global_log_level, FileNameShortcut};
use website::Website;

mod absolute_path;
mod config;
mod page_colors;
mod page_compilers;
mod routes;
mod utils;
mod website;

#[derive(Parser)]
#[clap(author, version, about = crate_description!(), long_about=None)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a sample configuration file with all the values pre-filled
    CreateSampleConfig {
        /// Create the configuration file even if it already exists
        #[clap(long, takes_value = false)]
        force: bool,
    },
    /// Run the server
    Run,
}

const CONFIG_FILE_NAME: &str = "config.json";

fn run_server(website: &Arc<Mutex<Website>>) -> Server {
    let mut config_guard = website.lock().unwrap();
    let config = &config_guard.config();
    HttpServer::new({
        let website = website.clone();
        move || {
            App::new()
                .app_data(web::Data::from(website.clone()))
                .route("/", web::get().to(routes::index))
                .route("/{filename}", web::get().to(routes::file))
        }
    })
    .bind((&config.host_name[..], config.port))
    .unwrap()
    .run()
}

pub struct WatchContext<Watcher> {
    _watcher: Watcher,
    event_receiver: mpsc::Receiver<DebouncedEvent>,
}

pub type WatchResult<Watcher> = Result<WatchContext<Watcher>, notify::Error>;

/// # Panics
/// Panics when there were some issues with the `mio`'s `EventLoop`. Don't want to handle those, to
/// be honest. It is too hard for them to occur anyway, but when they will occur, I don't want to
/// do anything, because __the main point__ of this program is to listen (or "watch") to file
/// changes.
#[allow(clippy::missing_errors_doc)]
pub fn watch<Watcher>(
    config: &Config,
    path: &Path,
    recursive_mode: RecursiveMode,
    watcher_maker: fn(RecommendedWatcher) -> Watcher,
) -> WatchResult<Watcher> {
    let (event_sender, event_receiver) = mpsc::channel();
    let mut watcher: RecommendedWatcher = notify::Watcher::new(
        event_sender,
        Duration::from_millis(config.file_watcher_delay_in_milliseconds),
    )
    .unwrap();
    notify::Watcher::watch(&mut watcher, path, recursive_mode)?;
    Ok(WatchContext {
        _watcher: watcher_maker(watcher),
        event_receiver,
    })
}

pub struct ArticlesWatcher(pub RecommendedWatcher);

#[allow(clippy::missing_errors_doc)]
pub fn watch_articles(config: &Config) -> WatchResult<ArticlesWatcher> {
    if config.articles_directory.as_ref().is_dir() {
        watch(
            config,
            config.articles_directory.as_ref(),
            RecursiveMode::Recursive,
            ArticlesWatcher,
        )
    } else {
        Err(notify::Error::Generic(format!(
            "{:?} is not a directory!",
            config.articles_directory.as_ref()
        )))
    }
}

pub struct ConfigWatcher(pub RecommendedWatcher);

#[allow(clippy::missing_errors_doc)]
pub fn watch_config(config: &Config) -> WatchResult<ConfigWatcher> {
    if Path::new(CONFIG_FILE_NAME).is_file() {
        watch(
            config,
            Path::new(CONFIG_FILE_NAME),
            RecursiveMode::NonRecursive,
            ConfigWatcher,
        )
    } else {
        Err(notify::Error::Generic(format!(
            "{:?} is not a file!",
            Path::new(CONFIG_FILE_NAME)
        )))
    }
}

macro_rules! clean_panic {
    ($message:literal$(,)? $($arg:expr),*) => {
        {
            use std::process;
            eprintln!($message, $($arg),*);
            process::exit(1);
        }
    }
}

trait CompareWithAbsolutePath {
    fn compare(&self, absolute_path: &Path) -> bool;
}

fn begin_watching<Watcher: 'static + Send>(
    watch_context: Arc<Mutex<WatchContext<Watcher>>>,
    website: Arc<Mutex<Website>>,
    filesystem_entry_name: &'static str,
    filesystem_entry_path: impl CompareWithAbsolutePath + Send + 'static,
    watch_context_maker: fn(&Config) -> WatchResult<Watcher>,
    mut event_receiver: impl FnMut(DebouncedEvent) + Send + 'static,
    mut resource_reloader: impl FnMut() + Send + 'static,
) {
    thread::spawn(move || loop {
        while let Ok(event) = watch_context.lock().unwrap().event_receiver.recv() {
            match event {
                DebouncedEvent::Remove(path) if filesystem_entry_path.compare(&path) => break,
                event => event_receiver(event),
            }
        }
        error!(
            "{} seems to be deleted. Waiting until it will be created...",
            filesystem_entry_name
        );
        loop {
            thread::sleep(Duration::from_secs(1));
            if let Ok(new_context) = watch_context_maker(website.lock().unwrap().config()) {
                *watch_context.lock().unwrap() = new_context;
                break;
            }
        }
        resource_reloader();
    });
}

#[derive(Clone)]
pub struct CustomServerHandle {
    server_handle: ServerHandle,
    restart_was_requested: Arc<AtomicBool>,
}

impl CustomServerHandle {
    #[must_use]
    pub fn new(server_handle: ServerHandle) -> Self {
        Self {
            server_handle,
            restart_was_requested: Arc::new(false.into()),
        }
    }

    #[must_use]
    pub fn restart_was_requested(self) -> bool {
        self.restart_was_requested.load(Ordering::Relaxed)
    }

    pub async fn request_restart(&self) {
        self.restart_was_requested.store(true, Ordering::Relaxed);
        self.server_handle.stop(true).await;
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Run => (),
        Command::CreateSampleConfig { force } => {
            let config_path = Path::new(CONFIG_FILE_NAME);
            assert!(
                !config_path.exists() || force,
                "`{:?}` already exists! To overwrite it, add a `--force` flag.",
                config_path
            );
            fs::write(
                config_path,
                serde_json::to_string_pretty(&config::Base::<PathBuf>::sample()).unwrap(),
            )
            .unwrap();
            return Ok(());
        }
    }

    let config: config::Base<PathBuf> = serde_json::from_str(
        &fs::read_to_string(Path::new(CONFIG_FILE_NAME)).unwrap_or_else(|error| {
            clean_panic!(
                "Configuration file isn't accessble! Consider creating a sample configuration \
                using `blog create_sample_config`, and then editing it. Details: {}",
                error
            );
        }),
    )
    .unwrap_or_else(|error| {
        clean_panic!(
            "Configuration file is poorly formatted!
            Fix it and try to `run` the program again. Details: {}",
            error
        );
    });
    set_global_log_level(&config.log_level).unwrap_or_else(|error| clean_panic!("{}", error));
    let config = config.upgrade().unwrap_or_else(|(error, config)| {
        clean_panic!(
            "Articles directory path (`{:?}`) cannot be expanded! Details: {}",
            config.articles_directory,
            error
        );
    });
    let articles_watch_context: Arc<Mutex<WatchContext<ArticlesWatcher>>> = Arc::new(Mutex::new(
        watch_articles(&config).unwrap_or_else(|error| {
            clean_panic!(
                "Articles directory `{:?}` is not accessible! Consider creating it. Details: {}",
                config.articles_directory.as_ref(),
                error
            );
        }),
    ));

    let config_watch_context: Arc<Mutex<WatchContext<ConfigWatcher>>> =
        Arc::new(Mutex::new(watch_config(&config).unwrap_or_else(|error| {
            clean_panic!(
                "Configuration file `{}` is not accessible! This is a really rare occasion that
                happened here. Consider creating the configuration file (probably, using the `blog
                create-sample-config` command). Details: {}",
                CONFIG_FILE_NAME,
                error
            );
        })));

    let website = Arc::new(Mutex::new(Website::new(config)));

    {
        let website = website.clone();
        begin_watching(
            articles_watch_context.clone(),
            website.clone(),
            "Articles directory",
            {
                struct ArticlesDirectory {
                    owner: Arc<Mutex<Website>>,
                }

                impl CompareWithAbsolutePath for ArticlesDirectory {
                    fn compare(&self, absolute_path: &Path) -> bool {
                        self.owner
                            .lock()
                            .unwrap()
                            .config()
                            .articles_directory
                            .as_ref()
                            == absolute_path
                    }
                }

                ArticlesDirectory {
                    owner: website.clone(),
                }
            },
            watch_articles,
            {
                let website = website.clone();
                move |event| {
                    match event {
                        DebouncedEvent::Remove(path) => {
                            website
                                .lock()
                                .unwrap()
                                .remove_article(&path.file_name_arc_str());
                        }
                        DebouncedEvent::Rename(from, to) => {
                            website
                                .lock()
                                .unwrap()
                                .rename_article(&from.file_name_arc_str(), to.file_name_arc_str());
                        }
                        DebouncedEvent::Write(path) | DebouncedEvent::Create(path) => {
                            website
                                .lock()
                                .unwrap()
                                .update_article(&path.file_name_arc_str());
                        }
                        _ => (),
                    };
                }
            },
            move || website.lock().unwrap().reload_articles_and_index(),
        );
    }

    loop {
        let server = run_server(&website);
        let server_handle = CustomServerHandle::new(server.handle());

        {
            let config_watch_context = config_watch_context.clone();
            let website = website.clone();
            let articles_watch_context = articles_watch_context.clone();

            let reload_config = {
                let config_watch_context = config_watch_context.clone();
                let website = website.clone();
                let articles_watch_context = articles_watch_context.clone();
                let server_handle = server_handle.clone();
                move || {
                    match fs::read_to_string(Path::new(CONFIG_FILE_NAME)) {
                        Ok(file_contents) => match serde_json::from_str(&file_contents) {
                            Ok(new_config) => website.lock().unwrap().config().update(
                                new_config,
                                &server_handle,
                                &website,
                                &articles_watch_context,
                                &config_watch_context,
                            ),
                            Err(error) => warn!(
                                "Updated configuration file is poorly formatted! \
                                Consider fixing it. Using the old configuration file \
                                for now. Details: {}",
                                error
                            ),
                        },
                        Err(error) => error!(
                            "Configuration file update was noticed, but the file \
                            couldn't be read. Details: {}",
                            error
                        ),
                    };
                }
            };

            #[allow(clippy::unit_arg)]
            begin_watching(
                config_watch_context,
                website.clone(),
                "Configuration file",
                {
                    impl CompareWithAbsolutePath for () {
                        fn compare(&self, _absolute_path: &Path) -> bool {
                            // Config watcher is non-recursive, so the path will always be the same
                            // (the configuration file's path)
                            true
                        }
                    }
                },
                watch_config,
                {
                    let reload_config = reload_config.clone();
                    move |event| {
                        match event {
                            DebouncedEvent::Write(_path) | DebouncedEvent::Create(_path) => {
                                reload_config();
                            }
                            _ => (),
                        };
                    }
                },
                reload_config,
            );
        }

        server.await?;

        if !server_handle.restart_was_requested() {
            break Ok(());
        }
    }
}
