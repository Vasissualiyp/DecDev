use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod common;

use commands::{list::ListArgs, search::SearchArgs, show::ShowArgs, validate::ValidateArgs};

#[derive(Parser)]
#[command(name = "decdev")]
struct Cli {
    /// Additional component source directories to load, on top of the
    /// discovered `components/` directory. Repeatable. Each source is a
    /// directory of `*.yaml` component specs, validated with exactly the
    /// same rules as the central registry — this is how a user loads their
    /// own recipes (from a local fork, a personal directory, or a
    /// third-party source repo) without editing this repo. The central
    /// registry's CI never passes this flag, so it only ever sees its own
    /// `components/`.
    #[arg(long = "source", global = true, value_name = "DIR")]
    sources: Vec<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List all valid components, optionally filtered by category.
    List(ListArgs),
    /// Search components by name/summary/description/reference_games, with optional filters.
    Search(SearchArgs),
    /// Show the full spec for one component by slug.
    Show(ShowArgs),
    /// Validate component spec files (defaults to every file under components/).
    Validate(ValidateArgs),
    /// Print the full validated component list as JSON. Strict: if any
    /// component fails validation, prints nothing to stdout and exits 1.
    Export,
}

fn main() {
    let Cli { sources, command } = Cli::parse();
    let exit_code = match command {
        Command::List(args) => commands::list::run(args, &sources),
        Command::Search(args) => commands::search::run(args, &sources),
        Command::Show(args) => commands::show::run(args, &sources),
        Command::Validate(args) => commands::validate::run(args, &sources),
        Command::Export => commands::export::run(&sources),
    };
    std::process::exit(exit_code);
}
