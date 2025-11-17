---
title: Postman Collection Generation Plan
category: api-documentation
created: 2025-11-09
status: planning
tags: [postman, api, documentation, testing]
---

# Postman Collection Generation Plan for GridTokenX API Gateway

## 📋 Overview

This document outlines the comprehensive plan for generating a Postman collection from the GridTokenX API Gateway specifications. The collection will enable API testing, documentation, and client integration.

---

## 🎯 Objectives

1. **Automated Generation**: Create tooling to automatically generate Postman collections from Rust code
2. **Complete Coverage**: Include all 50+ API endpoints across all modules
3. **Authentication Support**: Pre-configure JWT and API key authentication
4. **Environment Variables**: Set up development, staging, and production environments
5. **Example Requests**: Provide sample request bodies and expected responses
6. **Testing Scripts**: Include Postman test scripts for automated validation

---

## 📊 Current API Inventory

### Endpoint Breakdown by Module

Based on analysis of `/api-gateway/src/main.rs`:

```
Total Endpoints: ~55+

Public Endpoints (No Auth):
├── Health Check (1)
├── Authentication (4)
│   ├── POST /auth/login
│   ├── POST /auth/register
│   ├── POST /auth/wallet/login
│   └── POST /auth/wallet/register
└── Email Verification (2)
    ├── GET  /auth/verify-email
    └── POST /auth/resend-verification

Protected User Endpoints (JWT Required):
├── User Profile (3)
│   ├── GET  /auth/profile
│   ├── POST /auth/profile
│   └── POST /auth/password
├── User Management (4)
│   ├── POST   /user/wallet
│   ├── DELETE /user/wallet
│   └── GET    /user/activity
├── Blockchain (6)
│   ├── POST /blockchain/transactions
│   ├── GET  /blockchain/transactions
│   ├── GET  /blockchain/transactions/:signature
│   ├── POST /blockchain/programs/:name
│   ├── GET  /blockchain/accounts/:address
│   ├── GET  /blockchain/network
│   └── GET  /blockchain/users/:wallet_address
├── Oracle (3)
│   ├── POST /oracle/prices
│   ├── GET  /oracle/prices/current
│   └── GET  /oracle/data
├── Governance (1)
│   └── GET  /governance/status
├── Trading (6)
│   ├── POST /trading/orders
│   ├── GET  /trading/orders
│   ├── GET  /trading/market
│   ├── GET  /trading/stats
│   ├── GET  /trading/market/blockchain
│   └── POST /trading/orders/blockchain
├── Tokens (2)
│   ├── GET /tokens/balance/:wallet_address
│   └── GET /tokens/info
├── Energy Meters (8)
│   ├── POST /meters/submit-reading
│   ├── GET  /meters/my-readings
│   ├── GET  /meters/readings/:wallet_address
│   ├── GET  /meters/stats
│   ├── POST /meters/readings
│   ├── GET  /meters/readings
│   ├── GET  /meters/readings/:id
│   └── GET  /meters/aggregated
└── ERC (6)
    ├── POST /erc/issue
    ├── GET  /erc/my-certificates
    ├── GET  /erc/my-stats
    ├── GET  /erc/:certificate_id
    ├── POST /erc/:certificate_id/retire
    └── GET  /erc/wallet/:wallet_address

Admin-Only Endpoints (Admin Role Required):
├── User Management (5)
│   ├── GET    /users/:id
│   ├── PUT    /users/:id
│   ├── POST   /users/:id/deactivate
│   ├── POST   /users/:id/reactivate
│   ├── GET    /users/:id/activity
│   └── GET    /users/
├── Registry Admin (1)
│   └── POST /admin/users/:id/update-role
├── Governance Admin (2)
│   ├── POST /admin/governance/emergency-pause
│   └── POST /admin/governance/unpause
├── Token Admin (1)
│   └── POST /admin/tokens/mint
├── Trading Admin (1)
│   └── POST /admin/trading/match-orders
└── Meter Admin (2)
    ├── GET  /admin/meters/unminted
    └── POST /admin/meters/mint-from-reading
```

---

## 🛠️ Implementation Approach

### Option 1: Manual Collection Creation ✅ (Recommended for MVP)

**Timeline**: 2-3 days  
**Effort**: Low-Medium  
**Maintenance**: Manual updates needed

**Approach**:
1. Create Postman collection manually
2. Organize by module/feature
3. Add pre-request scripts for authentication
4. Include example requests/responses
5. Export as JSON

**Pros**:
- Quick to implement
- Full control over structure
- Easy to customize
- No additional tooling needed

**Cons**:
- Manual maintenance required
- Risk of drift from code
- Time-consuming for updates

### Option 2: OpenAPI/Swagger Generation 🎯 (Recommended for Production)

**Timeline**: 1-2 weeks  
**Effort**: Medium-High  
**Maintenance**: Automated

**Approach**:
1. Add OpenAPI annotations to Rust handlers using `utoipa` crate
2. Generate OpenAPI spec automatically
3. Convert OpenAPI → Postman using tools
4. Integrate into CI/CD pipeline

**Pros**:
- Automated generation
- Always up-to-date with code
- Industry-standard format
- Can generate docs + Postman

**Cons**:
- Initial setup time
- Requires code changes
- Learning curve for team

### Option 3: Custom Code Parser

**Timeline**: 3-4 weeks  
**Effort**: High  
**Maintenance**: Automated

**Approach**:
1. Write Rust parser to extract routes from `main.rs`
2. Parse handler files for request/response types
3. Generate Postman JSON directly
4. CLI tool for generation

**Pros**:
- Fully customized to our needs
- No external dependencies
- Complete control

**Cons**:
- Significant development time
- Maintenance overhead
- Reinventing the wheel

---

## 📝 Detailed Implementation Plan (Option 1 - MVP)

### Phase 1: Collection Structure Setup (Day 1)

**Tasks**:
1. Create main Postman collection: "GridTokenX API Gateway"
2. Set up folder structure:
   ```
   GridTokenX API Gateway/
   ├── 00. Health & Status
   ├── 01. Authentication
   ├── 02. User Management
   ├── 03. Blockchain
   ├── 04. Oracle
   ├── 05. Governance
   ├── 06. Trading
   ├── 07. Tokens
   ├── 08. Energy Meters
   ├── 09. ERC (Certificates)
   └── 10. Admin Operations
   ```

3. Create environments:
   - **Local Development**
     ```json
     {
       "base_url": "http://localhost:8080",
       "jwt_token": "",
       "admin_token": "",
       "wallet_address": "",
       "user_id": ""
     }
     ```
   - **Production**
     ```json
     {
       "base_url": "https://api.gridtokenx.com",
       "jwt_token": "",
       "admin_token": "",
       "wallet_address": "",
       "user_id": ""
     }
     ```

### Phase 2: Core Endpoints (Day 1-2)

#### 2.1 Health Check
```
GET {{base_url}}/health
```

#### 2.2 Authentication Endpoints
```
POST {{base_url}}/auth/login
POST {{base_url}}/auth/register
POST {{base_url}}/auth/wallet/login
POST {{base_url}}/auth/wallet/register
```

**Example Request Body** (`/auth/login`):
```json
{
  "username": "testuser",
  "password": "SecurePass123!"
}
```

**Example Response**:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user": {
    "id": "uuid",
    "username": "testuser",
    "email": "test@example.com",
    "role": "consumer"
  }
}
```

**Post-Response Script**:
```javascript
if (pm.response.code === 200) {
    const response = pm.response.json();
    pm.environment.set("jwt_token", response.token);
    pm.environment.set("user_id", response.user.id);
    console.log("Token saved:", response.token);
}
```

#### 2.3 Protected Endpoints Setup

**Pre-Request Script** (Collection level):
```javascript
const token = pm.environment.get("jwt_token");
if (token) {
    pm.request.headers.add({
        key: "Authorization",
        value: `Bearer ${token}`
    });
}
```

### Phase 3: Feature-Specific Endpoints (Day 2)

#### 3.1 User Management
- GET /auth/profile
- POST /auth/profile
- POST /auth/password
- POST /user/wallet
- DELETE /user/wallet
- GET /user/activity

#### 3.2 Blockchain
- POST /blockchain/transactions
- GET /blockchain/transactions
- GET /blockchain/transactions/:signature
- POST /blockchain/programs/:name
- GET /blockchain/accounts/:address
- GET /blockchain/network
- GET /blockchain/users/:wallet_address

#### 3.3 Oracle
- POST /oracle/prices
- GET /oracle/prices/current
- GET /oracle/data

#### 3.4 Governance
- GET /governance/status

#### 3.5 Trading
- POST /trading/orders
- GET /trading/orders
- GET /trading/market
- GET /trading/stats
- GET /trading/market/blockchain
- POST /trading/orders/blockchain

#### 3.6 Tokens
- GET /tokens/balance/:wallet_address
- GET /tokens/info

#### 3.7 Energy Meters
- POST /meters/submit-reading
- GET /meters/my-readings
- GET /meters/readings/:wallet_address
- GET /meters/stats

#### 3.8 ERC (Certificates)
- POST /erc/issue
- GET /erc/my-certificates
- GET /erc/my-stats
- GET /erc/:certificate_id
- POST /erc/:certificate_id/retire
- GET /erc/wallet/:wallet_address

### Phase 4: Admin Endpoints (Day 3)

#### 4.1 Admin User Management
- GET /users/:id
- PUT /users/:id
- POST /users/:id/deactivate
- POST /users/:id/reactivate
- GET /users/:id/activity
- GET /users/

#### 4.2 Admin Operations
- POST /admin/users/:id/update-role
- POST /admin/governance/emergency-pause
- POST /admin/governance/unpause
- POST /admin/tokens/mint
- POST /admin/trading/match-orders
- GET /admin/meters/unminted
- POST /admin/meters/mint-from-reading

**Admin Pre-Request Script**:
```javascript
const adminToken = pm.environment.get("admin_token");
if (!adminToken) {
    // Try to use regular token if admin token not set
    const token = pm.environment.get("jwt_token");
    if (token) {
        pm.request.headers.add({
            key: "Authorization",
            value: `Bearer ${token}`
        });
    }
} else {
    pm.request.headers.add({
        key: "Authorization",
        value: `Bearer ${adminToken}`
    });
}
```

### Phase 5: Test Scripts & Validation (Day 3)

**Generic Success Test**:
```javascript
pm.test("Status code is 200", function () {
    pm.response.to.have.status(200);
});

pm.test("Response time is acceptable", function () {
    pm.expect(pm.response.responseTime).to.be.below(2000);
});

pm.test("Response has valid JSON", function () {
    pm.response.to.be.json;
});
```

**Authentication Test**:
```javascript
pm.test("Returns JWT token", function () {
    const response = pm.response.json();
    pm.expect(response).to.have.property("token");
    pm.expect(response.token).to.be.a("string");
    pm.expect(response.token.length).to.be.above(20);
});
```

**User Data Test**:
```javascript
pm.test("Returns user information", function () {
    const response = pm.response.json();
    pm.expect(response).to.have.property("user");
    pm.expect(response.user).to.have.property("id");
    pm.expect(response.user).to.have.property("username");
    pm.expect(response.user).to.have.property("email");
    pm.expect(response.user).to.have.property("role");
});
```

### Phase 6: Documentation & Examples (Day 3)

For each endpoint, add:
1. **Description**: What the endpoint does
2. **Authentication**: Required role/token
3. **Request Body**: Schema and example
4. **Query Parameters**: If applicable
5. **Path Variables**: If applicable
6. **Response Body**: Expected schema and example
7. **Error Responses**: Common error cases

**Example Documentation Template**:
```markdown
### Submit Energy Reading

**Endpoint**: `POST /meters/submit-reading`

**Description**: Submit a new energy meter reading for token minting.

**Authentication**: JWT token required (Prosumer role)

**Request Body**:
```json
{
  "meter_id": "string (UUID)",
  "reading_kwh": "number (decimal)",
  "timestamp": "string (ISO 8601)",
  "metadata": {
    "location": "string (optional)",
    "meter_type": "string (optional)"
  }
}
```

**Success Response** (200):
```json
{
  "reading_id": "uuid",
  "tokens_pending": 125.5,
  "status": "pending_mint",
  "created_at": "2025-11-09T12:00:00Z"
}
```

**Error Responses**:
- 400: Invalid reading data
- 401: Not authenticated
- 403: Not authorized (wrong role)
- 500: Server error
```

---

## 🚀 Alternative: OpenAPI Implementation Plan (Option 2)

### Phase 1: Setup utoipa (Week 1)

**Dependencies to add** (`Cargo.toml`):
```toml
[dependencies]
utoipa = { version = "4", features = ["axum_extras", "uuid"] }
utoipa-swagger-ui = { version = "6", features = ["axum"] }
```

### Phase 2: Annotate Handlers (Week 1-2)

**Example Handler Annotation**:
```rust
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, Validate, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "testuser", min_length = 3, max_length = 50)]
    pub username: String,
    
    #[schema(example = "SecurePass123!", min_length = 8, max_length = 128)]
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = SecureAuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<SecureAuthResponse>> {
    // ... implementation
}
```

### Phase 3: Generate OpenAPI Spec

**Create OpenAPI route**:
```rust
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        auth::login,
        auth::get_profile,
        // ... all endpoints
    ),
    components(
        schemas(LoginRequest, SecureAuthResponse, /* ... */)
    ),
    tags(
        (name = "Authentication", description = "User authentication endpoints"),
        (name = "Blockchain", description = "Blockchain interaction endpoints"),
        // ... more tags
    )
)]
struct ApiDoc;

// In main.rs
let app = Router::new()
    .merge(SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", ApiDoc::openapi()))
    // ... rest of routes
```

### Phase 4: Convert to Postman

**Using openapi-to-postman CLI**:
```bash
npm install -g openapi-to-postman

# Generate Postman collection
openapi-to-postman -s http://localhost:8080/api-docs/openapi.json \
    -o postman-collection.json \
    -p
```

---

## 📦 Deliverables

### Immediate (MVP - Option 1)
1. ✅ Postman Collection JSON file
2. ✅ Environment files (local, staging, prod)
3. ✅ README with setup instructions
4. ✅ Example requests for all endpoints
5. ✅ Basic test scripts

### Future (Production - Option 2)
1. ✅ OpenAPI specification (openapi.json)
2. ✅ Swagger UI integration
3. ✅ Automated Postman generation
4. ✅ CI/CD integration
5. ✅ Auto-published documentation

---

## 📁 File Structure

```
api-gateway/
├── docs/
│   ├── POSTMAN_GENERATION_PLAN.md (this file)
│   └── POSTMAN_SETUP_GUIDE.md (user guide)
├── postman/
│   ├── GridTokenX-API-Gateway.postman_collection.json
│   ├── Local-Development.postman_environment.json
│   ├── Production.postman_environment.json
│   └── README.md
└── scripts/
    ├── generate-postman.sh (for Option 2)
    └── update-openapi.sh (for Option 2)
```

---

## 🎯 Success Criteria

### MVP Success (Option 1)
- [ ] All 55+ endpoints documented
- [ ] Authentication working in Postman
- [ ] Environment variables configured (local, production)
- [ ] Basic test scripts included
- [ ] Documentation for setup

### Production Success (Option 2)
- [ ] OpenAPI spec generated
- [ ] Swagger UI accessible
- [ ] Postman auto-generated
- [ ] CI/CD integration
- [ ] Version control for API changes

---

## ⏱️ Timeline Estimate

### Option 1 (Manual - MVP)
- **Day 1**: Setup + Health/Auth endpoints (6 hours)
- **Day 2**: Feature endpoints (8 hours)
- **Day 3**: Admin + Tests + Docs (6 hours)
- **Total**: 3 days (~20 hours)

### Option 2 (OpenAPI - Production)
- **Week 1**: Setup utoipa + annotate core endpoints (20 hours)
- **Week 2**: Complete annotations + generate specs (20 hours)
- **Total**: 2 weeks (~40 hours)

---

## 🚦 Next Steps

1. **Immediate** (This week):
   - [ ] Review and approve this plan
   - [ ] Decide on approach (Option 1 for MVP, Option 2 for future)
   - [ ] Create `postman/` directory structure
   - [ ] Start with Option 1 implementation

2. **Short-term** (Next sprint):
   - [ ] Complete Option 1 MVP
   - [ ] Test collection with real API
   - [ ] Share with team for feedback
   - [ ] Document any issues

3. **Long-term** (Q1 2026):
   - [ ] Plan migration to Option 2 (OpenAPI)
   - [ ] Add to Stage 2 roadmap
   - [ ] Budget time for utoipa integration
   - [ ] Set up CI/CD for auto-generation

---

## 📚 Resources

### Postman
- [Postman Collection Format](https://learning.postman.com/collection-format/)
- [Postman Pre-request Scripts](https://learning.postman.com/docs/writing-scripts/pre-request-scripts/)
- [Postman Tests](https://learning.postman.com/docs/writing-scripts/test-scripts/)

### OpenAPI
- [utoipa Documentation](https://docs.rs/utoipa/)
- [OpenAPI to Postman Converter](https://github.com/postmanlabs/openapi-to-postman)
- [OpenAPI 3.0 Specification](https://swagger.io/specification/)

### Rust Integration
- [utoipa Examples](https://github.com/juhaku/utoipa/tree/master/examples)
- [Axum + utoipa Integration](https://github.com/juhaku/utoipa/blob/master/examples/todo-axum/)

---

**Document Version**: 1.0  
**Created**: November 9, 2025  
**Status**: Planning  
**Next Review**: After MVP completion

---

*This plan aligns with Stage 1 objectives and will be implemented during December 2025.*
