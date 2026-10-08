use decdev_core::Component;
use serde_json::{json, Value};

/// Fallback protocol version when the client's `initialize` omits one.
const DEFAULT_PROTOCOL_VERSION: &str = "2025-06-18";

/// The loaded, immutable catalog plus the three read tools. Kept free of any
/// I/O so the tools and the JSON-RPC dispatcher are unit-testable directly
/// (see `specs/09-mcp.md`).
pub struct Catalog {
    components: Vec<Component>,
}

impl Catalog {
    pub fn new(components: Vec<Component>) -> Self {
        Self { components }
    }

    /// `list` tool: every component as a `{slug, name, category}` record.
    pub fn tool_list(&self) -> Value {
        json!({ "components": self.components.iter().map(summary).collect::<Vec<_>>() })
    }

    /// `show` tool: the full component JSON, or a message naming the unknown
    /// slug.
    pub fn tool_show(&self, slug: &str) -> Result<Value, String> {
        self.components
            .iter()
            .find(|c| c.slug == slug)
            .map(|c| serde_json::to_value(c).expect("Component is serializable"))
            .ok_or_else(|| format!("unknown component: {slug}"))
    }

    /// `search` tool: components matching `query`, using the same
    /// `Component::matches_query` the CLI and API use.
    pub fn tool_search(&self, query: &str) -> Value {
        let query = query.to_lowercase();
        json!({
            "components": self
                .components
                .iter()
                .filter(|c| c.matches_query(&query))
                .map(summary)
                .collect::<Vec<_>>()
        })
    }
}

fn summary(component: &Component) -> Value {
    json!({ "slug": component.slug, "name": component.name, "category": component.category })
}

fn tools() -> Value {
    json!([
        {
            "name": "list",
            "description": "List every component in the DecDev catalog.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false }
        },
        {
            "name": "show",
            "description": "Show the full spec for one component by slug.",
            "inputSchema": {
                "type": "object",
                "properties": { "slug": { "type": "string" } },
                "required": ["slug"],
                "additionalProperties": false
            }
        },
        {
            "name": "search",
            "description": "Search components by name/summary/description/reference_games.",
            "inputSchema": {
                "type": "object",
                "properties": { "query": { "type": "string" } },
                "required": ["query"],
                "additionalProperties": false
            }
        }
    ])
}

/// Dispatches one JSON-RPC message. Returns `None` for notifications (which
/// get no response) and `Some(response)` for anything that expects one.
pub fn handle_message(catalog: &Catalog, message: &Value) -> Option<Value> {
    let id = message.get("id").cloned();
    let has_id = message.get("id").is_some();
    let method = message.get("method").and_then(Value::as_str).unwrap_or("");
    let params = message.get("params").cloned().unwrap_or(Value::Null);

    match method {
        "initialize" => {
            let protocol_version = params
                .get("protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or(DEFAULT_PROTOCOL_VERSION);
            Some(result(
                id,
                json!({
                    "protocolVersion": protocol_version,
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "decdev", "version": env!("CARGO_PKG_VERSION") }
                }),
            ))
        }
        "notifications/initialized" => None,
        "ping" => Some(result(id, json!({}))),
        "tools/list" => Some(result(id, json!({ "tools": tools() }))),
        "tools/call" => Some(result(id, call_tool(catalog, &params))),
        _ => {
            if has_id {
                Some(error(id, -32601, format!("method not found: {method}")))
            } else {
                None
            }
        }
    }
}

fn call_tool(catalog: &Catalog, params: &Value) -> Value {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    match name {
        "list" => text_result(catalog.tool_list()),
        "show" => match arguments.get("slug").and_then(Value::as_str) {
            Some(slug) => match catalog.tool_show(slug) {
                Ok(value) => text_result(value),
                Err(message) => error_result(&message),
            },
            None => error_result("missing required argument: slug"),
        },
        "search" => match arguments.get("query").and_then(Value::as_str) {
            Some(query) => text_result(catalog.tool_search(query)),
            None => error_result("missing required argument: query"),
        },
        other => error_result(&format!("unknown tool: {other}")),
    }
}

fn text_result(value: Value) -> Value {
    let text = serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_string());
    json!({ "content": [{ "type": "text", "text": text }], "isError": false })
}

fn error_result(message: &str) -> Value {
    json!({ "content": [{ "type": "text", "text": message }], "isError": true })
}

fn result(id: Option<Value>, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error(id: Option<Value>, code: i64, message: String) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}
