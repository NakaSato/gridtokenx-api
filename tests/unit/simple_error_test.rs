#[cfg(test)]
mod tests {
    use api_gateway::error::{ApiError, ErrorCode};

    #[test]
    fn test_email_not_verified_error_code() {
        let error = ApiError::email_not_verified();
        
        // Test that the error has the correct code
        match error {
            ApiError::WithCode(code, _) => {
                assert_eq!(code, ErrorCode::EmailNotVerified);
            }
            _ => panic!("Expected WithCode error variant"),
        }
    }

    #[test]
    fn test_email_not_verified_status_code() {
        let error = ApiError::email_not_verified();
        
        // Test that the error returns 401 status code
        use axum::response::IntoResponse;
        let response = error.into_response();
        
        assert_eq!(response.status(), 401);
    }

    #[test]
    fn test_error_code_properties() {
        let code = ErrorCode::EmailNotVerified;
        
        assert_eq!(code.code(), 1005);
        assert_eq!(code.message(), "Please verify your email address before proceeding");
    }
}
