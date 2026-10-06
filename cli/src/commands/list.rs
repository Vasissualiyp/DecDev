use clap::Args;

#[derive(Args)]
pub struct ListArgs {
    /// Filter to components with exactly this category.
    #[arg(long)]
    category: Option<String>,
    /// Print the full component array as JSON instead of tab-separated rows.
    #[arg(long)]
    json: bool,
}

pub fn run(args: ListArgs) -> i32 {
    let Some(dir) = crate::common::find_components_dir() else {
        eprintln!("could not find a 'components' directory in the current directory or any parent");
        return 1;
    };

    let (mut valid, _errors) = decdev_core::load_and_validate_components(&dir);
    if let Some(category) = &args.category {
        valid.retain(|c| &c.category == category);
    }

    crate::common::print_rows(&valid, args.json)
}
