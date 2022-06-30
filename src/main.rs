use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use actix_web::{dev::Server, web, App, HttpServer};
use clap::{crate_description, Parser, Subcommand};
use config::Config;
use log::{error, warn};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use utils::{set_global_log_level, FileNameShortcut};
use website::Website;

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

pub struct WatchContext {
    _watcher: RecommendedWatcher,
    event_receiver: mpsc::Receiver<DebouncedEvent>,
}

pub type WatchResult = Result<WatchContext, notify::Error>;

/// # Panics
/// Panics when there were some issues with the `mio`'s `EventLoop`. Don't want to handle those, to
/// be honest. It is too hard for them to occur anyway, but when they will occur, I don't want to do
/// anything, because __the main point__ of this program is to listen (or "watch") to file changes.
#[allow(clippy::missing_errors_doc)]
pub fn watch(config: &Config, path: &Path, recursive_mode: RecursiveMode) -> WatchResult {
    let (event_sender, event_receiver) = mpsc::channel();
    let mut watcher: RecommendedWatcher = Watcher::new(
        event_sender,
        Duration::from_millis(config.file_watcher_delay_in_milliseconds),
    )
    .unwrap();
    watcher.watch(path, recursive_mode)?;
    Ok(WatchContext {
        _watcher: watcher,
        event_receiver,
    })
}

#[allow(clippy::missing_errors_doc)]
pub fn watch_articles(config: &Config) -> WatchResult {
    if config.articles_directory.as_ref().is_dir() {
        watch(
            config,
            config.articles_directory.as_ref(),
            RecursiveMode::Recursive,
        )
    } else {
        Err(notify::Error::Generic(format!(
            "{:?} is not a directory!",
            config.articles_directory.as_ref()
        )))
    }
}

#[allow(clippy::missing_errors_doc)]
pub fn watch_config(config: &Config) -> WatchResult {
    if Path::new(CONFIG_FILE_NAME).is_file() {
        watch(
            config,
            Path::new(CONFIG_FILE_NAME),
            RecursiveMode::NonRecursive,
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

fn begin_watching(
    watch_context: Arc<Mutex<WatchContext>>,
    website: Arc<Mutex<Website>>,
    filesystem_entry_name: &'static str,
    filesystem_entry_path: impl PartialEq<PathBuf> + Send + 'static,
    watch_context_maker: fn(&Config) -> WatchResult,
    mut event_receiver: impl FnMut(DebouncedEvent) + Send + 'static,
    mut resource_reloader: impl FnMut() + Send + 'static,
) {
    thread::spawn(move || loop {
        while let Ok(event) = watch_context.lock().unwrap().event_receiver.recv() {
            match event {
                DebouncedEvent::Remove(path) if filesystem_entry_path == path => break,
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
    set_global_log_level(&config.log_level);
    let config = config.upgrade().unwrap_or_else(|(error, config)| {
        clean_panic!(
            "Articles directory path (`{:?}`) cannot be expanded! Details: {}",
            config.articles_directory,
            error
        );
    });
    let articles_watch_context = Arc::new(Mutex::new(watch_articles(&config).unwrap_or_else(
        |error| {
            clean_panic!(
                "Articles directory `{:?}` is not accessible! Consider creating it. Details: {}",
                config.articles_directory.as_ref(),
                error
            );
        },
    )));

    let config_watch_context = Arc::new(Mutex::new(watch_articles(&config).unwrap_or_else(
        |error| {
            clean_panic!(
                "Configuration file `{}` is not accessible! This is a really rare occasion that
                happened here. Consider creating the configuration file (probably, using the `blog
                create-sample-config` command). Details: {}",
                CONFIG_FILE_NAME,
                error
            );
        },
    )));

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

                impl PartialEq<PathBuf> for ArticlesDirectory {
                    fn eq(&self, other: &PathBuf) -> bool {
                        self.owner
                            .lock()
                            .unwrap()
                            .config()
                            .articles_directory
                            .as_ref()
                            == other
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
        let mut server_handle = server.handle();

        {
            let config_watch_context = config_watch_context.clone();
            let website = website.clone();
            let articles_watch_context = articles_watch_context.clone();

            let reload_config = {
                let config_watch_context = config_watch_context.clone();
                let website = website.clone();
                let articles_watch_context = articles_watch_context.clone();
                move || {
                    match fs::read_to_string(Path::new(CONFIG_FILE_NAME)) {
                        Ok(file_contents) => match serde_json::from_str(&file_contents) {
                            Ok(new_config) => website.lock().unwrap().config().update(
                                new_config,
                                &mut server_handle,
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

            begin_watching(
                config_watch_context,
                website.clone(),
                "Configuration file",
                Path::new(CONFIG_FILE_NAME),
                watch_config,
                {
                    let mut reload_config = reload_config.clone();
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
    }
}
