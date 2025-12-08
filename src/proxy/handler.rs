use axum::extract::{ConnectInfo, State};
use axum::{
    body::Bytes,
    extract::{Path, Query},
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::config::AppState;
use crate::error::GatewayError;

const HOP_BY_HOP_HEADERS: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailers",
    "transfer-encoding",
    "upgrade",
];
pub async fn proxy_handler(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(path): Path<String>,
    Query(query_params): Query<HashMap<String, String>>,
    method: Method,
    headers: HeaderMap,
    body: Bytes,
) -> Result<impl IntoResponse, GatewayError> {
    state.metrics.increment_total();
    let api_key = headers.get("x-api-key").and_then(|key| key.to_str().ok());
    match api_key {
        Some(key) => {
            if !state.auth_validator.validate(key) {
                state.metrics.increment_unauthorized();
                return Err(GatewayError::Unauthorized);
            }
        }
        None => {
            if state.auth_validator.is_enabled() {
                state.metrics.increment_unauthorized();
                return Err(GatewayError::Unauthorized);
            }
        }
    }

    if !state.rate_limiter.check(addr.ip()) {
        state.metrics.increment_rate_limited();
        return Err(GatewayError::RateLimitExceeded);
    }

    let full_path = format!("/{}", path);
    let route = state
        .config
        .find_route(&full_path)
        .ok_or_else(|| GatewayError::RouteNotFound(full_path.clone()))?;

    let downstream_path = full_path.strip_prefix(&route.path).unwrap_or(&full_path);
    let query_string = if query_params.is_empty() {
        String::new()
    } else {
        let params: Vec<String> = query_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        format!("?{}", params.join("&"))
    };

    let upstream_url = format!("{}/{}{}", route.upstream, downstream_path, query_string);
    tracing::info!(
        method=%method,
        path=%path,
        query=%query_string,
        body_size=body.len(),
        "Proxying request"
    );

    let mut upstream_request = match method {
        Method::GET => state.http_client.get(&upstream_url),
        Method::POST => state.http_client.post(&upstream_url),
        Method::PUT => state.http_client.put(&upstream_url),
        Method::DELETE => state.http_client.delete(&upstream_url),
        Method::PATCH => state.http_client.patch(&upstream_url),
        Method::HEAD => state.http_client.head(&upstream_url),
        Method::OPTIONS => state.http_client.request(Method::OPTIONS, &upstream_url),
        other => {
            state.metrics.increment_failed();
            return Err(GatewayError::UnsupportedMethod(other.to_string()));
        }
    };

    for (name, value) in headers.iter() {
        let name_str = name.as_str().to_lowercase();

        if HOP_BY_HOP_HEADERS.contains(&name_str.as_str()) {
            continue;
        }

        if name_str == "host" {
            continue;
        }

        if name_str == "content-length" {
            continue;
        }

        if let Ok(value_str) = value.to_str() {
            upstream_request = upstream_request.header(name.as_str(), value_str);
        }
    }

    if !body.is_empty() {
        upstream_request = upstream_request.body(body.to_vec());
    }

    let response = match upstream_request.send().await {
        Ok(response) => {
            state.metrics.increment_successful();
            response
        }
        Err(e) => {
            state.metrics.increment_failed();
            return Err(GatewayError::UpstreamRequest(e));
        }
    };

    let response_status =
        StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::OK);

    let mut response_headers = HeaderMap::new();
    for (name, value) in response.headers().iter() {
        let name_str = name.as_str().to_lowercase();
        if !HOP_BY_HOP_HEADERS.contains(&name_str.as_str()) {
            response_headers.insert(name.clone(), value.clone());
        }
    }

    let response_body = response.bytes().await?;
    tracing::info!(status=%response_status, "Upstream responded");

    Ok((response_status, response_headers, response_body))
}
