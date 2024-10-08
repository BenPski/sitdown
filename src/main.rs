use axum::Router;
use clap::{Parser, Subcommand};
use notify::{RecursiveMode, Watcher};
use notify_debouncer_full::new_debouncer;
use sitdown::app::App;
use sitdown::error::Result;
use sitdown::utils::{create_new, get_config};
use std::fs;
use std::{collections::HashSet, net::SocketAddr, time::Duration};
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(about, version)]
struct Args {
    /// command
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// create a new minimal site
    New {
        /// name of the new site
        name: String,
    },
    /// start the server
    Serve,
    /// generate the files to serve
    Generate,
    /// watch for updates and re-generate site on updates
    Watch,
    /// clean up the generated files
    Clean,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Serve => {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "sitdown=debug,tower_http=debug".into()),
                )
                .with(tracing_subscriber::fmt::layer())
                .init();

            tokio::join!(serve(using_serve_dir(), 3000));
        }
        Commands::Generate => {
            let config = get_config();
            let app = App::new(&config);
            app.create().unwrap();
            // Site::new().run();
        }
        Commands::Watch => {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                .init();
            if let Err(err) = watch() {
                log::error!("Encountered error `{err:?}`");
            }
        }
        Commands::New { name } => {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                .init();
            if let Err(err) = create_new(name) {
                log::error!("Encountered error `{err}`");
            }
        }
        Commands::Clean => {
            let config = get_config();
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                .init();
            if let Err(err) = fs::remove_dir_all(config.structure.site) {
                log::error!("Encountered error `{err}`");
            }

            if let Err(err) = fs::remove_dir_all(config.structure.work) {
                log::error!("Encountered error `{err}`");
            }
        } // _ => println!("lol"),
    }
}

fn watch() -> Result<()> {
    let config = get_config();
    let (tx, rx) = std::sync::mpsc::channel();

    let mut debouncer = new_debouncer(Duration::from_secs(2), None, tx)?;

    debouncer
        .watcher()
        .watch(config.structure.assets.as_ref(), RecursiveMode::Recursive)?;
    debouncer
        .watcher()
        .watch(config.structure.template.as_ref(), RecursiveMode::Recursive)?;
    debouncer
        .watcher()
        .watch(config.structure.content.as_ref(), RecursiveMode::Recursive)?;

    for res in rx {
        match res {
            Ok(event) => {
                let updated: HashSet<_> = event.into_iter().flat_map(|e| e.paths.clone()).collect();
                log::info!("Changes in: {updated:?}");
                log::info!("Regenerating");
                App::new(&config).create()?;
            }
            Err(error) => {
                println!("Error received `{error:?}`");
            }
        }
    }
    Ok(())
}

fn using_serve_dir() -> Router {
    let config = get_config();
    Router::new().nest_service("/", ServeDir::new(config.structure.site))
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}
