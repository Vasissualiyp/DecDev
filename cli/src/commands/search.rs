use clap::Args;
use decdev_core::Component;

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

pub fn run(args: SearchArgs) -> i32 {
    let Some(dir) = crate::common::find_components_dir() else {
        eprintln!("could not find a 'components' directory in the current directory or any parent");
        return 1;
    };

    let (valid, _errors) = decdev_core::load_and_validate_components(&dir);
    let query = args.query.to_lowercase();

    let results: Vec<Component> = valid
        .into_iter()
        .filter(|c| matches_query(c, &query))
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

fn matches_query(c: &Component, query: &str) -> bool {
    c.name.to_lowercase().contains(query)
        || c.summary.to_lowercase().contains(query)
        || c.description
            .as_deref()
            .map(|d| d.to_lowercase().contains(query))
            .unwrap_or(false)
        || c.reference_games
            .iter()
            .any(|g| g.to_lowercase().contains(query))
}
