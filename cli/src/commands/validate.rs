use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ValidateArgs {
    /// Specific file(s) to validate. With none given, validates every file
    /// under the discovered `components` directory.
    paths: Vec<PathBuf>,
}

pub fn run(args: ValidateArgs, extra_sources: &[PathBuf]) -> i32 {
    if args.paths.is_empty() {
        run_on_sources(extra_sources)
    } else {
        run_on_explicit_paths(&args.paths)
    }
}

fn run_on_sources(extra_sources: &[PathBuf]) -> i32 {
    let sources = match crate::common::resolve_sources(extra_sources) {
        Ok(sources) => sources,
        Err(message) => {
            eprintln!("{message}");
            return 1;
        }
    };

    let (valid, errors) = decdev_core::load_and_validate_sources(&sources);

    let mut rows: Vec<(String, Result<(), String>)> = valid
        .iter()
        .map(|c| (c.slug.clone(), Ok(())))
        .chain(errors.iter().map(|e| {
            let slug = e
                .file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string();
            (slug, Err(e.message.clone()))
        }))
        .collect();
    rows.sort_by(|a, b| a.0.cmp(&b.0));

    print_rows_and_exit_code(rows)
}

fn run_on_explicit_paths(paths: &[PathBuf]) -> i32 {
    let rows: Vec<(String, Result<(), String>)> = paths
        .iter()
        .map(|path| {
            let slug = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("?")
                .to_string();
            let result = decdev_core::validate_file(path).map(|_| ());
            (slug, result)
        })
        .collect();

    print_rows_and_exit_code(rows)
}

fn print_rows_and_exit_code(rows: Vec<(String, Result<(), String>)>) -> i32 {
    let mut exit_code = 0;
    for (slug, result) in rows {
        match result {
            Ok(()) => println!("OK {slug}"),
            Err(message) => {
                println!("FAIL {slug}: {message}");
                exit_code = 1;
            }
        }
    }
    exit_code
}
