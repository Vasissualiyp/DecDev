use clap::Args;
use decdev_core::Component;
use std::path::PathBuf;

#[derive(Args)]
pub struct SearchArgs {
    /// Case-insensitive substring to match against name/summary/description/reference_games.
    query: String,
    /// Filter to components with exactly this category.
    #[arg(long)]
    category: Option<String>,
    /// Filter to components with at least one implementation using this engine.
    #[arg(long)]
    engine: Option<String>,
    /// Filter to components with at least one implementation under this license
    /// (an implementation's license, NOT the top-level spec license).
    #[arg(long)]
    license: Option<String>,
    /// Print the full component array as JSON instead of tab-separated rows.
    #[arg(long)]
    json: bool,
}

pub fn run(args: SearchArgs, extra_sources: &[PathBuf]) -> i32 {
    let sources = match crate::common::resolve_sources(extra_sources) {
        Ok(sources) => sources,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
    };

    let (valid, _errors) = decdev_core::load_and_validate_sources(&sources);
    let query = args.query.to_lowercase();

    let results: Vec<Component> = valid
        .into_iter()
        .filter(|c| c.matches_query(&query))
        .filter(|c| args.category.as_deref().is_none_or(|cat| c.category == cat))
        .filter(|c| {
            args.engine
                .as_deref()
                .is_none_or(|eng| c.implementations.iter().any(|i| i.engine == eng))
        })
        .filter(|c| {
            args.license
                .as_deref()
                .is_none_or(|lic| c.implementations.iter().any(|i| i.license == lic))
        })
        .collect();

    crate::common::print_rows(&results, args.json)
}
