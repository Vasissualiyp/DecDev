/// Finds the components directory, loads and validates every component in
/// it, and prints the result. Strict: any validation error means nothing is
/// printed to stdout and the process exits 1 — this is the invariant the
/// website build (`decdev export > site/src/data/components.json`) depends
/// on.
pub fn run() -> i32 {
    let Some(components_dir) = crate::common::find_components_dir() else {
        eprintln!("could not find a 'components' directory in the current directory or any parent");
        return 1;
    };

    let (valid, errors) = decdev_core::load_and_validate_components(&components_dir);

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{}: {}", error.file.display(), error.message);
        }
        return 1;
    }

    match serde_json::to_string(&valid) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(e) => {
            eprintln!("failed to serialize components to JSON: {e}");
            1
        }
    }
}
