/// Finds the component source directories (the discovered `components/`
/// plus any `--source` extras), loads and validates every component in
/// them, and prints the result. Strict: any validation error means nothing
/// is printed to stdout and the process exits 1 — this is the invariant the
/// website build (`decdev export > site/src/data/components.json`) depends
/// on. The website build passes no `--source`, so it is unaffected by this
/// flag.
pub fn run(extra_sources: &[std::path::PathBuf]) -> i32 {
    let sources = match crate::common::resolve_sources(extra_sources) {
        Ok(sources) => sources,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
    };

    let (valid, errors) = decdev_core::load_and_validate_sources(&sources);

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
