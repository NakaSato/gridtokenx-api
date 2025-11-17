# OpenAPI Implementation Plan

## Overview
This document outlines the implementation of OpenAPI documentation using `utoipa` for the GridTokenX Platform API Gateway.

## Status
✅ Completed:
1. Added `utoipa` and `utoipa-swagger-ui` dependencies to Cargo.toml
2. Created OpenAPI module structure (`src/openapi/mod.rs`)
3. Added `ToSchema` derives to database schema types
4. Added `ToSchema` derives to models (user, blockchain, trading, energy)
5. Added `#[utoipa::path]` to health check handler

🚧 In Progress:
6. Adding `ToSchema` and `#[utoipa::path]` to all handlers

📋 Remaining Tasks:

### Handler Files Needing Documentation

#### Authentication & User Management
- [ ] `src/handlers/auth.rs` - Add ToSchema to LoginRequest, LoginResponse, UpdateProfileRequest, ChangePasswordRequest
- [ ] `src/handlers/user_management.rs` - Document all request/response types
- [ ] `src/handlers/email_verification.rs` - Document verification endpoints
- [ ] `src/handlers/wallet_auth.rs` - Document wallet-based auth

#### Blockchain Operations
- [ ] `src/handlers/blockchain.rs` - Document transaction and account endpoints
- [ ] `src/handlers/blockchain_test.rs` - Document testing endpoints
- [ ] `src/handlers/registry.rs` - Document user registry operations

#### Trading & Energy
- [ ] `src/handlers/trading.rs` - Document trading operations
- [ ] `src/handlers/meters.rs` - Document meter reading operations
- [ ] `src/handlers/erc.rs` - Document certificate operations
- [ ] `src/handlers/token.rs` - Document token operations

#### Oracle & Governance
- [ ] `src/handlers/oracle.rs` - Document price oracle operations
- [ ] `src/handlers/governance.rs` - Document governance operations

### Integration Steps

1. **Add OpenAPI module to main.rs**
   ```rust
   mod openapi;
   use utoipa_swagger_ui::SwaggerUi;
   ```

2. **Add Swagger UI routes**
   ```rust
   let app = Router::new()
       .merge(SwaggerUi::new("/api/docs")
           .url("/api/docs/openapi.json", openapi::ApiDoc::openapi()))
       // ... rest of routes
   ```

3. **Add ToSchema to all request/response types**
   - Each handler file needs ToSchema imports
   - Add #[derive(ToSchema)] to all request/response structs
   - Add #[utoipa::path(...)] macros to all public handlers

4. **Document security requirements**
   - JWT Bearer token authentication
   - Configure security schemes in OpenAPI struct

## Example Pattern

### For Handler Functions:
```rust
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ApiError),
        (status = 400, description = "Validation error", body = ApiError)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<LoginResponse>> {
    // ... implementation
}
```

### For Request/Response Types:
```rust
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}
```

## Testing

After implementation:
1. Start the server: `cargo run --bin api-gateway`
2. Access Swagger UI: `http://localhost:8080/api/docs`
3. Verify all endpoints are documented
4. Test authentication flows with JWT tokens
5. Validate response schemas match actual responses

## Benefits

- **Automatic Documentation**: Keep API docs in sync with code
- **Interactive Testing**: Use Swagger UI for manual testing
- **Type Safety**: Compile-time validation of schemas
- **Client Generation**: Generate client SDKs from OpenAPI spec
- **Developer Experience**: Integrated documentation for all endpoints

## Next Steps

1. Complete handler documentation (in order of priority):
   - Authentication & user management (critical)
   - Trading & blockchain (high)
   - Meters & tokens (medium)
   - Oracle & governance (low)

2. Add Swagger UI endpoint to main.rs
3. Test all documented endpoints
4. Add examples to common request/response types
5. Document query parameters and path parameters
6. Add error response documentation
