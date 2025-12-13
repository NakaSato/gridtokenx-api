// Metrics middleware for collecting API request metrics - Minimal version
// This middleware just adds request IDs and logs request duration

use axum::{
    extract::Request,
    http::{HeaderValue, Uri},
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{debug, error, warn};
use uuid::Uuid;

/// Request ID header name
const REQUEST_ID_HEADER: &str = "x-request-id";

/// Metrics middleware to collect API request metrics
pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();

    // Generate or extract request ID
    let request_id = extract_or_generate_request_id(&request);

    // Extract endpoint path without query parameters
    let path = extract_path_from_uri(&uri);

    // Log the incoming request
    debug!("Request started: {} {} (ID: {})", method, path, request_id);

    // Process the request
    let response = next.run(request).await;

    // Calculate request duration
    let duration = start_time.elapsed();

    // Get status code
    let status = response.status();
    let status_code = status.as_u16();

    // Log the completed request
    debug!(
        "Request completed: {} {} (ID: {}, Status: {}, Duration: {}ms)",
        method,
        path,
        request_id,
        status_code,
        duration.as_millis()
    );

    // Record slow requests as warnings
    if duration.as_millis() > 1000 {
        warn!(
            "Slow request detected: {} {} (ID: {}, Duration: {}ms)",
            method,
            path,
            request_id,
            duration.as_millis()
        );
    }

    // Record error requests with additional context
    if status_code >= 400 {
        error!(
            "Error response: {} {} (ID: {}, Status: {}, Duration: {}ms)",
            method,
            path,
            request_id,
            status_code,
            duration.as_millis()
        );
    }

    // Add request ID to response headers
    let (mut parts, body) = response.into_parts();
    parts.headers.insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&request_id).unwrap_or_else(|_| {
            error!("Failed to add request ID header");
            HeaderValue::from_static("unknown")
        }),
    );

    Response::from_parts(parts, body)
}

/// Extract existing request ID or generate a new one
fn extract_or_generate_request_id<B>(request: &Request<B>) -> String {
    // Try to extract from headers
    if let Some(header_value) = request.headers().get(REQUEST_ID_HEADER) {
        if let Ok(header_str) = header_value.to_str() {
            return header_str.to_string();
        }
    }

    // Generate a new UUID as request ID
    Uuid::new_v4().to_string()
}

/// Extract path from URI without query parameters
fn extract_path_from_uri(uri: &Uri) -> String {
    let path_and_query = uri.path_and_query();

    match path_and_query {
        Some(pq) => pq.path().to_string(),
        None => "/".to_string(),
    }
}
