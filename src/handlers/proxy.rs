use axum::{
    extract::{Request, State},
    response::{IntoResponse, Response},
    http::{StatusCode, Uri},
    body::Body,
};
use crate::app_state::AppState;
use std::str::FromStr;

pub async fn proxy_to_simulator(
    State(_state): State<AppState>,
    mut req: Request<Body>,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    // Target URL (Simulator)
    let target_url = format!("http://127.0.0.1:8000{}", path_query);

    // Create a new Reqwest client
    let client = reqwest::Client::new();

    // Create the new request
    let url = reqwest::Url::from_str(&target_url).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let mut request_builder = client.request(req.method().clone(), url);
    
    // Copy headers
    for (key, value) in req.headers() {
        request_builder = request_builder.header(key, value);
    }

    // Determine body
    // Convert axum Body to reqwest Body if necessary, or just forward bytes.
    // Simplifying for now assuming GET requests mostly, but for full proxy might need body handling.
    // For this specific use case (getting zones/topology), it's a GET request with no body.
    
    match request_builder.send().await {
        Ok(res) => {
            let status = res.status();
            let mut response_builder = Response::builder().status(status);
            
            // Copy response headers
            for (key, value) in res.headers() {
                response_builder = response_builder.header(key, value);
            }
            
            let bytes = res.bytes().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
            
            response_builder
                .body(Body::from(bytes))
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(_) => Err(StatusCode::BAD_GATEWAY),
    }
}
