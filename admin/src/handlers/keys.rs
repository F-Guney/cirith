use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
// imports
use crate::state::AdminState;
use cirith_shared::auth::hash_key;

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

pub async fn list_api_keys(
    State(state): State<Arc<AdminState>>,
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
    State(state): State<Arc<AdminState>>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let key_hash = hash_key(&payload.key);
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
    State(state): State<Arc<AdminState>>,
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
