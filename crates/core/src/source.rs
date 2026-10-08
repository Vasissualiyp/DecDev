use std::path::{Path, PathBuf};

/// Finds the `components` directory by walking up from the current working
/// directory, checking up to 10 parent levels. Shared by the CLI and the
/// REST API so both discover the catalog the same way.
pub fn find_components_dir() -> Option<PathBuf> {
    let dir = std::env::current_dir().ok()?;
    find_components_dir_from(&dir)
}

/// Same as `find_components_dir` but from an explicit starting directory,
/// so callers (and tests) don't have to depend on the process cwd.
pub fn find_components_dir_from(start: &Path) -> Option<PathBuf> {
    let mut dir = start.to_path_buf();
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

/// Builds the ordered list of component source directories: the discovered
/// `components/` directory (if any) followed by any explicit `--source`
/// extras. Returns an error only when there is nothing to read. Shared by
/// the CLI's commands and the API's startup so `--source` behaves the same
/// everywhere.
pub fn resolve_sources(extra: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut sources = Vec::new();
    if let Some(dir) = find_components_dir() {
        sources.push(dir);
    }
    sources.extend(extra.iter().cloned());
    if sources.is_empty() {
        return Err(
            "could not find a 'components' directory in the current directory or any parent, and no --source was given"
                .to_string(),
        );
    }
    Ok(sources)
}
