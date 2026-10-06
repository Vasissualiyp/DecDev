use clap::{Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "decdev")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the full validated component list as JSON. Strict: if any
    /// component fails validation, prints nothing to stdout and exits 1.
    Export,
}

fn main() {
    let cli = Cli::parse();
    let exit_code = match cli.command {
        Command::Export => commands::export::run(),
    };
    std::process::exit(exit_code);
}
