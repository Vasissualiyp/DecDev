use std::path::PathBuf;

/// Finds the `components` directory by walking up from the current working
/// directory (checking up to 10 parent levels), loads and validates every
/// component in it, and prints the result. Strict: any validation error
/// means nothing is printed to stdout and the process exits 1 — this is the
/// invariant the website build (`decdev export > site/src/data/components.json`)
/// depends on.
pub fn run() -> i32 {
    let components_dir = match find_components_dir() {
        Some(dir) => dir,
        None => {
            eprintln!(
                "could not find a 'components' directory in the current directory or any parent"
            );
            return 1;
        }
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

fn find_components_dir() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..10 {
        let candidate = dir.join("components");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}
