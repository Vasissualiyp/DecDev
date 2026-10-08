use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ListArgs {
    /// Filter to components with exactly this category.
    #[arg(long)]
    category: Option<String>,
    /// Print the full component array as JSON instead of tab-separated rows.
    #[arg(long)]
    json: bool,
}

pub fn run(args: ListArgs, extra_sources: &[PathBuf]) -> i32 {
    let sources = match crate::common::resolve_sources(extra_sources) {
        Ok(sources) => sources,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
    };

    let (mut valid, _errors) = decdev_core::load_and_validate_sources(&sources);
    if let Some(category) = &args.category {
        valid.retain(|c| &c.category == category);
    }

    crate::common::print_rows(&valid, args.json)
}
