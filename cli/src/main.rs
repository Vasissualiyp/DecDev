use clap::{Parser, Subcommand};

mod commands;
mod common;

use commands::{list::ListArgs, search::SearchArgs, show::ShowArgs, validate::ValidateArgs};

#[derive(Parser)]
#[command(name = "decdev")]
struct Cli {
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
    let cli = Cli::parse();
    let exit_code = match cli.command {
        Command::List(args) => commands::list::run(args),
        Command::Search(args) => commands::search::run(args),
        Command::Show(args) => commands::show::run(args),
        Command::Validate(args) => commands::validate::run(args),
        Command::Export => commands::export::run(),
    };
    std::process::exit(exit_code);
}
