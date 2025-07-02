mod error;
mod routes;
mod shared_state;
mod util;

use axum::{
    Router,
    routing::{get, head},
};
use clap::{Parser, arg};
use shared_state::SharedState;
use std::path::PathBuf;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The port that s3plz should run on.
    #[arg(short = 'p', long = "port", env = "S3PLZ_PORT")]
    port: u32,

    // The host that s3plz should run on.
    #[arg(
        short = 'd',
        long = "host",
        env = "S3PLZ_HOST",
        default_value = "localhost"
    )]
    host: String,

    /// the path that will be read and served. the bucket name is the LAST PATH
    /// SEGMENT OF THE PATH. that is, if you pass in `/etc/files/manifests`, the
    /// bucket name becomes `manifests`.
    path: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let now = chrono::Utc::now();
    let url = format!("{}:{}", args.host, args.port);

    debug!("serving on {}. Global LastModified: {}", url, now);

    let path = PathBuf::from(&args.path);
    let bucket = path
        .file_stem()
        .expect(format!("could not conclude bucket from path {:?}", path).as_str())
        .to_str()
        .expect(
            format!(
                "could not convert {:?} to a str; please remove special chars.",
                path
            )
            .as_str(),
        );

    let shared_state = SharedState::new(path.clone(), bucket.to_string(), now)
        .map_err(|e| format!("could not parse dir {}: {}", args.path, e))
        .unwrap();

    let api = Router::<std::sync::Arc<SharedState>>::new()
        .route("/", get(routes::root::get_root))
        .route("/", head(routes::root::head_root))
        // dirty hack until https://github.com/ibraheemdev/matchit/issues/39
        // gets merged. could also use a middleware but -\_( '' )_/-
        .route("/{a}", get(routes::path::get_path_oneseg))
        .route("/{a}/{b}", get(routes::path::get_path_twoseg))
        .route("/{a}/{b}/{c}", get(routes::path::get_path_threeseg))
        .route("/{a}/{b}/{c}/{d}", get(routes::path::get_path_fourseg))
        // if you want more than 4 path segments feel free to pr.
        .fallback(util::fallback)
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(&url).await.unwrap();
    let app = Router::new().nest(format!("/{}", bucket).as_str(), api);
    axum::serve(listener, app).await.unwrap();
}
