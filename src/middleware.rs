use axum::{extract::{Request, State}, http::StatusCode, middleware::Next, response::Response};

use crate::obj::Config;

pub async fn auth(State(config): State<Config>, req: Request, next: Next) -> Result<Response, StatusCode> {
    if config.auth.is_none() {
        return Ok(next.run(req).await);
    }
    let auth_header_r = req.headers().get("Authorization");
    if auth_header_r.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let auth_header = auth_header_r.unwrap();
    let auth_header_str_r = auth_header.to_str();
    if auth_header_str_r.is_err() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let auth_header_str = auth_header_str_r.unwrap();
    if auth_header_str != config.auth.unwrap() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(req).await)
}