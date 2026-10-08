use clap::Parser;
use std::io::{BufRead, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "decdev-mcp",
    about = "MCP stdio server over the DecDev catalog"
)]
struct Args {
    /// Additional component source directories to load, on top of the
    /// discovered `components/` (repeatable). Same semantics as the CLI.
    #[arg(long = "source", value_name = "DIR")]
    sources: Vec<PathBuf>,
}

fn main() {
    let args = Args::parse();

    let sources = match decdev_core::resolve_sources(&args.sources) {
        Ok(sources) => sources,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    };

    // Strict, like `decdev export` and the API: never serve partial data.
    let (components, errors) = decdev_core::load_and_validate_sources(&sources);
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{}: {}", error.file.display(), error.message);
        }
        std::process::exit(1);
    }

    let catalog = decdev_mcp::Catalog::new(components);

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    // Newline-delimited JSON-RPC 2.0 over stdio (the MCP stdio transport).
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let message: serde_json::Value = match serde_json::from_str(&line) {
            Ok(message) => message,
            Err(e) => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": serde_json::Value::Null,
                    "error": { "code": -32700, "message": format!("parse error: {e}") }
                });
                let _ = writeln!(out, "{response}");
                let _ = out.flush();
                continue;
            }
        };

        if let Some(response) = decdev_mcp::handle_message(&catalog, &message) {
            let _ = writeln!(out, "{response}");
            let _ = out.flush();
        }
    }
}
