# GridTokenX API - OpenAPI Documentation Quick Start

## What We've Done

### ✅ Completed Setup

1. **Added Dependencies** (`Cargo.toml`)
   - `utoipa = { version = "5", features = ["axum_extras", "uuid", "chrono"] }`
   - `utoipa-swagger-ui = { version = "8", features = ["axum"] }`

2. **Created OpenAPI Module** (`src/openapi/mod.rs`)
   - Defined API metadata (title, version, description)
   - Configured security schemes (JWT Bearer token)
   - Set up tags for API organization
   - Registered the health check endpoint
   - Registered all data models

3. **Added ToSchema Derives**
   - ✅ Database schema types (`src/database/schema.rs`)
   - ✅ User models (`src/models/user.rs`)
   - ✅ Blockchain models (`src/models/blockchain.rs`)
   - ✅ Trading models (`src/models/trading.rs`)
   - ✅ Energy models (`src/models/energy.rs`)
   - ✅ Health check models (`src/handlers/health.rs`)

4. **Integrated Swagger UI** (`src/main.rs`)
   - Added OpenAPI module import
   - Mounted Swagger UI at `/api/docs`
   - OpenAPI JSON available at `/api/docs/openapi.json`

5. **Documented First Endpoint**
   - ✅ Health check endpoint with `#[utoipa::path]`

## Access the Documentation

Once the server is running:

```bash
# Start the API Gateway
cargo run --bin api-gateway

# Access Swagger UI
open http://localhost:8080/api/docs

# Get OpenAPI JSON spec
curl http://localhost:8080/api/docs/openapi.json
```

## Next Steps: Complete Handler Documentation

### Priority 1: Authentication & User Management

Add `#[utoipa::path]` annotations to these files:

#### `src/handlers/auth.rs`
```rust
use utoipa::ToSchema;

// Add ToSchema to request/response types
#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct LoginRequest { /* ... */ }

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse { /* ... */ }

// Add path documentation
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 400, description = "Validation error")
    )
)]
pub async fn login(/* ... */) -> Result<Json<LoginResponse>> {
    // implementation
}
```

#### `src/handlers/user_management.rs`
Similar pattern for:
- `register`
- `update_wallet_address`
- `remove_wallet_address`
- `get_user_activity`
- `admin_update_user`
- `admin_deactivate_user`
- `admin_reactivate_user`

### Priority 2: Blockchain Operations

Document these handlers:
- `src/handlers/blockchain.rs`
- `src/handlers/blockchain_test.rs`
- `src/handlers/registry.rs`

### Priority 3: Trading & Energy

Document these handlers:
- `src/handlers/trading.rs`
- `src/handlers/meters.rs`
- `src/handlers/erc.rs`
- `src/handlers/token.rs`

### Priority 4: Oracle & Governance

Document these handlers:
- `src/handlers/oracle.rs`
- `src/handlers/governance.rs`

### Priority 5: Email & Wallet Auth

Document these handlers:
- `src/handlers/email_verification.rs`
- `src/handlers/wallet_auth.rs`

## Documentation Pattern

For each handler function, follow this pattern:

```rust
#[utoipa::path(
    {method},                          // get, post, put, delete, etc.
    path = "{full_path}",             // e.g., "/api/trading/orders"
    tag = "{category}",               // e.g., "trading"
    request_body = {RequestType},     // Optional, for POST/PUT
    params(                           // Optional, for path/query params
        ("id" = Uuid, Path, description = "User ID"),
        ("page" = Option<u32>, Query, description = "Page number")
    ),
    responses(
        (status = 200, description = "Success", body = {ResponseType}),
        (status = 404, description = "Not found"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Bad request")
    ),
    security(                         // Optional, for protected endpoints
        ("bearer_auth" = [])
    )
)]
pub async fn handler_function(/* ... */) -> Result<Json<ResponseType>> {
    // implementation
}
```

## Update OpenAPI Module

After documenting handlers, add them to `src/openapi/mod.rs`:

```rust
paths(
    // Health
    crate::handlers::health::health_check,
    
    // Add newly documented handlers here
    crate::handlers::auth::login,
    crate::handlers::auth::register,
    // etc...
),
components(schemas(
    // Add newly created request/response types here
    crate::handlers::auth::LoginRequest,
    crate::handlers::auth::LoginResponse,
    // etc...
)),
```

## Benefits

✅ **Automatic Documentation**: API docs stay in sync with code  
✅ **Interactive Testing**: Test endpoints directly from Swagger UI  
✅ **Type Safety**: Compile-time validation of schemas  
✅ **Client Generation**: Generate SDKs from OpenAPI spec  
✅ **Better DX**: Clear API contracts for developers

## Common Issues

### SQLx Compile-Time Verification
If you see database connection errors during compilation, you can:
1. Start the database before building
2. Use offline mode: `cargo sqlx prepare` then build with prepared queries
3. Set `SQLX_OFFLINE=true` environment variable

### Missing Schemas
If compilation fails with "cannot find type X in scope":
1. Make sure the type has `#[derive(ToSchema)]`
2. Import `use utoipa::ToSchema;` at the top of the file
3. Add the type to the `components(schemas(...))` section

### Path Not Found
If a handler isn't showing in Swagger UI:
1. Check that `#[utoipa::path]` is on the handler function
2. Add the handler to the `paths(...)` section in `src/openapi/mod.rs`
3. Rebuild and restart the server

## Testing Checklist

- [ ] Server starts without errors
- [ ] Swagger UI accessible at http://localhost:8080/api/docs
- [ ] All tags appear in Swagger UI sidebar
- [ ] All endpoints are listed under correct tags
- [ ] Request/response schemas are complete
- [ ] Authentication works (JWT Bearer token)
- [ ] Example requests work correctly
- [ ] All error responses are documented

## Resources

- [utoipa Documentation](https://docs.rs/utoipa/latest/utoipa/)
- [utoipa-swagger-ui Documentation](https://docs.rs/utoipa-swagger-ui/latest/utoipa_swagger_ui/)
- [OpenAPI 3.1 Specification](https://spec.openapis.org/oas/latest.html)
- [Axum Integration Guide](https://docs.rs/utoipa/latest/utoipa/attr.path.html#axum_extras-feature-support-for-axum)

## Summary

We've successfully set up the foundation for OpenAPI documentation in your API Gateway:
- ✅ Dependencies installed
- ✅ OpenAPI module created
- ✅ Swagger UI integrated at `/api/docs`
- ✅ All data models have ToSchema
- ✅ First endpoint (health check) documented
- 📋 Remaining: Document ~60 handler functions

The infrastructure is ready. Now it's a matter of adding `#[utoipa::path]` annotations to each handler function and their request/response types!
