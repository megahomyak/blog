use std::{
    fs, io,
    path::Path,
    sync::{Arc, Mutex},
};

use actix_web::{dev::Server, web, App, HttpServer};
use clap::{crate_description, Parser, Subcommand};
use config::Config;
use context::Context;
use file_watcher::FileWatcher;

mod config;
mod context;
mod file_watcher;
mod helpers;
mod page_compilers;
mod routes;

#[derive(Parser)]
#[clap(author, version, about = crate_description!(), long_about=None, propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Creates a sample config with all the values pre-filled
    CreateSampleConfig {
        #[clap(long, takes_value = false)]
        force: bool,
    },
    /// Runs the server
    Run,
}

const CONFIG_FILE_NAME: &str = "config.toml";

fn watch_articles(
    mut context: Arc<Mutex<Context<file_watcher::hotwatch::WatchGuard, file_watcher::hotwatch::WatchGuard>>>,
) -> file_watcher::hotwatch::WatchGuard {
    let path = context.lock().unwrap().config().articles_directory.clone();
    file_watcher::hotwatch::Watcher::new(path, move |event| {
        file_watcher::handle_event(&mut context, event);
    })
}

fn run_server<ArticlesWatchGuard: 'static, ConfigWatchGuard>(
    context: &Arc<Mutex<Context<ArticlesWatchGuard, ConfigWatchGuard>>>,
) -> Server
where
    ArticlesWatchGuard: file_watcher::WatchGuard + Send,
{
    HttpServer::new({
        let context = context.clone();
        move || {
            App::new()
                .app_data(web::Data::from(context.clone()))
                .route("/", web::get().to(routes::index::<ArticlesWatchGuard>))
                .route(
                    "/{filename}",
                    web::get().to(routes::file::<ArticlesWatchGuard>),
                )
        }
    })
    .bind((
        &context.lock().unwrap().config().host_name[..],
        context.lock().unwrap().config().port,
    ))
    .unwrap()
    .run()
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
                serde_json::to_string(&Config::sample()).unwrap(),
            )
            .unwrap();
            return Ok(());
        }
    }

    let config: Config = serde_json::from_str(
        &fs::read_to_string(Path::new(CONFIG_FILE_NAME)).expect(
            "Configuration file wasn't found! Consider creating a sample configuration using \
             `blog create_sample_config`, and then editing it.",
        ),
    )
    .expect("Configuration file is poorly formatted! Fix it and try to `run` the program again.");
    let context = Arc::new(Mutex::new(Context::new(config)));

    context.lock().unwrap().set_articles_watch_guard(watch_articles(context.clone()));

    loop {
        let server = run_server(&context);
        context.lock().unwrap().set_server_handle(server.handle());
        server.await?;
    }
}
