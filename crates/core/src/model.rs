use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub category: String,
    pub summary: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub reference_games: Vec<String>,
    #[serde(default)]
    pub genre_tags: Vec<String>,
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
