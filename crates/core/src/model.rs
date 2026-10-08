use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    /// The component's filename stem (e.g. `quake-strafe-movement` for
    /// `quake-strafe-movement.yaml`) — the component's id. Never present in
    /// the authored YAML (the schema's `additionalProperties: false` would
    /// reject it there); set programmatically in `validate::validate_one`
    /// after the file passes validation, so every `Component` the rest of
    /// the codebase sees has it populated.
    #[serde(default)]
    pub slug: String,
    pub name: String,
    pub category: String,
    pub summary: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub reference_games: Vec<String>,
    #[serde(default)]
    pub genre_tags: Vec<String>,
    /// Manually-asserted relationships to other components, by slug — never
    /// computed. See `Compatibility` and
    /// `specs/01-capability-spec-format.md`.
    #[serde(default)]
    pub compatibility: Vec<Compatibility>,
    pub capability: Capability,
    pub implementations: Vec<Implementation>,
    pub license: String,
    pub provenance: Provenance,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    #[serde(default)]
    pub inputs: Vec<Field>,
    #[serde(default)]
    pub outputs: Vec<Field>,
    #[serde(default)]
    pub events: Vec<Event>,
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub invariants: Vec<String>,
    pub determinism: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    #[serde(default)]
    pub payload: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implementation {
    pub engine: String,
    pub language: String,
    pub url: String,
    pub license: String,
    #[serde(default)]
    pub maturity: Option<String>,
    #[serde(default)]
    pub demo_url: Option<String>,
    #[serde(default)]
    pub extraction: Option<Extraction>,
}

/// A recipe for pulling the smallest useful subset of a larger upstream
/// project rather than vendoring the whole repository. Only meaningful
/// when `provenance.derived_from` is set and `provenance.type` is not
/// `proprietary_analysis` — see `validate::validate_one` and
/// `specs/01-capability-spec-format.md`. Deliberately generic: nothing
/// here keys off the literal string `open_source_derived`, so a future
/// additional legitimate provenance category needs no changes here.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extraction {
    pub include: Vec<String>,
    pub entry_points: Vec<EntryPoint>,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub external_dependencies: Vec<ExternalDependency>,
    #[serde(default)]
    pub build_requirements: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    pub path: String,
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDependency {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub purpose: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub derived_from: Option<String>,
    #[serde(default)]
    pub legal_review: Option<bool>,
}

/// One hand-written compatibility assertion, authored on the component it
/// appears on and pointing at another component by slug. `relation` is
/// optional; absent means `pairs-with`. There is no reciprocal requirement in
/// the file — the reverse direction is derived for display (see
/// `inbound_compatibility`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compatibility {
    #[serde(rename = "with")]
    pub with_slug: String,
    #[serde(default)]
    pub relation: Option<String>,
    pub note: String,
}

/// Every assertion authored on *other* components that points at `slug`,
/// paired with the component that authored it — the reciprocal view the CLI
/// and site render so a relationship recorded once shows on both pages.
pub fn inbound_compatibility<'a>(
    all: &'a [Component],
    slug: &str,
) -> Vec<(&'a Component, &'a Compatibility)> {
    all.iter()
        .filter(|c| c.slug != slug)
        .flat_map(|c| {
            c.compatibility
                .iter()
                .filter(move |entry| entry.with_slug == slug)
                .map(move |entry| (c, entry))
        })
        .collect()
}

impl Component {
    /// The one definition of "does this component match a search query",
    /// shared by `decdev search` and the REST API's `/search` so the two
    /// surfaces can't drift. `query` is expected already lower-cased.
    pub fn matches_query(&self, query: &str) -> bool {
        self.name.to_lowercase().contains(query)
            || self.summary.to_lowercase().contains(query)
            || self
                .description
                .as_deref()
                .map(|d| d.to_lowercase().contains(query))
                .unwrap_or(false)
            || self
                .reference_games
                .iter()
                .any(|g| g.to_lowercase().contains(query))
    }
}
