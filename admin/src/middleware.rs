use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use std::sync::Arc;
// imports
use crate::state::AdminState;

pub async fn auth_middleware(
    State(state): State<Arc<AdminState>>,
    request: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    let auth_header = request.headers().get("authorization");

    match auth_header {
        Some(auth_header) => {
            let auth_str = auth_header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;
            let token = auth_str
                .strip_prefix("Bearer ")
                .ok_or(StatusCode::UNAUTHORIZED)?;

            if token != state.config.admin.token {
                return Err(StatusCode::UNAUTHORIZED);
            }

            Ok(next.run(request).await)
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
