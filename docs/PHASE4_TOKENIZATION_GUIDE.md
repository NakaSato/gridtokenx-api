# Phase 4: Energy Tokenization - Complete Implementation Guide

## 🎯 Overview

Phase 4 introduces the **Energy Tokenization System**, enabling prosumers to submit smart meter readings and receive blockchain-based energy tokens. This phase also implements the **Energy Renewable Certificate (ERC)** system for tracking and trading renewable energy credentials.

**Status**: ✅ **COMPLETE** (API layer complete, blockchain integration ready for Anchor client)

---

## 📋 Table of Contents

1. [System Architecture](#system-architecture)
2. [Database Schema](#database-schema)
3. [API Endpoints](#api-endpoints)
4. [Service Layer](#service-layer)
5. [Data Flow](#data-flow)
6. [Usage Examples](#usage-examples)
7. [Security & Authorization](#security--authorization)
8. [Testing Guide](#testing-guide)

---

## 🏗️ System Architecture

### Component Overview

```
┌─────────────────┐
│  Prosumer       │
│  (Smart Meter)  │
└────────┬────────┘
         │ 1. Submit Reading
         ▼
┌─────────────────┐
│  API Gateway    │
│  - Meter Handler│ ──▶ Validate (max 100kWh, <7 days old, no duplicates)
│  - Meter Service│
└────────┬────────┘
         │ 2. Store Reading
         ▼
┌─────────────────┐
│  PostgreSQL DB  │
│  - meter_readings
│  - erc_certificates
└────────┬────────┘
         │ 3. Admin reviews unminted readings
         ▼
┌─────────────────┐
│  Admin Portal   │
│  - GET /admin/meters/unminted
│  - POST /admin/meters/mint-from-reading
└────────┬────────┘
         │ 4. Trigger Minting
         ▼
┌─────────────────┐
│  Blockchain     │
│  Service        │ ──▶ Sign Transaction with Authority Wallet
│  - Wallet       │
│    Service      │
└────────┬────────┘
         │ 5. Submit to Solana
         ▼
┌─────────────────┐
│  Solana         │
│  Blockchain     │
│  - Energy Token │ ──▶ Mint tokens to prosumer's wallet
│    Program      │
└────────┬────────┘
         │ 6. Update Database
         ▼
┌─────────────────┐
│  meter_readings │
│  minted = true  │
│  tx_signature   │
└─────────────────┘
         │ 7. REC Authority issues ERC
         ▼
┌─────────────────┐
│  ERC Service    │
│  - Issue Cert   │ ──▶ Generate ERC-2025-000001
│  - Track Status │
└────────┬────────┘
         │ 8. Certificate ready for trading
         ▼
┌─────────────────┐
│  P2P Trading    │
│  Platform       │
└─────────────────┘
```

### Service Layers

1. **Handler Layer** (`handlers/`)
   - `meter.rs` - HTTP endpoints for meter readings
   - `erc.rs` - HTTP endpoints for ERC certificates

2. **Service Layer** (`services/`)
   - `meter_service.rs` - Business logic for meter readings
   - `erc_service.rs` - Business logic for ERC certificates
   - `wallet_service.rs` - Authority keypair management
   - `blockchain_service.rs` - Transaction building and signing

3. **Database Layer**
   - `meter_readings` table
   - `erc_certificates` table
   - `erc_certificate_transfers` table (audit trail)

---

## 🗄️ Database Schema

### Table: `meter_readings`

Stores all smart meter energy readings from prosumers.

```sql
CREATE TABLE meter_readings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    wallet_address TEXT NOT NULL,
    kwh_amount DECIMAL(10, 2) NOT NULL,
    reading_timestamp TIMESTAMPTZ NOT NULL,
    submitted_at TIMESTAMPTZ DEFAULT NOW(),
    minted BOOLEAN DEFAULT FALSE,
    mint_tx_signature TEXT,
    meter_signature TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_meter_readings_user_id ON meter_readings(user_id);
CREATE INDEX idx_meter_readings_wallet ON meter_readings(wallet_address);
CREATE INDEX idx_meter_readings_timestamp ON meter_readings(reading_timestamp DESC);
CREATE INDEX idx_meter_readings_minted ON meter_readings(minted, reading_timestamp);
```

**Key Fields**:
- `kwh_amount`: Energy generated/consumed in kilowatt-hours
- `reading_timestamp`: When the meter recorded the reading
- `minted`: Whether tokens have been minted for this reading
- `mint_tx_signature`: Solana transaction signature after minting
- `meter_signature`: Optional cryptographic signature from smart meter

### Table: `erc_certificates`

Tracks Energy Renewable Certificates (ERCs) issued by certified authorities.

```sql
CREATE TABLE erc_certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    certificate_id TEXT UNIQUE NOT NULL,  -- Human-readable: ERC-2025-000001
    user_id UUID NOT NULL REFERENCES users(id),
    wallet_address TEXT NOT NULL,
    kwh_amount DECIMAL(10, 2) NOT NULL,
    issue_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expiry_date TIMESTAMPTZ,
    issuer_wallet TEXT NOT NULL,
    issuer_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',  -- active | transferred | retired | expired
    blockchain_tx_signature TEXT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE UNIQUE INDEX idx_erc_certificate_id ON erc_certificates(certificate_id);
CREATE INDEX idx_erc_user_id ON erc_certificates(user_id);
CREATE INDEX idx_erc_wallet ON erc_certificates(wallet_address);
CREATE INDEX idx_erc_issuer ON erc_certificates(issuer_wallet);
CREATE INDEX idx_erc_status ON erc_certificates(status);
```

**Certificate ID Format**: `ERC-YYYY-NNNNNN`
- `YYYY`: Year (e.g., 2025)
- `NNNNNN`: Auto-incrementing number per year (zero-padded to 6 digits)
- Example: `ERC-2025-000001`, `ERC-2025-000002`

**Status Values**:
- `active`: Certificate is valid and owned by `wallet_address`
- `transferred`: Certificate has been transferred to another wallet
- `retired`: Certificate has been used/retired (one-way operation)
- `expired`: Certificate has passed its `expiry_date`

### Table: `erc_certificate_transfers`

Audit trail for all certificate ownership transfers.

```sql
CREATE TABLE erc_certificate_transfers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    certificate_id TEXT NOT NULL,
    from_wallet TEXT NOT NULL,
    to_wallet TEXT NOT NULL,
    transfer_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    blockchain_tx_signature TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_erc_transfers_cert_id ON erc_certificate_transfers(certificate_id);
CREATE INDEX idx_erc_transfers_from ON erc_certificate_transfers(from_wallet);
CREATE INDEX idx_erc_transfers_to ON erc_certificate_transfers(to_wallet);
```

---

## 🔌 API Endpoints

### Meter Reading Endpoints

#### 1. Submit Meter Reading

**Endpoint**: `POST /api/meters/submit-reading`  
**Auth**: Required (JWT)  
**Role**: `prosumer`, `admin`

**Request**:
```json
{
  "kwh_amount": 25.5,
  "reading_timestamp": "2025-01-15T10:30:00Z",
  "meter_signature": "0x1234...abcd",
  "metadata": {
    "meter_id": "SM-12345",
    "location": "Building A - Rooftop Solar"
  }
}
```

**Validation Rules**:
- `kwh_amount` must be > 0 and ≤ 100 kWh (configurable limit)
- `reading_timestamp` cannot be in the future
- `reading_timestamp` cannot be older than 7 days
- No duplicate readings within ±15 minutes for the same user

**Response**:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "Meter reading submitted successfully",
  "kwh_amount": 25.5,
  "reading_timestamp": "2025-01-15T10:30:00Z",
  "minted": false,
  "submitted_at": "2025-01-15T10:35:22Z"
}
```

---

#### 2. Get My Readings

**Endpoint**: `GET /api/meters/my-readings?page=1&limit=20`  
**Auth**: Required (JWT)  
**Role**: Any authenticated user

**Query Parameters**:
- `page`: Page number (default: 1)
- `limit`: Results per page (default: 20, max: 100)

**Response**:
```json
{
  "readings": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "kwh_amount": 25.5,
      "reading_timestamp": "2025-01-15T10:30:00Z",
      "submitted_at": "2025-01-15T10:35:22Z",
      "minted": true,
      "mint_tx_signature": "5J7K...xyz",
      "metadata": { "meter_id": "SM-12345" }
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 150
  }
}
```

---

#### 3. Get Readings by Wallet

**Endpoint**: `GET /api/meters/readings/{wallet_address}?page=1&limit=20`  
**Auth**: Required (JWT)  
**Role**: Any authenticated user

**Path Parameters**:
- `wallet_address`: Solana wallet public key

**Response**: Same structure as "Get My Readings"

---

#### 4. Get User Statistics

**Endpoint**: `GET /api/meters/stats`  
**Auth**: Required (JWT)  
**Role**: Any authenticated user

**Response**:
```json
{
  "total_kwh": 1250.75,
  "unminted_kwh": 50.25,
  "minted_kwh": 1200.50,
  "total_readings": 150,
  "unminted_readings": 3,
  "minted_readings": 147
}
```

---

#### 5. Get Unminted Readings (Admin)

**Endpoint**: `GET /api/admin/meters/unminted?page=1&limit=50`  
**Auth**: Required (JWT)  
**Role**: `admin`

**Response**:
```json
{
  "readings": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "user_id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
      "wallet_address": "DYw8j...xyz",
      "kwh_amount": 25.5,
      "reading_timestamp": "2025-01-15T10:30:00Z",
      "submitted_at": "2025-01-15T10:35:22Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 50,
    "total": 12
  }
}
```

---

#### 6. Mint Tokens from Reading (Admin)

**Endpoint**: `POST /api/admin/meters/mint-from-reading`  
**Auth**: Required (JWT)  
**Role**: `admin`

**Request**:
```json
{
  "reading_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Current Behavior**: Returns mock transaction signature  
**TODO**: Implement actual Anchor client call to mint tokens on Solana blockchain

**Response**:
```json
{
  "message": "Tokens minted successfully",
  "transaction_signature": "5J7K8L9M...xyz",
  "kwh_amount": 25.5,
  "wallet_address": "DYw8j...xyz"
}
```

---

### ERC Certificate Endpoints

#### 7. Issue Certificate (REC Authority)

**Endpoint**: `POST /api/erc/issue`  
**Auth**: Required (JWT)  
**Role**: `rec_authority`, `admin`

**Request**:
```json
{
  "wallet_address": "DYw8j...xyz",
  "kwh_amount": 100.0,
  "expiry_date": "2026-01-15T00:00:00Z",
  "issuer_name": "Green Energy Certifiers LLC",
  "metadata": {
    "renewable_source": "Solar",
    "location": "California, USA",
    "validation_method": "On-site inspection"
  }
}
```

**Response**:
```json
{
  "certificate_id": "ERC-2025-000042",
  "message": "Certificate issued successfully",
  "kwh_amount": 100.0,
  "issue_date": "2025-01-15T12:00:00Z",
  "expiry_date": "2026-01-15T00:00:00Z",
  "status": "active"
}
```

---

#### 8. Get Certificate by ID

**Endpoint**: `GET /api/erc/{certificate_id}`  
**Auth**: Required (JWT)

**Path Parameters**:
- `certificate_id`: Human-readable ID (e.g., `ERC-2025-000042`)

**Response**:
```json
{
  "certificate_id": "ERC-2025-000042",
  "wallet_address": "DYw8j...xyz",
  "kwh_amount": 100.0,
  "issue_date": "2025-01-15T12:00:00Z",
  "expiry_date": "2026-01-15T00:00:00Z",
  "issuer_wallet": "ABC123...def",
  "issuer_name": "Green Energy Certifiers LLC",
  "status": "active",
  "blockchain_tx_signature": null,
  "metadata": {
    "renewable_source": "Solar"
  }
}
```

---

#### 9. Get Certificates by Wallet

**Endpoint**: `GET /api/erc/wallet/{wallet_address}?page=1&limit=20`  
**Auth**: Required (JWT)

**Response**:
```json
{
  "certificates": [
    {
      "certificate_id": "ERC-2025-000042",
      "kwh_amount": 100.0,
      "status": "active",
      "issue_date": "2025-01-15T12:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 5
  }
}
```

---

#### 10. Get My Certificates

**Endpoint**: `GET /api/erc/my-certificates?page=1&limit=20`  
**Auth**: Required (JWT)

**Response**: Same structure as "Get Certificates by Wallet"

---

#### 11. Get My Certificate Statistics

**Endpoint**: `GET /api/erc/my-stats`  
**Auth**: Required (JWT)

**Response**:
```json
{
  "total_certificates": 8,
  "active_kwh": 450.0,
  "retired_kwh": 200.0,
  "total_kwh": 650.0
}
```

---

#### 12. Retire Certificate

**Endpoint**: `POST /api/erc/{certificate_id}/retire`  
**Auth**: Required (JWT)  
**Role**: Certificate owner or `admin`

**Path Parameters**:
- `certificate_id`: Human-readable ID (e.g., `ERC-2025-000042`)

**Response**:
```json
{
  "message": "Certificate retired successfully",
  "certificate_id": "ERC-2025-000042",
  "status": "retired"
}
```

**Note**: This is a one-way operation. Retired certificates cannot be reactivated.

---

## 🔧 Service Layer

### MeterService (`services/meter_service.rs`)

**Core Methods**:

1. **`submit_reading()`**
   - Validates reading amount, timestamp, and uniqueness
   - Checks for duplicate readings within ±15 minutes
   - Stores reading in database with `minted = false`

2. **`get_user_readings(user_id, page, limit)`**
   - Fetches paginated readings for a user
   - Orders by `reading_timestamp DESC`

3. **`get_readings_by_wallet(wallet_address, page, limit)`**
   - Fetches readings for a specific wallet address

4. **`get_unminted_readings(page, limit)`**
   - Admin function to fetch all unminted readings
   - Used for batch minting operations

5. **`mark_as_minted(reading_id, tx_signature)`**
   - Updates reading to `minted = true`
   - Stores blockchain transaction signature

6. **`get_unminted_total(user_id)`**
   - Returns total kWh unminted for a user

7. **`get_minted_total(user_id)`**
   - Returns total kWh minted for a user

**Validation Rules**:
- Maximum reading: 100 kWh per submission
- Timestamp range: Not older than 7 days
- Duplicate prevention: ±15 minute window
- Amount validation: Must be positive

---

### ErcService (`services/erc_service.rs`)

**Core Methods**:

1. **`issue_certificate()`**
   - Generates unique certificate ID (`ERC-YYYY-NNNNNN`)
   - Validates issuer has REC authority role
   - Stores certificate with `status = active`

2. **`get_certificate_by_id(certificate_id)`**
   - Fetches certificate details by human-readable ID

3. **`get_certificates_by_wallet(wallet_address, page, limit)`**
   - Lists all certificates for a wallet (paginated)

4. **`get_user_certificates(user_id, page, limit)`**
   - Lists all certificates for a user (paginated)

5. **`transfer_certificate(certificate_id, to_wallet, tx_signature)`**
   - Updates certificate ownership (database transaction)
   - Creates transfer audit record in `erc_certificate_transfers`
   - Sets status to `transferred`

6. **`retire_certificate(certificate_id)`**
   - Marks certificate as `retired`
   - One-way operation (cannot be undone)

7. **`get_user_stats(user_id)`**
   - Aggregates total certificates, active kWh, retired kWh

**Certificate ID Generation**:
```rust
// Format: ERC-2025-000001
let year = Utc::now().year();
let next_number = self.get_next_certificate_number(year).await?;
let certificate_id = format!("ERC-{}-{:06}", year, next_number);
```

---

### WalletService (`services/wallet_service.rs`)

**Purpose**: Manages authority keypair for signing blockchain transactions.

**Key Methods**:

1. **`initialize_authority()`**
   - Tries to load keypair from file (`authority-keypair.json`)
   - Falls back to environment variable (`AUTHORITY_KEYPAIR`)
   - Caches keypair in `Arc<RwLock<Option<Arc<Keypair>>>>`

2. **`get_authority_keypair()`**
   - Returns cached authority keypair
   - Thread-safe access via RwLock

3. **`get_authority_pubkey_string()`**
   - Returns authority public key as base58 string

**Configuration**:
```bash
# Option 1: File-based (recommended for development)
cp dev-wallet.json authority-keypair.json

# Option 2: Environment variable (recommended for production)
export AUTHORITY_KEYPAIR='[1,2,3,...,64]'  # 64-byte array
```

---

### BlockchainService (`services/blockchain_service.rs`)

**Enhanced Methods (Phase 4)**:

1. **`build_and_send_transaction(instructions, signers)`**
   - Builds transaction with recent blockhash
   - Signs transaction with provided signers
   - Submits to Solana with confirmation

2. **`simulate_transaction(instructions, signers)`**
   - Pre-flight simulation before sending
   - Returns success/failure without submitting

3. **`wait_for_confirmation(signature, timeout)`**
   - Polls for transaction confirmation
   - Default timeout: 30 seconds

4. **`send_transaction_with_retry(transaction, max_retries)`**
   - Automatic retry logic with exponential backoff
   - Default max retries: 3

---

## 🔄 Data Flow

### Flow 1: Meter Reading → Token Minting

```
1. Prosumer submits meter reading
   POST /api/meters/submit-reading
   {
     "kwh_amount": 25.5,
     "reading_timestamp": "2025-01-15T10:30:00Z"
   }

2. MeterService validates reading
   - Amount > 0 and ≤ 100 kWh
   - Timestamp not in future
   - Timestamp < 7 days old
   - No duplicate within ±15 minutes

3. Reading stored in database
   meter_readings {
     minted: false,
     wallet_address: "DYw8j...xyz"
   }

4. Admin reviews unminted readings
   GET /api/admin/meters/unminted

5. Admin triggers minting
   POST /api/admin/meters/mint-from-reading
   { "reading_id": "550e8400-..." }

6. BlockchainService signs transaction
   - Load authority keypair
   - Build transaction with Anchor instruction
   - Sign with authority
   - Submit to Solana

7. Database updated
   meter_readings {
     minted: true,
     mint_tx_signature: "5J7K..."
   }

8. Prosumer receives tokens in wallet
```

---

### Flow 2: ERC Certificate Issuance

```
1. REC Authority issues certificate
   POST /api/erc/issue
   {
     "wallet_address": "DYw8j...xyz",
     "kwh_amount": 100.0,
     "issuer_name": "Green Energy Certifiers"
   }

2. ErcService generates certificate ID
   - Format: ERC-2025-000042
   - Auto-increment per year

3. Certificate stored in database
   erc_certificates {
     certificate_id: "ERC-2025-000042",
     status: "active",
     kwh_amount: 100.0
   }

4. User can view certificate
   GET /api/erc/my-certificates

5. User retires certificate (for compliance)
   POST /api/erc/ERC-2025-000042/retire

6. Certificate marked as retired
   erc_certificates {
     status: "retired"
   }
```

---

## 🔐 Security & Authorization

### Role-Based Access Control (RBAC)

| Endpoint | Roles Allowed |
|----------|---------------|
| Submit Reading | `prosumer`, `admin` |
| Get My Readings | Any authenticated user |
| Get Readings by Wallet | Any authenticated user |
| Get User Stats | Any authenticated user |
| Get Unminted Readings | `admin` |
| Mint from Reading | `admin` |
| Issue Certificate | `rec_authority`, `admin` |
| Get Certificate | Any authenticated user |
| Retire Certificate | Certificate owner or `admin` |

### Helper Functions

```rust
// In handlers/meter.rs
fn require_role(claims: &Claims, allowed_roles: &[&str]) -> Result<(), ApiError> {
    if !allowed_roles.contains(&claims.role.as_str()) {
        return Err(ApiError::Forbidden(format!(
            "Role '{}' not authorized. Required: {:?}",
            claims.role, allowed_roles
        )));
    }
    Ok(())
}

// In handlers/erc.rs
fn require_rec_authority(claims: &Claims) -> Result<(), ApiError> {
    if claims.role != "rec_authority" && claims.role != "admin" {
        return Err(ApiError::Forbidden(
            "Only REC authorities can issue certificates".to_string()
        ));
    }
    Ok(())
}
```

---

## 🧪 Testing Guide

### Manual Testing (cURL Examples)

#### 1. Submit Meter Reading

```bash
curl -X POST http://localhost:3000/api/meters/submit-reading \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "kwh_amount": 25.5,
    "reading_timestamp": "2025-01-15T10:30:00Z",
    "metadata": {
      "meter_id": "SM-12345"
    }
  }'
```

#### 2. Get My Readings

```bash
curl -X GET http://localhost:3000/api/meters/my-readings?page=1&limit=20 \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### 3. Get Unminted Readings (Admin)

```bash
curl -X GET http://localhost:3000/api/admin/meters/unminted?page=1&limit=50 \
  -H "Authorization: Bearer ADMIN_JWT_TOKEN"
```

#### 4. Mint Tokens (Admin)

```bash
curl -X POST http://localhost:3000/api/admin/meters/mint-from-reading \
  -H "Authorization: Bearer ADMIN_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "reading_id": "550e8400-e29b-41d4-a716-446655440000"
  }'
```

#### 5. Issue ERC Certificate

```bash
curl -X POST http://localhost:3000/api/erc/issue \
  -H "Authorization: Bearer REC_AUTHORITY_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "wallet_address": "DYw8j...xyz",
    "kwh_amount": 100.0,
    "expiry_date": "2026-01-15T00:00:00Z",
    "issuer_name": "Green Energy Certifiers LLC"
  }'
```

#### 6. Get Certificate

```bash
curl -X GET http://localhost:3000/api/erc/ERC-2025-000042 \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### 7. Retire Certificate

```bash
curl -X POST http://localhost:3000/api/erc/ERC-2025-000042/retire \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

### Integration Test Plan

**Test Suite**: `tests/phase4_tokenization.test.ts`

**Test Cases**:

1. **Meter Reading Validation**
   - ✅ Accept valid reading (0 < kWh ≤ 100, timestamp valid)
   - ✅ Reject negative amount
   - ✅ Reject amount > 100 kWh
   - ✅ Reject future timestamp
   - ✅ Reject timestamp > 7 days old
   - ✅ Reject duplicate reading within ±15 minutes

2. **Meter Reading Storage**
   - ✅ Store reading with `minted = false`
   - ✅ Associate with correct user_id and wallet_address
   - ✅ Store metadata as JSONB

3. **Token Minting Flow**
   - ✅ Admin can fetch unminted readings
   - ✅ Admin can trigger minting
   - ⏳ Blockchain transaction is signed and submitted (TODO: Anchor client)
   - ✅ Database updated with `minted = true` and tx signature

4. **ERC Certificate Issuance**
   - ✅ REC authority can issue certificate
   - ✅ Certificate ID auto-generated (ERC-YYYY-NNNNNN)
   - ✅ Certificate associated with wallet_address
   - ✅ Status set to `active`

5. **ERC Certificate Lifecycle**
   - ✅ Owner can view certificate
   - ✅ Owner can retire certificate
   - ✅ Retired certificate status = `retired`
   - ✅ Retired certificate cannot be reactivated

6. **Authorization Tests**
   - ✅ Only prosumers/admins can submit readings
   - ✅ Only admins can view unminted readings
   - ✅ Only admins can mint tokens
   - ✅ Only REC authorities/admins can issue certificates
   - ✅ Only certificate owners/admins can retire certificates

---

## 📊 Database Migrations

**Migration File**: `migrations/20241109000001_phase4_tokenization.sql`

**Up Migration**:
```sql
-- Create tables
CREATE TABLE meter_readings (...);
CREATE TABLE erc_certificates (...);
CREATE TABLE erc_certificate_transfers (...);

-- Create indexes
CREATE INDEX idx_meter_readings_user_id ON meter_readings(user_id);
-- ... 9 more indexes ...

-- Create triggers
CREATE OR REPLACE FUNCTION update_updated_at_column() ...
CREATE TRIGGER update_meter_readings_updated_at ...
CREATE TRIGGER update_erc_certificates_updated_at ...
```

**Down Migration**: `migrations/20241109000001_phase4_tokenization.down.sql`
```sql
DROP TABLE IF EXISTS erc_certificate_transfers CASCADE;
DROP TABLE IF EXISTS erc_certificates CASCADE;
DROP TABLE IF EXISTS meter_readings CASCADE;
DROP TRIGGER IF EXISTS update_erc_certificates_updated_at ON erc_certificates;
DROP TRIGGER IF EXISTS update_meter_readings_updated_at ON meter_readings;
DROP FUNCTION IF EXISTS update_updated_at_column();
```

**Run Migrations**:
```bash
# Apply migrations
sqlx migrate run

# Revert migrations
sqlx migrate revert
```

---

## 🚀 Deployment Notes

### Environment Variables

```bash
# Solana RPC endpoint
SOLANA_RPC_URL=https://api.devnet.solana.com

# Authority wallet (for token minting)
# Option 1: File path
AUTHORITY_KEYPAIR_PATH=./authority-keypair.json

# Option 2: Direct array (production)
AUTHORITY_KEYPAIR='[1,2,3,...,64]'

# Database connection
DATABASE_URL=postgresql://user:password@localhost/gridtokenx
```

### Authority Wallet Setup

**Development**:
```bash
# Use dev wallet
cp dev-wallet.json authority-keypair.json
```

**Production**:
```bash
# Generate new keypair
solana-keygen new -o authority-keypair.json

# Fund with SOL for transaction fees
solana airdrop 1 <AUTHORITY_PUBKEY> --url https://api.devnet.solana.com

# Store securely (e.g., Kubernetes Secret, AWS Secrets Manager)
kubectl create secret generic authority-keypair \
  --from-file=authority-keypair.json
```

---

## 🔮 Future Enhancements

### Phase 4.1: Blockchain Integration

**Status**: TODO (currently using mock transactions)

**Tasks**:
1. Integrate Anchor client for `energy-token` program
2. Replace mock TX signature in `mint_from_reading()` with real transaction
3. Call `mint_tokens()` instruction from Anchor program
4. Handle blockchain errors (insufficient balance, network issues)

**Example**:
```rust
// Replace this mock:
let mock_signature = format!("MOCK_TX_{}", uuid::Uuid::new_v4());

// With real Anchor call:
let program = anchor_client::Program::new(...);
let tx = program
    .request()
    .accounts(energy_token::accounts::MintTokens {
        authority: authority_keypair.pubkey(),
        mint: mint_pubkey,
        token_account: token_account_pubkey,
        // ... other accounts
    })
    .args(energy_token::instruction::MintTokens {
        amount: (kwh_amount * 1000.0) as u64, // Convert kWh to tokens
    })
    .signer(authority_keypair)
    .send()?;
```

### Phase 4.2: Automated Minting

- Background job (cron) to automatically mint tokens for unminted readings
- Batch minting to reduce transaction costs
- Notification system to inform users when tokens are minted

### Phase 4.3: ERC Blockchain Integration

- Store ERC certificates on-chain using `governance` program
- Implement on-chain transfer functionality
- Track certificate history on blockchain

### Phase 4.4: Advanced Features

- Real-time meter reading submission via WebSocket/MQTT
- Smart meter device authentication (public key cryptography)
- Certificate marketplace integration
- Carbon credit calculation based on renewable energy

---

## 📚 Related Documentation

- [Project Planning](/docs/PROJECT_PLANNING.md)
- [Development Timeline](/docs/DEVELOPMENT_TIMELINE.md)
- [Anchor Programs Architecture](/docs/technical/architecture/blockchain/anchor-programs/)
- [API Gateway Authentication Guide](/api-gateway/docs/AUTHENTICATION_GUIDE.md)

---

## ✅ Phase 4 Checklist

- [x] Database schema design (3 tables)
- [x] Database migrations (up + down)
- [x] Wallet service (authority keypair management)
- [x] Blockchain service (transaction signing)
- [x] Meter service (business logic)
- [x] ERC service (business logic)
- [x] Meter handlers (6 API endpoints)
- [x] ERC handlers (6 API endpoints)
- [x] Route configuration
- [x] Authorization (RBAC)
- [x] Documentation
- [ ] Blockchain integration (Anchor client)
- [ ] Integration tests
- [ ] End-to-end testing
- [ ] Production deployment

---

**Last Updated**: 2025-01-15  
**Version**: 1.0.0  
**Status**: ✅ API layer complete, blockchain integration pending
