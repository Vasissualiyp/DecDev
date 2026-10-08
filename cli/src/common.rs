use std::path::PathBuf;

/// Finds the `components` directory by walking up from the current working
/// directory, checking up to 10 parent levels. Shared by every command that
/// operates on the whole catalog (list/search/show/validate/export) so a
/// contributor can run `decdev <command>` from anywhere inside the repo.
pub fn find_components_dir() -> Option<PathBuf> {
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

/// Builds the ordered list of component source directories for a command:
/// the discovered `components/` directory (if any) followed by any explicit
/// `--source` extras. Returns an error only when there is nothing to read.
/// Shared by every command so `--source` behaves identically everywhere.
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

/// Prints a component list either as tab-separated rows (`slug\tname\tcategory`)
/// or, with `json: true`, as a JSON array. Shared by `list` and `search`,
/// which both need identical output formatting. Always returns 0 — an
/// empty result set is not an error for either command.
pub fn print_rows(components: &[decdev_core::Component], json: bool) -> i32 {
    if json {
        match serde_json::to_string(components) {
            Ok(s) => {
                println!("{s}");
                0
            }
            Err(e) => {
                eprintln!("failed to serialize components to JSON: {e}");
                1
            }
        }
    } else {
        for c in components {
            println!("{}\t{}\t{}", c.slug, c.name, c.category);
        }
        0
    }
}

/// Levenshtein edit distance, used by `show` to suggest a close slug on a
/// typo. Small and dependency-free rather than pulling in a crate for one
/// ~15-line algorithm.
pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();

    for i in 1..=a.len() {
        let mut prev_diag = row[0];
        row[0] = i;
        for j in 1..=b.len() {
            let temp = row[j];
            row[j] = if a[i - 1] == b[j - 1] {
                prev_diag
            } else {
                1 + prev_diag.min(row[j]).min(row[j - 1])
            };
            prev_diag = temp;
        }
    }

    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_identical_strings_is_zero() {
        assert_eq!(
            levenshtein("quake-strafe-movement", "quake-strafe-movement"),
            0
        );
    }

    #[test]
    fn levenshtein_one_char_off() {
        assert_eq!(
            levenshtein("quake-strafe-movment", "quake-strafe-movement"),
            1
        );
    }
}
