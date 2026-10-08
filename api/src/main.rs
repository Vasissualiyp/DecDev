use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "decdev-api",
    about = "Read-only HTTP API over the DecDev catalog"
)]
struct Args {
    /// Port to listen on (bound to 127.0.0.1).
    #[arg(long, default_value_t = 8080)]
    port: u16,
    /// Additional component source directories to load, on top of the
    /// discovered `components/` (repeatable). Same semantics as the CLI's
    /// `--source`.
    #[arg(long = "source", value_name = "DIR")]
    sources: Vec<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let sources = match decdev_core::resolve_sources(&args.sources) {
        Ok(sources) => sources,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    };

    // Strict, like `decdev export`: never serve a partially-invalid dataset.
    let (components, errors) = decdev_core::load_and_validate_sources(&sources);
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{}: {}", error.file.display(), error.message);
        }
        std::process::exit(1);
    }

    let router = decdev_api::router(components);
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("failed to bind {addr}: {e}");
            std::process::exit(1);
        }
    };

    eprintln!("decdev-api listening on http://{addr}");
    if let Err(e) = axum::serve(listener, router).await {
        eprintln!("server error: {e}");
        std::process::exit(1);
    }
}
