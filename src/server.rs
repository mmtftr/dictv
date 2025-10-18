use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use std::sync::Arc;
use std::time::Instant;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::models::{HealthResponse, SearchQuery, SearchResponse, StatsResponse};
use crate::search::SearchEngine;

/// Application state
#[derive(Clone)]
pub struct AppState {
    search_engine: Arc<SearchEngine>,
}

impl AppState {
    pub fn new(search_engine: SearchEngine) -> Self {
        Self {
            search_engine: Arc::new(search_engine),
        }
    }
}

/// Create the HTTP server router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/search", get(search_handler))
        .route("/health", get(health_handler))
        .route("/stats", get(stats_handler))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Search endpoint handler
async fn search_handler(
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, AppError> {
    let start = Instant::now();

    // Validate query
    if params.q.is_empty() {
        return Err(AppError::BadRequest("Query cannot be empty".to_string()));
    }

    if params.max_distance > 2 {
        return Err(AppError::BadRequest("max_distance must be 0-2".to_string()));
    }

    // Perform search
    let results = state
        .search_engine
        .search(
            &params.q,
            params.mode,
            params.lang,
            params.max_distance,
            params.limit,
        )
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let query_time_ms = start.elapsed().as_secs_f64() * 1000.0;
    let total_results = results.len();

    Ok(Json(SearchResponse {
        results,
        query_time_ms,
        total_results,
    }))
}

/// Health check endpoint handler
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Statistics endpoint handler
async fn stats_handler(State(state): State<AppState>) -> Result<Json<StatsResponse>, AppError> {
    let (total_entries, en_de_entries, de_en_entries) = state
        .search_engine
        .get_stats()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    // Get index size (approximate)
    let index_size_bytes = 0; // TODO: Implement actual size calculation

    Ok(Json(StatsResponse {
        total_entries,
        en_de_entries,
        de_en_entries,
        index_size_bytes,
    }))
}

/// Custom error type for HTTP handlers
#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    _NotFound(String),
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::_NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = serde_json::json!({
            "error": message,
        });

        (status, Json(body)).into_response()
    }
}

/// Start the HTTP server
pub async fn serve(search_engine: SearchEngine, port: u16) -> anyhow::Result<()> {
    let state = AppState::new(search_engine);
    let app = create_router(state);

    let addr = format!("127.0.0.1:{}", port);
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint() {
        let response = health_handler().await;
        assert_eq!(response.0.status, "ok");
    }
}
