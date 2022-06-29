use std::{
    fs, io,
    path::Path,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use actix_web::{dev::Server, web, App, HttpServer};
use clap::{crate_description, Parser, Subcommand};
use config::Config;
use log::error;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use utils::FileNameShortcut;
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

const CONFIG_FILE_NAME: &str = "config.toml";

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
    if Path::new(&config.articles_directory).is_dir() {
        watch(config, &config.articles_directory, RecursiveMode::Recursive)
    } else {
        Err(notify::Error::Generic(format!(
            "{:?} is not a directory!",
            &config.articles_directory
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
                serde_json::to_string_pretty(&Config::sample()).unwrap(),
            )
            .unwrap();
            return Ok(());
        }
    }

    let config: Config = serde_json::from_str(
        &fs::read_to_string(Path::new(CONFIG_FILE_NAME)).unwrap_or_else(|_| {
            clean_panic!(
                "Configuration file wasn't found! Consider creating a sample configuration using \
                `blog create_sample_config`, and then editing it."
            );
        }),
    )
    .unwrap_or_else(|_| {
        clean_panic!(
            "Configuration file is poorly formatted! Fix it and try to `run` the program again."
        );
    });
    let articles_watch_context =
        Arc::new(Mutex::new(watch_articles(&config).unwrap_or_else(|_| {
            clean_panic!(
                "Articles directory `{:?}` was not found! Consider creating it",
                config.articles_directory
            );
        })));

    let website = Arc::new(Mutex::new(Website::new(config)));

    let config_watch_context = Arc::new(Mutex::new(
        watch_articles(website.lock().unwrap().config()).unwrap_or_else(|_| {
            clean_panic!(
                "Configuration file `{}` was not found! This is a really rare occasion that
                happened here. Consider creating the configuration file (probably, using the `blog
                create-sample-config` command)",
                CONFIG_FILE_NAME
            );
        }),
    ));

    thread::spawn({
        let articles_watch_context = articles_watch_context.clone();
        let website = website.clone();
        move || {
            while let Ok(event) = articles_watch_context.lock().unwrap().event_receiver.recv() {
                match event {
                    DebouncedEvent::Remove(path) => {
                        if path == website.lock().unwrap().config().articles_directory {
                            error!(
                                "Articles directory seems to be deleted. Waiting until it will be \
                                created..."
                            );
                            loop {
                                match watch_articles(website.lock().unwrap().config()) {
                                    Ok(new_context) => {
                                        *articles_watch_context.lock().unwrap() = new_context;
                                        break;
                                    }
                                    Err(_error) => thread::sleep(Duration::from_secs(1)),
                                }
                            }
                        } else {
                            website
                                .lock()
                                .unwrap()
                                .remove_article(&path.file_name_arc_str());
                        }
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
        }
    });

    loop {
        let server = run_server(&website);
        let mut server_handle = server.handle();

        thread::spawn({
            let articles_watch_context = articles_watch_context.clone();
            let website = website.clone();
            let config_watch_context = config_watch_context.clone();
            move || {
                while let Ok(event) = config_watch_context.lock().unwrap().event_receiver.recv() {
                    match event {
                        DebouncedEvent::Write(path) | DebouncedEvent::Create(path) => {
                            if let Ok(file_contents) = fs::read_to_string(path) {
                                if let Ok(new_config) = serde_json::from_str(&file_contents) {
                                    website.lock().unwrap().config().update(
                                        new_config,
                                        &mut server_handle,
                                        &website,
                                        &articles_watch_context,
                                        &config_watch_context,
                                    );
                                }
                            }
                        }
                        DebouncedEvent::Remove(path) => {
                            if path == Path::new(CONFIG_FILE_NAME) {
                                error!(
                                    "Config file seems to be deleted. \
                                    Waiting until it will be created..."
                                );
                                loop {
                                    match watch_config(website.lock().unwrap().config()) {
                                        Ok(new_context) => {
                                            *config_watch_context.lock().unwrap() = new_context;
                                            break;
                                        }
                                        Err(_error) => thread::sleep(Duration::from_secs(1)),
                                    }
                                }
                            }
                        }
                        _ => (),
                    };
                }
            }
        });

        server.await?;
    }
}
