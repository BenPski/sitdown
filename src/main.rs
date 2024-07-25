use axum::Router;
use clap::{Parser, Subcommand};
use performance_muscle::site::Site;
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
            let s = Site::new();
            println!("{s:?}");
            let p = s.prepare();
            println!("{p:?}");
            p.generate();
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

// // #[tokio::main]
// async fn main_templates() {
//     let mut env = Environment::new();
//     env.add_template("layout", include_str!("../templates/layout.jinja"))
//         .unwrap();
//     env.add_template("home", include_str!("../templates/home.jinja"))
//         .unwrap();
//     env.add_template("content", include_str!("../templates/content.jinja"))
//         .unwrap();
//     env.add_template("about", include_str!("../templates/about.jinja"))
//         .unwrap();
//
//     let app_state = Arc::new(AppState { env });
//
//     let app = Router::new()
//         .route("/", get(handler_home))
//         .route("/content", get(handler_content))
//         .route("/about", get(handler_about))
//         .with_state(app_state);
//
//     let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
//         .await
//         .unwrap();
//     println!("listening on {}", listener.local_addr().unwrap());
//     axum::serve(listener, app).await.unwrap();
// }
//
// async fn handler_home(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
//     let template = state.env.get_template("home").unwrap();
//
//     let rendered = template
//         .render(context! { title => "Home", welcome_text => "Hello"})
//         .unwrap();
//     Ok(Html(rendered))
// }
//
// async fn handler_content(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
//     let template = state.env.get_template("content").unwrap();
//
//     let entries_example = vec!["Data 1", "Data 2", "Data 3"];
//
//     let rendered = template
//         .render(context! { title => "Home", entries => entries_example})
//         .unwrap();
//     Ok(Html(rendered))
// }
//
// async fn handler_about(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
//     let template = state.env.get_template("about").unwrap();
//
//     let rendered = template
//         .render(context! { title => "About", about_text => "Something here"})
//         .unwrap();
//     Ok(Html(rendered))
// }
