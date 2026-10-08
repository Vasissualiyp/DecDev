use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use decdev_core::Component;
use serde::Deserialize;
use std::sync::Arc;

/// Immutable in-memory catalog the API serves. Read-only by design: the API
/// loads and validates once at startup and never writes (see
/// `specs/08-api.md`). Wrapped in `Arc` so cloning the router state is cheap.
#[derive(Clone)]
pub struct AppState {
    components: Arc<Vec<Component>>,
}

/// Builds the read-only router over an already-validated component list.
/// Kept separate from `main` so tests can drive it directly, with no real
/// socket and no filesystem dependency.
pub fn router(components: Vec<Component>) -> Router {
    let state = AppState {
        components: Arc::new(components),
    };

    Router::new()
        .route("/components", get(list_components))
        .route("/components/{slug}", get(show_component))
        .route("/search", get(search_components))
        .with_state(state)
}

#[derive(Debug)]
enum ApiError {
    NotFound(String),
    BadRequest(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::NotFound(slug) => {
                (StatusCode::NOT_FOUND, format!("unknown component: {slug}"))
            }
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

async fn list_components(State(state): State<AppState>) -> Json<Vec<Component>> {
    Json((*state.components).clone())
}

async fn show_component(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Component>, ApiError> {
    state
        .components
        .iter()
        .find(|c| c.slug == slug)
        .cloned()
        .map(Json)
        .ok_or(ApiError::NotFound(slug))
}

#[derive(Deserialize)]
struct SearchParams {
    q: Option<String>,
}

async fn search_components(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<Component>>, ApiError> {
    let query = params.q.unwrap_or_default();
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Err(ApiError::BadRequest(
            "missing required query parameter: q".to_string(),
        ));
    }

    let results: Vec<Component> = state
        .components
        .iter()
        .filter(|c| c.matches_query(&query))
        .cloned()
        .collect();

    Ok(Json(results))
}
