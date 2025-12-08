use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
// module imports
use crate::config::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateRouteRequest {
    pub path: String,
    pub upstream: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub key: String,
}

#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: i64,
    pub name: String,
}

pub async fn list_routes(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let routes = state
        .database
        .get_routes()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(routes))
}

pub async fn create_route(
    State(state): State<Arc<AppState>>,
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
    State(state): State<Arc<AppState>>,
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

pub async fn list_api_keys(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let api_keys = state
        .database
        .get_api_keys()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response: Vec<ApiKeyResponse> = api_keys
        .into_iter()
        .map(|k| ApiKeyResponse {
            id: k.id,
            name: k.name,
        })
        .collect();

    Ok(Json(response))
}

pub async fn create_api_key(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut hasher = Sha256::new();
    hasher.update(payload.key.as_bytes());
    let key_hash = format!("{:x}", hasher.finalize());
    let key = state
        .database
        .add_api_key(&payload.name, &key_hash)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(ApiKeyResponse {
            id: key.id,
            name: key.name,
        }),
    ))
}

pub async fn delete_api_key(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    let deleted = state
        .database
        .delete_api_key(&name)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
