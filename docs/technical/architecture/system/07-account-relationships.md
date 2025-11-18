# 🔗 Account Relationships

**GridTokenX Platform - Account Dependencies & PDA Structure**

> **📘 For complete account structure documentation, see:**
> - [Account Structure Diagram](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml#ANCHOR_ACCOUNT_STRUCTURE)
> - [Account Schemas](../anchor/ANCHOR_QUICK_REFERENCE.md#account-structures)
> - [PDA Details](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#account-structure)
> 
> **Note**: This document provides account relationships. For complete field definitions and sizes, refer to the Anchor documentation.

---

## Table of Contents

1. [Account Dependency Graph](#account-dependency-graph)
2. [Program Derived Addresses (PDAs)](#program-derived-addresses-pdas)
3. [Account Ownership & Access Patterns](#account-ownership--access-patterns)
4. [Cross-Program Account References](#cross-program-account-references)
5. [Account Lifecycle Management](#account-lifecycle-management)

---

## Account Dependency Graph

```
┌──────────────────────────────────────────────────────────┐
│              Solana Blockchain Accounts                  │
└──────────────────────────────────────────────────────────┘

Oracle Accounts:
  OracleData [seed: b"oracle_data"]
      │
      │ (references)
      ├─► API Gateway Pubkey
      │
      └─► Authority Pubkey

Governance Accounts:
  PoAConfig [seed: b"poa_config"]
      │
      ├─► Authority (Engineering Dept)
      │
      └─► ErcCertificate [seed: b"erc_certificate", certificate_id]
          │
          ├─► Authority (issuer)
          │
          └─► Status (Valid/Expired/Revoked/Pending)

Registry Accounts:
  Registry [seed: b"registry"]
      │
      ├─► Authority
      │
      ├─► UserAccount [seed: b"user", user_pubkey] (multiple)
      │   │
      │   ├─► User Authority
      │   ├─► User Type (Prosumer/Consumer)
      │   │
      │   └─► MeterAccount [seed: b"meter", meter_id] (multiple)
      │       │
      │       ├─► Meter Owner
      │       ├─► Meter Type (Solar/Wind/Battery/Grid)
      │       │
      │       └─► readings (via Oracle)

Energy-Token Accounts:
  TokenInfo [seed: b"token_info"]
      │
      ├─► Authority
      │
      ├─► SPL Token Mint
      │
      └─► User TokenAccounts (via SPL Token Program)

Trading Accounts:
  Market [seed: b"market"]
      │
      ├─► Authority
      │
      ├─► Order (multiple) [seed: b"order", order_id]
      │   │
      │   ├─► Seller Pubkey
      │   ├─► Buyer Pubkey
      │   │
      │   └─► Order Status (Active/PartiallyFilled/Completed/Cancelled/Expired)
      │
      └─► TradeRecord (multiple)
          │
          ├─► Seller Pubkey
          ├─► Buyer Pubkey
          │
          └─► Trade Execution Timestamp
```

---

## Program Derived Addresses (PDAs)

### Oracle Program PDAs

#### OracleData Account
- **Seed**: `[b"oracle_data"]`
- **Program**: Oracle Program
- **Purpose**: Central oracle configuration and state
- **Size**: 8 + OracleData::INIT_SPACE bytes
- **Lifetime**: Permanent (until program upgrade)

**Fields**:
```rust
pub struct OracleData {
    pub authority: Pubkey,           // 32 bytes
    pub api_gateway: Pubkey,         // 32 bytes  
    pub total_readings: u64,         // 8 bytes
    pub last_reading_timestamp: i64, // 8 bytes
    pub last_clearing: i64,          // 8 bytes
    pub active: bool,                // 1 byte
    pub created_at: i64,             // 8 bytes
}
```

### Governance Program PDAs

#### PoAConfig Account
- **Seed**: `[b"poa_config"]`
- **Program**: Governance Program
- **Purpose**: Proof of Authority governance configuration
- **Size**: 8 + PoAConfig::INIT_SPACE bytes
- **Lifetime**: Permanent (until governance changes)

**Fields**:
```rust
pub struct PoAConfig {
    pub authority: Pubkey,              // 32 bytes
    pub authority_name: String,         // Variable (max 64)
    pub contact_info: String,           // Variable (max 128)
    pub emergency_paused: bool,         // 1 byte
    pub emergency_timestamp: Option<i64>, // 9 bytes
    pub erc_validation_enabled: bool,   // 1 byte
    pub max_erc_amount: u64,           // 8 bytes
    pub min_energy_amount: u64,        // 8 bytes
    pub erc_validity_period: i64,      // 8 bytes
    pub total_ercs_issued: u64,        // 8 bytes
    pub total_ercs_validated: u64,     // 8 bytes
    pub maintenance_mode: bool,         // 1 byte
    pub created_at: i64,               // 8 bytes
    pub last_updated: i64,             // 8 bytes
}
```

#### ErcCertificate Account
- **Seed**: `[b"erc_certificate", certificate_id.as_bytes()]`
- **Program**: Governance Program
- **Purpose**: Individual Energy Renewable Certificate
- **Size**: 8 + ErcCertificate::INIT_SPACE bytes
- **Lifetime**: Based on validity period (typically 1 year)

**Fields**:
```rust
pub struct ErcCertificate {
    pub certificate_id: String,           // Variable (max 64)
    pub authority: Pubkey,                 // 32 bytes
    pub energy_amount: u64,               // 8 bytes
    pub renewable_source: String,          // Variable (max 64)
    pub validation_data: String,           // Variable (max 256)
    pub status: ErcStatus,                 // 1 byte
    pub validated_for_trading: bool,       // 1 byte
    pub created_at: i64,                  // 8 bytes
    pub expires_at: i64,                  // 8 bytes
    pub trading_validated_at: Option<i64>, // 9 bytes
    pub trading_validator: Option<Pubkey>, // 33 bytes
}
```

### Registry Program PDAs

#### Registry Account
- **Seed**: `[b"registry"]`
- **Program**: Registry Program
- **Purpose**: Central registry state and statistics
- **Size**: 8 + Registry::INIT_SPACE bytes
- **Lifetime**: Permanent

**Fields**:
```rust
pub struct Registry {
    pub authority: Pubkey,    // 32 bytes
    pub user_count: u64,      // 8 bytes
    pub meter_count: u64,     // 8 bytes
    pub created_at: i64,      // 8 bytes
    pub last_updated: i64,    // 8 bytes
}
```

#### UserAccount Account
- **Seed**: `[b"user", user_pubkey.as_ref()]`
- **Program**: Registry Program
- **Purpose**: Individual user registration and state
- **Size**: 8 + UserAccount::INIT_SPACE bytes
- **Lifetime**: Permanent (until user deregistration)

**Fields**:
```rust
pub struct UserAccount {
    pub owner: Pubkey,                   // 32 bytes
    pub user_type: UserType,             // 1 byte
    pub location: String,                // Variable (max 128)
    pub status: UserStatus,              // 1 byte
    pub meter_count: u32,                // 4 bytes
    pub total_energy_produced: u64,      // 8 bytes
    pub total_energy_consumed: u64,      // 8 bytes
    pub created_at: i64,                 // 8 bytes
    pub last_updated: i64,               // 8 bytes
}
```

#### MeterAccount Account
- **Seed**: `[b"meter", meter_id.as_bytes()]`
- **Program**: Registry Program
- **Purpose**: Individual meter registration and readings
- **Size**: 8 + MeterAccount::INIT_SPACE bytes
- **Lifetime**: Permanent (until meter decommission)

**Fields**:
```rust
pub struct MeterAccount {
    pub meter_id: String,                // Variable (max 64)
    pub owner: Pubkey,                   // 32 bytes
    pub meter_type: MeterType,           // 1 byte
    pub status: MeterStatus,             // 1 byte
    pub total_energy_produced: u64,      // 8 bytes
    pub total_energy_consumed: u64,      // 8 bytes
    pub last_reading_timestamp: i64,     // 8 bytes
    pub created_at: i64,                 // 8 bytes
    pub last_updated: i64,               // 8 bytes
}
```

### Energy-Token Program PDAs

#### TokenInfo Account
- **Seed**: `[b"token_info"]`
- **Program**: Energy-Token Program
- **Purpose**: SPL token mint management and statistics
- **Size**: 8 + TokenInfo::INIT_SPACE bytes
- **Lifetime**: Permanent

**Fields**:
```rust
pub struct TokenInfo {
    pub authority: Pubkey,           // 32 bytes
    pub mint: Option<Pubkey>,        // 33 bytes
    pub total_supply: u64,           // 8 bytes
    pub total_burned: u64,           // 8 bytes
    pub created_at: i64,             // 8 bytes
    pub last_updated: i64,           // 8 bytes
}
```

### Trading Program PDAs

#### Market Account
- **Seed**: `[b"market"]`
- **Program**: Trading Program
- **Purpose**: Central market configuration and statistics
- **Size**: 8 + Market::INIT_SPACE bytes
- **Lifetime**: Permanent

**Fields**:
```rust
pub struct Market {
    pub authority: Pubkey,         // 32 bytes
    pub total_orders: u64,         // 8 bytes
    pub total_trades: u64,         // 8 bytes
    pub total_volume: u64,         // 8 bytes
    pub trading_fee: u16,          // 2 bytes (basis points)
    pub min_order_size: u64,       // 8 bytes
    pub max_order_size: u64,       // 8 bytes
    pub clearing_enabled: bool,    // 1 byte
    pub created_at: i64,           // 8 bytes
    pub last_updated: i64,         // 8 bytes
}
```

#### Order Account
- **Seed**: `[b"order", order_id.as_bytes()]`
- **Program**: Trading Program
- **Purpose**: Individual buy/sell order state and escrow
- **Size**: 8 + Order::INIT_SPACE bytes
- **Lifetime**: Until completion, cancellation, or expiration

**Fields**:
```rust
pub struct Order {
    pub order_id: String,               // Variable (max 32)
    pub seller: Option<Pubkey>,         // 33 bytes
    pub buyer: Option<Pubkey>,          // 33 bytes
    pub order_type: OrderType,          // 1 byte
    pub amount: u64,                    // 8 bytes
    pub price_per_kwh: u64,            // 8 bytes
    pub max_price_per_kwh: Option<u64>, // 9 bytes
    pub status: OrderStatus,            // 1 byte
    pub escrow_amount: u64,            // 8 bytes
    pub filled_amount: u64,            // 8 bytes
    pub created_at: i64,               // 8 bytes
    pub expires_at: i64,               // 8 bytes
    pub cancelled_at: Option<i64>,     // 9 bytes
}
```

---

## Account Ownership & Access Patterns

### Authority Hierarchy

```
System Level:
├── Oracle Authority (System Admin)
├── Governance Authority (Engineering Department)
├── Registry Authority (System Admin)
├── Energy-Token Authority (System Admin)
└── Trading Authority (System Admin)

User Level:
├── User Account Owner (Individual Users)
├── Meter Account Owner (User who registered meter)
└── Order Account Owner (User who created order)

External:
└── API Gateway (Authorized Oracle caller)
```

### Access Control Matrix

| Account Type | Create | Read | Update | Delete | Authority Required |
|--------------|--------|------|--------|--------|--------------------|
| OracleData | ✅ | ✅ | ✅ | ❌ | Oracle Authority |
| PoAConfig | ✅ | ✅ | ✅ | ❌ | Governance Authority |
| ErcCertificate | ✅ | ✅ | ✅ | ❌ | Governance Authority |
| Registry | ✅ | ✅ | ✅ | ❌ | Registry Authority |
| UserAccount | ✅ | ✅ | ✅ | ❌ | User (self-registration) |
| MeterAccount | ✅ | ✅ | ✅ | ❌ | User (owner) |
| TokenInfo | ✅ | ✅ | ✅ | ❌ | Energy-Token Authority |
| SPL Token Accounts | 🔄 | ✅ | 🔄 | 🔄 | SPL Token Program |
| Market | ✅ | ✅ | ✅ | ❌ | Trading Authority |
| Order | ✅ | ✅ | ✅ | 🔄 | User (order creator) |

**Legend**: ✅ = Allowed, ❌ = Not Allowed, 🔄 = Via Program Integration

---

## Cross-Program Account References

### Oracle → Registry
- **Purpose**: Update meter readings after validation
- **Pattern**: Oracle reads meter data, validates, then calls Registry
- **Accounts**: Oracle accesses Registry's MeterAccount PDAs

### Governance → Energy-Token
- **Purpose**: Trigger token issuance after ERC validation
- **Pattern**: Governance validates ERC, then calls Energy-Token
- **Accounts**: Governance triggers token minting to user SPL accounts

### Registry → Governance
- **Purpose**: Automatic ERC issuance for energy production
- **Pattern**: Registry meter updates trigger ERC issuance
- **Accounts**: Registry meter data flows to Governance ErcCertificate creation

### Energy-Token → Trading
- **Purpose**: Energy tokens become tradeable assets
- **Pattern**: Users trade SPL tokens representing energy
- **Accounts**: Trading program manages SPL token escrow accounts

### All Programs → SPL Token Program
- **Purpose**: Standard token operations (mint, transfer, burn)
- **Pattern**: Energy-Token wraps SPL Token Program calls
- **Accounts**: User token accounts managed by SPL Token Program

---

## Account Lifecycle Management

### Creation Lifecycle

```
1. Program Initialization:
   ┌─ Oracle.initialize() → OracleData PDA
   ├─ Governance.initialize_poa() → PoAConfig PDA
   ├─ Registry.initialize() → Registry PDA
   ├─ Energy-Token.initialize() → TokenInfo PDA
   └─ Trading.initialize_market() → Market PDA

2. User Onboarding:
   ┌─ Registry.register_user() → UserAccount PDA
   └─ Registry.register_meter() → MeterAccount PDA

3. Energy Certificate Flow:
   ┌─ Governance.issue_erc() → ErcCertificate PDA
   └─ Governance.validate_erc_for_trading() → Updates ErcCertificate

4. Token Management:
   ┌─ Energy-Token.initialize_token() → SPL Token Mint
   ├─ Energy-Token.transfer_tokens() → SPL Token Accounts
   └─ Energy-Token.burn_tokens() → Reduces SPL Token Supply

5. Trading Activity:
   ┌─ Trading.create_sell_order() → Order PDA + Escrow
   ├─ Trading.create_buy_order() → Order PDA + Escrow
   ├─ Trading.match_orders() → TradeRecord + Token Transfers
   └─ Trading.cancel_order() → Order Cleanup + Escrow Return
```

### Account State Transitions

```
UserAccount States:
Active → Suspended → Inactive
  ↑         ↓
  └─── Reactivated ────┘

MeterAccount States:
Active → Maintenance → Inactive
  ↑         ↓
  └─── Restored ───────┘

ErcCertificate States:
Pending → Valid → Expired
             ↓
           Revoked

Order States:
Active → PartiallyFilled → Completed
  ↓                          ↑
Cancelled ←────────────────┘
  ↓
Expired
```

### Account Cleanup & Archival

```
Automatic Cleanup:
├── Orders: Expire after 24 hours if not filled
├── ERCs: Expire after validity period (1 year)
└── Trading Records: Permanent (for audit trail)

Manual Cleanup:
├── Suspended Users: Admin intervention required
├── Maintenance Meters: Owner or admin intervention
└── Emergency Paused State: Authority intervention required

Permanent Storage:
├── All transaction history (via events)
├── User and meter registration records
├── ERC issuance and validation records
└── Trade execution records
```

---

## Security Considerations

### PDA Seed Security
- **Deterministic**: Seeds ensure predictable account addresses
- **Collision Resistant**: Unique seeds prevent account conflicts
- **Program Bound**: PDAs can only be signed by owning program

### Access Control Enforcement
- **Program Level**: Each program validates caller permissions
- **Account Level**: PDAs restrict access to authorized signers
- **Cross-Program**: CPI calls validate source program authority

### Data Integrity
- **Immutable Records**: Critical data (trades, ERCs) cannot be deleted
- **Audit Trail**: All state changes emit events for verification
- **Consistency**: Cross-program calls maintain data consistency

---

**[← Back to Architecture Overview](./README.md)**