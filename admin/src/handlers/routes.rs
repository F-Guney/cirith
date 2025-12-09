use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use std::sync::Arc;
// module imports
use crate::state::AdminState;

#[derive(Debug, Deserialize)]
pub struct CreateRouteRequest {
    pub path: String,
    pub upstream: String,
}

pub async fn list_routes(
    State(state): State<Arc<AdminState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let routes = state
        .database
        .get_routes()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(routes))
}

pub async fn create_route(
    State(state): State<Arc<AdminState>>,
    Json(payload): Json<CreateRouteRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let route = state
        .database
        .add_route(&payload.path, &payload.upstream)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(route)))
}

pub async fn delete_route(
    State(state): State<Arc<AdminState>>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let path = format!("/{}", path);
    let deleted = state
        .database
        .delete_route(&path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
