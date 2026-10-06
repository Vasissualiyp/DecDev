use clap::Args;
use decdev_core::{Component, Extraction};

#[derive(Args)]
pub struct ShowArgs {
    slug: String,
    /// Print the raw component as JSON instead of a labeled human-readable form.
    #[arg(long)]
    json: bool,
}

pub fn run(args: ShowArgs) -> i32 {
    let Some(dir) = crate::common::find_components_dir() else {
        eprintln!("could not find a 'components' directory in the current directory or any parent");
        return 1;
    };

    let (valid, _errors) = decdev_core::load_and_validate_components(&dir);

    match valid.iter().find(|c| c.slug == args.slug) {
        Some(component) => {
            if args.json {
                match serde_json::to_string(component) {
                    Ok(s) => {
                        println!("{s}");
                        0
                    }
                    Err(e) => {
                        eprintln!("failed to serialize component to JSON: {e}");
                        1
                    }
                }
            } else {
                print_human(component);
                0
            }
        }
        None => {
            eprintln!("Unknown component: {}", args.slug);
            if let Some(closest) = valid
                .iter()
                .min_by_key(|c| crate::common::levenshtein(&c.slug, &args.slug))
            {
                eprintln!("Did you mean: {}?", closest.slug);
            }
            1
        }
    }
}

fn print_human(c: &Component) {
    println!("name: {}", c.name);
    println!("slug: {}", c.slug);
    println!("category: {}", c.category);
    println!("summary: {}", c.summary);
    if let Some(description) = &c.description {
        println!("description: {description}");
    }
    if !c.reference_games.is_empty() {
        println!("reference_games: {}", c.reference_games.join(", "));
    }
    if !c.genre_tags.is_empty() {
        println!("genre_tags: {}", c.genre_tags.join(", "));
    }

    println!("capability:");
    print_fields("inputs", &c.capability.inputs);
    print_fields("outputs", &c.capability.outputs);
    if !c.capability.events.is_empty() {
        println!("  events:");
        for event in &c.capability.events {
            match &event.payload {
                Some(payload) => println!("    - {} ({})", event.name, payload),
                None => println!("    - {}", event.name),
            }
        }
    }
    println!(
        "  dependencies: {}",
        if c.capability.dependencies.is_empty() {
            "(none)".to_string()
        } else {
            c.capability.dependencies.join(", ")
        }
    );
    if !c.capability.invariants.is_empty() {
        println!("  invariants:");
        for invariant in &c.capability.invariants {
            println!("    - {invariant}");
        }
    }
    println!("  determinism: {}", c.capability.determinism);

    println!("implementations:");
    for implementation in &c.implementations {
        println!(
            "  - engine: {}, language: {}, license: {}{}",
            implementation.engine,
            implementation.language,
            implementation.license,
            implementation
                .maturity
                .as_deref()
                .map(|m| format!(", maturity: {m}"))
                .unwrap_or_default()
        );
        println!("    url: {}", implementation.url);
        if let Some(demo_url) = &implementation.demo_url {
            println!("    demo_url: {demo_url}");
        }
        if let Some(extraction) = &implementation.extraction {
            print_extraction(extraction);
        }
    }

    println!("license: {}", c.license);
    println!("provenance:");
    println!("  type: {}", c.provenance.type_);
    if let Some(notes) = &c.provenance.notes {
        println!("  notes: {notes}");
    }
    if let Some(derived_from) = &c.provenance.derived_from {
        println!("  derived_from: {derived_from}");
    }
    if let Some(legal_review) = c.provenance.legal_review {
        println!("  legal_review: {legal_review}");
    }

    if let Some(version) = &c.version {
        println!("version: {version}");
    }
}

fn print_extraction(extraction: &Extraction) {
    println!("    extraction recipe (selective pull from a larger upstream project):");
    println!("      include:");
    for path in &extraction.include {
        println!("        - {path}");
    }
    println!("      entry_points:");
    for entry_point in &extraction.entry_points {
        match (&entry_point.symbol, &entry_point.description) {
            (Some(symbol), Some(description)) => {
                println!(
                    "        - {} :: {} ({})",
                    entry_point.path, symbol, description
                )
            }
            (Some(symbol), None) => println!("        - {} :: {}", entry_point.path, symbol),
            (None, Some(description)) => {
                println!("        - {} ({})", entry_point.path, description)
            }
            (None, None) => println!("        - {}", entry_point.path),
        }
    }
    if !extraction.exclude.is_empty() {
        println!("      exclude: {}", extraction.exclude.join(", "));
    }
    if !extraction.external_dependencies.is_empty() {
        println!("      external_dependencies:");
        for dependency in &extraction.external_dependencies {
            let version = dependency
                .version
                .as_deref()
                .map(|v| format!(" {v}"))
                .unwrap_or_default();
            let purpose = dependency
                .purpose
                .as_deref()
                .map(|p| format!(" — {p}"))
                .unwrap_or_default();
            println!("        - {}{}{}", dependency.name, version, purpose);
        }
    }
    if let Some(build_requirements) = &extraction.build_requirements {
        println!("      build_requirements: {build_requirements}");
    }
    if let Some(notes) = &extraction.notes {
        println!("      notes: {notes}");
    }
}

fn print_fields(label: &str, fields: &[decdev_core::Field]) {
    if fields.is_empty() {
        return;
    }
    println!("  {label}:");
    for field in fields {
        match &field.description {
            Some(description) => {
                println!("    - {}: {} ({})", field.name, field.type_, description)
            }
            None => println!("    - {}: {}", field.name, field.type_),
        }
    }
}
