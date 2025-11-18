---
title: Phase 4 Energy Tokenization - Implementation Guide
category: development
last_updated: 2025-11-09
status: in-progress
tags: [phase-4, energy-tokenization, minting, erc]
---

# Phase 4: Energy Tokenization - Implementation Guide

**Date**: November 9, 2025  
**Phase**: 4 - Energy Tokenization  
**Status**: In Progress  
**Database**: ✅ Migrations applied successfully

---

## 📋 Phase 4 Overview

Phase 4 enables the core energy tokenization workflow:
1. Users submit meter readings (kWh)
2. System validates and mints energy tokens (1 kWh = 1 token)
3. REC authorities issue Energy Renewable Certificates (ERCs)
4. Tokens can be traded in Phase 5

---

## ✅ Completed Work

### Database Schema (Nov 9, 2025)
- ✅ Migration `20241109000003_phase4_energy_tokenization.sql` applied
- ✅ Created `erc_certificates` table (20 columns, 5 indexes)
- ✅ Created `erc_certificate_transfers` table (7 columns, 3 indexes)
- ✅ All indexes created for optimal query performance

### Existing Handlers
- ✅ `api-gateway/src/handlers/token.rs` - Token endpoints partially ready
- ✅ `api-gateway/src/handlers/erc.rs` - ERC endpoints partially ready
- ✅ `api-gateway/src/handlers/meters.rs` - Meter reading handlers

### Build Status
- ✅ Zero compilation errors
- ⚠️  40 warnings (mostly unused imports/variables)

---

## 🚀 Implementation Roadmap

### STEP 1: Enhance Meter Reading Submission ⏳
**File**: `api-gateway/src/handlers/meters.rs`  
**Current**: `submit_reading()` exists but needs integration

#### Tasks:
1. [ ] Accept meter reading JSON: `{ meter_id, kwh_amount, timestamp, wallet_address }`
2. [ ] Validate input:
   - kwh_amount > 0
   - timestamp is recent (within 24 hours)
   - wallet_address is valid Solana pubkey
3. [ ] Store in database (already structured)
4. [ ] Return submission confirmation

#### Code Location:
```rust
// api-gateway/src/handlers/meters.rs:90-165
pub async fn submit_reading(...)  // READY
```

---

### STEP 2: Implement Token Minting Endpoint ⏳
**File**: `api-gateway/src/handlers/token.rs`  
**Current**: `mint_tokens()` exists as admin-only stub

#### Create New Function: `mint_from_meter_reading()`
```rust
/// Mint tokens from a verified meter reading
/// POST /api/tokens/mint-from-reading
/// Request: { reading_id: UUID, wallet_address: String }
/// Response: { success: bool, tx_signature: String, tokens_minted: u64 }

pub async fn mint_from_meter_reading(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<MintFromReadingRequest>,
) -> Result<Json<MintTokensResponse>> {
    // 1. Fetch meter reading from database
    let reading = sqlx::query!(
        "SELECT id, minted, wallet_address FROM meter_readings WHERE id = $1",
        payload.reading_id
    ).fetch_optional(&state.db)
     .await?
     .ok_or_else(|| ApiError::NotFound("Reading not found".to_string()))?;
    
    // 2. Check if already minted (double-claim prevention)
    if reading.minted {
        return Err(ApiError::BadRequest("Reading already minted".to_string()));
    }
    
    // 3. Call Energy Token program to mint tokens
    let amount_kwh = reading.kwh_amount;  // Already in kWh
    let tx_sig = state.blockchain_service.mint_energy_tokens(
        &user_keypair,
        &user_token_account,
        &token_mint,
        amount_kwh,
    ).await?;
    
    // 4. Update database: mark as minted
    sqlx::query!(
        "UPDATE meter_readings SET minted = TRUE, mint_tx_signature = $1, mint_timestamp = NOW() WHERE id = $2",
        tx_sig.to_string(),
        payload.reading_id
    ).execute(&state.db).await?;
    
    // 5. Return success response
    Ok(Json(MintTokensResponse {
        success: true,
        transaction_signature: Some(tx_sig.to_string()),
        tokens_minted: amount_kwh,
        reading_id: payload.reading_id,
    }))
}
```

#### Request/Response Types:
```rust
#[derive(Debug, Deserialize)]
pub struct MintFromReadingRequest {
    pub reading_id: Uuid,
    pub wallet_address: String,
}

#[derive(Debug, Serialize)]
pub struct MintTokensResponse {
    pub success: bool,
    pub transaction_signature: Option<String>,
    pub tokens_minted: BigDecimal,
    pub reading_id: Uuid,
}
```

#### Database Update:
```sql
UPDATE meter_readings 
SET minted = TRUE, 
    mint_tx_signature = 'tx_...',
    mint_timestamp = NOW()
WHERE id = $1;
```

---

### STEP 3: Implement Token Balance Endpoint ⏳
**File**: `api-gateway/src/handlers/token.rs`  
**Current**: `get_token_balance()` exists but incomplete

#### Enhance Function:
```rust
/// Get user's energy token balance
/// GET /api/tokens/balance/:wallet_address
pub async fn get_token_balance(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(wallet_address): Path<String>,
) -> Result<Json<TokenBalanceResponse>> {
    // 1. Validate wallet address
    let wallet = Pubkey::from_str(&wallet_address)?;
    
    // 2. Get token mint from blockchain
    let mint = state.blockchain_service.get_token_mint()?;
    
    // 3. Calculate associated token account
    let token_account = get_associated_token_address(&wallet, &mint);
    
    // 4. Fetch balance from blockchain
    let balance = state.blockchain_service.get_token_balance(&token_account)?;
    
    // 5. Return formatted response
    Ok(Json(TokenBalanceResponse {
        wallet_address,
        balance_tokens: balance,
        balance_kwh: balance,  // 1:1 ratio
        last_updated: Utc::now(),
    }))
}
```

---

### STEP 4: Implement Double-Claim Prevention ⏳
**File**: `api-gateway/src/handlers/meters.rs`

#### Add Validation Before Minting:
```rust
// Check if reading was already minted
let already_minted = sqlx::query!(
    "SELECT minted FROM meter_readings WHERE id = $1",
    reading_id
).fetch_optional(&state.db).await?;

if let Some(record) = already_minted {
    if record.minted {
        return Err(ApiError::BadRequest(
            "This meter reading has already been minted".to_string()
        ));
    }
}
```

#### Database Constraint (Already in schema):
```sql
CREATE UNIQUE INDEX idx_meter_readings_no_double_mint 
    ON meter_readings(user_id, reading_timestamp) 
    WHERE minted = true;
```

---

### STEP 5: Implement ERC Certificate Endpoints ⏳
**File**: `api-gateway/src/handlers/erc.rs`

#### Endpoint 1: Issue ERC Certificate
```rust
/// Issue ERC certificate for verified energy production
/// POST /api/erc/issue
pub async fn issue_certificate(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(request): Json<IssueErcRequest>,
) -> Result<Json<ErcCertificateResponse>> {
    // 1. Check REC authority role
    require_rec_authority(&user)?;
    
    // 2. Fetch user who produced the energy
    let user_record = sqlx::query!(
        "SELECT wallet_address FROM users WHERE id = $1",
        request.user_id
    ).fetch_one(&state.db).await?;
    
    // 3. Generate certificate ID (e.g., "ERC-2025-001")
    let cert_id = generate_certificate_id();
    
    // 4. Insert into database
    let certificate = sqlx::query_as!(
        ErcCertificate,
        r#"
        INSERT INTO erc_certificates (
            certificate_id, user_id, wallet_address, kwh_amount,
            issuer_wallet, issuer_name, status, metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active', $7)
        RETURNING *
        "#,
        cert_id, request.user_id, user_record.wallet_address,
        request.kwh_amount, user.wallet, user.name, metadata
    ).fetch_one(&state.db).await?;
    
    // 5. Optionally submit to blockchain (future)
    // let tx_sig = state.blockchain_service.issue_erc(...).await?;
    
    Ok(Json(certificate.into()))
}
```

#### Endpoint 2: Get User's Certificates
```rust
/// Get all ERC certificates for authenticated user
/// GET /api/erc/my-certificates?limit=50&offset=0
pub async fn get_my_certificates(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<ErcCertificateResponse>>> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);
    
    let certificates = sqlx::query_as!(
        ErcCertificate,
        r#"
        SELECT * FROM erc_certificates 
        WHERE user_id = $1
        ORDER BY issue_date DESC
        LIMIT $2 OFFSET $3
        "#,
        user.sub, limit, offset
    ).fetch_all(&state.db).await?;
    
    Ok(Json(certificates.into_iter()
        .map(|c| c.into())
        .collect()))
}
```

---

## 🔌 API Endpoints Summary

### Energy Token Endpoints

| Endpoint | Method | Auth | Status | Description |
|----------|--------|------|--------|-------------|
| `/api/tokens/balance/:wallet` | GET | Yes | 🔄 | Get token balance for wallet |
| `/api/tokens/mint-from-reading` | POST | Yes | 🔄 | Mint tokens from meter reading |
| `/api/tokens/info` | GET | Yes | ✅ | Get token program info |
| `/api/admin/tokens/mint` | POST | Admin | ✅ | Admin-only token minting |

### ERC Certificate Endpoints

| Endpoint | Method | Auth | Status | Description |
|----------|--------|------|--------|-------------|
| `/api/erc/issue` | POST | REC | ✅ | Issue ERC certificate |
| `/api/erc/my-certificates` | GET | Yes | ✅ | Get user's certificates |
| `/api/erc/:id` | GET | Yes | ✅ | Get certificate details |
| `/api/erc/transfer` | POST | Yes | ⏳ | Transfer certificate |

### Meter Reading Endpoints

| Endpoint | Method | Auth | Status | Description |
|----------|--------|------|--------|-------------|
| `/api/meters/submit-reading` | POST | Yes | ✅ | Submit meter reading |
| `/api/meters/my-readings` | GET | Yes | ✅ | Get user's readings |
| `/api/admin/meters/unminted` | GET | Admin | ✅ | Get unminted readings |

---

## 📊 Data Flow

```
1. User submits meter reading
   POST /api/meters/submit-reading
   → Stored in meter_readings table
   
2. System mints tokens
   POST /api/tokens/mint-from-reading
   → Energy Token program mints SPL tokens
   → meter_readings.minted = TRUE
   
3. REC authority issues certificate
   POST /api/erc/issue
   → ERC certificate created in database
   → Links to original meter reading
   
4. User checks balance
   GET /api/tokens/balance/:wallet
   → Queries blockchain for SPL balance
   → Returns token count (= kWh)
   
5. User views certificates
   GET /api/erc/my-certificates
   → Returns list of issued ERCs
   → Shows status and metadata
```

---

## 🧪 Testing Checklist

### Unit Tests
- [ ] Meter reading validation
- [ ] Double-claim prevention logic
- [ ] ERC certificate generation
- [ ] Token balance calculation

### Integration Tests
- [ ] Submit meter reading → verify in database
- [ ] Mint tokens → verify blockchain transaction
- [ ] Issue ERC → verify certificate created
- [ ] Check balance → verify correct value returned

### E2E Flow Test
- [ ] User registers
- [ ] User submits meter reading
- [ ] Tokens are minted
- [ ] Certificate is issued
- [ ] All reflected in UI

### Edge Cases
- [ ] Double submit same reading (should fail)
- [ ] Invalid wallet address (should fail)
- [ ] Negative kWh (should fail)
- [ ] Non-REC issuing certificate (should fail)

---

## 📝 Database Schema Reference

### meter_readings (existing, enhanced)
```sql
- id (UUID) - Primary key
- user_id (UUID) - Owner of meters
- wallet_address (VARCHAR) - Solana wallet
- kwh_amount (DECIMAL) - Energy generated
- timestamp (TIMESTAMPTZ) - Reading time
- minted (BOOLEAN) - Whether tokens created
- mint_tx_signature (VARCHAR) - Blockchain tx
- mint_timestamp (TIMESTAMPTZ) - When minted
- created_at (TIMESTAMPTZ)
- updated_at (TIMESTAMPTZ)
```

### erc_certificates (new)
```sql
- id (UUID) - Primary key
- certificate_id (VARCHAR) - ERC-2025-001 format
- user_id (UUID) - Certificate owner
- reading_id (UUID) - Associated meter reading
- wallet_address (VARCHAR) - Owner's wallet
- kwh_amount (DECIMAL) - Energy represented
- issue_date (TIMESTAMPTZ) - When issued
- issuer_wallet (VARCHAR) - REC authority
- issuer_name (VARCHAR) - Authority name
- status (VARCHAR) - active|transferred|retired
- blockchain_tx_signature (VARCHAR) - Blockchain tx
- metadata (JSONB) - Additional data
```

### erc_certificate_transfers (new)
```sql
- id (UUID)
- certificate_id (UUID) - Which certificate
- from_wallet (VARCHAR) - Previous owner
- to_wallet (VARCHAR) - New owner
- transfer_date (TIMESTAMPTZ)
- blockchain_tx_signature (VARCHAR)
```

---

## 🔧 Implementation Notes

### Solana Integration
- Energy Token program must be running
- Token mint must be initialized
- User token accounts (ATA) created beforehand

### Database
- Use transactions for atomic updates
- Prevent race conditions with WHERE clauses
- Use indexes for performance

### Error Handling
- Validate all inputs
- Return descriptive errors
- Log blockchain failures
- Handle network timeouts

---

## 📈 Success Criteria

Phase 4 is complete when:
- [ ] All 3 token endpoints working
- [ ] All 3 ERC endpoints working
- [ ] Double-claim prevention implemented
- [ ] Integration tests passing (>90% success)
- [ ] No critical bugs
- [ ] API response time <300ms average
- [ ] Documentation complete

---

## 🚀 Next Steps

After Phase 4:
1. **Phase 5**: Implement trading platform
2. **Phase 6**: Develop frontend UI
3. **Phase 7**: Add monitoring & analytics

---

**Status**: Phase 4 - In Progress  
**Last Updated**: November 9, 2025  
**Owner**: GridTokenX Development Team
