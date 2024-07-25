use axum::Router;
use clap::{Parser, Subcommand};
use sitdown::site::Site;
use std::net::SocketAddr;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// struct AppState {
//     env: Environment<'static>,
// }

#[derive(Parser)]
struct Args {
    /// command
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// start the server
    Serve,
    /// generate the files to serve
    Generate,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Serve => {
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| "static_site=debug,tower_http=debug".into()),
                )
                .with(tracing_subscriber::fmt::layer())
                .init();

            tokio::join!(serve(using_serve_dir(), 3000));
        }
        Commands::Generate => {
            Site::new().run();
        }
    }
}

fn using_serve_dir() -> Router {
    Router::new().nest_service("/", ServeDir::new("site"))
}

async fn serve(app: Router, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}
