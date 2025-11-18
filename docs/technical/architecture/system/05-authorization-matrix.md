# 🔐 Authorization Matrix

**GridTokenX Platform - Access Control & Permission Documentation**

> **📘 For complete security model documentation, see:**
> - [Security Model Overview](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#security-model)
> - [Security Layers Diagram](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml#ANCHOR_SECURITY_MODEL)
> - [Authorization Details](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#authorization-and-access-control)
> 
> **Note**: This document provides the authorization matrix. For security implementation details and patterns, refer to the Anchor documentation.

---

## Table of Contents

1. [Function Authorization Matrix](#function-authorization-matrix)
2. [Role-Based Access Control](#role-based-access-control)
3. [Account-Level Permissions](#account-level-permissions)
4. [Cross-Program Authorization](#cross-program-authorization)
5. [Emergency Override Procedures](#emergency-override-procedures)
6. [Security Implementation Details](#security-implementation-details)

---

## Function Authorization Matrix

### Complete Function Access Control

```
┌─────────────────────────────────┬───────────┬──────────┬────────┬───────┬─────────┐
│ Function                        │ Authority │ API Gw   │ User   │ Other │ Notes   │
├─────────────────────────────────┼───────────┼──────────┼────────┼───────┼─────────┤
│ ORACLE PROGRAM                  │           │          │        │       │         │
│ ├─ initialize                   │ ✅        │          │        │       │ Setup   │
│ ├─ submit_meter_reading         │           │ ✅       │        │       │ AMI     │
│ ├─ trigger_market_clearing      │           │ ✅       │        │       │ AMI     │
│ ├─ update_oracle_status         │ ✅        │          │        │       │ Admin   │
│ ├─ update_api_gateway           │ ✅        │          │        │       │ Admin   │
│ └─ get_oracle_data              │ ✅        │ ✅       │ ✅     │ ✅    │ Read    │
├─────────────────────────────────┼───────────┼──────────┼────────┼───────┼─────────┤
│ GOVERNANCE PROGRAM              │           │          │        │       │         │
│ ├─ initialize_poa               │ ✅        │          │        │       │ Setup   │
│ ├─ issue_erc                    │ ✅        │          │        │       │ Admin   │
│ ├─ validate_erc_for_trading     │ ✅        │          │        │       │ Admin   │
│ ├─ emergency_pause              │ ✅        │          │        │       │ Admin   │
│ ├─ emergency_unpause            │ ✅        │          │        │       │ Admin   │
│ ├─ update_governance_config     │ ✅        │          │        │       │ Admin   │
│ ├─ set_maintenance_mode         │ ✅        │          │        │       │ Admin   │
│ ├─ update_erc_limits            │ ✅        │          │        │       │ Admin   │
│ ├─ update_authority_info        │ ✅        │          │        │       │ Admin   │
│ ├─ get_governance_stats         │ ✅        │ ✅       │ ✅     │ ✅    │ Read    │
│ └─ get_erc_certificate          │ ✅        │ ✅       │ ✅     │ ✅    │ Read    │
├─────────────────────────────────┼───────────┼──────────┼────────┼───────┼─────────┤
│ REGISTRY PROGRAM                │           │          │        │       │         │
│ ├─ initialize                   │ ✅        │          │        │       │ Setup   │
│ ├─ register_user                │           │          │ ✅     │       │ Self    │
│ ├─ register_meter               │           │          │ ✅     │       │ Owner   │
│ ├─ update_meter_reading         │           │ ✅       │        │       │ Oracle  │
│ ├─ update_user_status           │ ✅        │          │        │       │ Admin   │
│ ├─ is_valid_user                │ ✅        │ ✅       │ ✅     │ ✅    │ Read    │
│ ├─ is_valid_meter               │ ✅        │ ✅       │ ✅     │ ✅    │ Read    │
│ ├─ get_user_account             │ ✅        │ ✅       │ ✅     │ ✅    │ Read    │
│ └─ get_meter_account            │ ✅        │ ✅       │ ✅     │ ✅    │ Read    │
├─────────────────────────────────┼───────────┼──────────┼────────┼───────┼─────────┤
│ ENERGY-TOKEN PROGRAM            │           │          │        │       │         │
│ ├─ initialize                   │ ✅        │          │        │       │ Setup   │
│ ├─ initialize_token             │ ✅        │          │        │       │ Setup   │
│ ├─ add_rec_validator            │ ✅        │          │        │       │ Admin   │
│ ├─ transfer_tokens              │ 🔄        │          │ ✅     │       │ CPI/User│
│ ├─ burn_tokens                  │           │          │ ✅     │       │ Owner   │
│ └─ get_token_info               │ ✅        │ ✅       │ ✅     │ ✅    │ Read    │
├─────────────────────────────────┼───────────┼──────────┼────────┼───────┼─────────┤
│ TRADING PROGRAM                 │           │          │        │       │         │
│ ├─ initialize                   │ ✅        │          │        │       │ Setup   │
│ ├─ initialize_market            │ ✅        │          │        │       │ Setup   │
│ ├─ create_sell_order            │           │          │ ✅     │       │ Owner   │
│ ├─ create_buy_order             │           │          │ ✅     │       │ Owner   │
│ ├─ match_orders                 │ ✅        │          │        │       │ Admin   │
│ ├─ cancel_order                 │           │          │ ✅     │       │ Owner   │
│ ├─ update_market_params         │ ✅        │          │        │       │ Admin   │
│ ├─ get_market_data              │ ✅        │ ✅       │ ✅     │ ✅    │ Read    │
│ ├─ get_order                    │ ✅        │ ✅       │ ✅     │ ✅    │ Read    │
│ └─ get_trade_record             │ ✅        │ ✅       │ ✅     │ ✅    │ Read    │
└─────────────────────────────────┴───────────┴──────────┴────────┴───────┴─────────┘

Legend:
✅ = Authorized to call function
🔄 = Authorized via Cross-Program Invocation (CPI)
(blank) = Not authorized to call

Authority = Program admin/authority account
API Gw = API Gateway (for Oracle)  
User = Individual user/signer
Other = Public/anyone
```

---

## Role-Based Access Control

### System Roles Definition

#### 1. System Administrator
**Purpose**: Highest level access for system operations and emergency management

**Permissions**:
- Initialize all programs
- Update system configurations
- Manage emergency states
- Override user restrictions
- Access all system data

**Functions Authorized**:
```rust
// Oracle Program
initialize()
update_oracle_status()
update_api_gateway()

// Governance Program  
initialize_poa()
emergency_pause()
emergency_unpause()
update_governance_config()
set_maintenance_mode()
update_erc_limits()
update_authority_info()

// Registry Program
initialize()
update_user_status()

// Energy-Token Program
initialize()
initialize_token()
add_rec_validator()

// Trading Program
initialize()
initialize_market()
match_orders()
update_market_params()
```

#### 2. Engineering Authority (Governance)
**Purpose**: Technical oversight and renewable energy certificate management

**Permissions**:
- Issue Energy Renewable Certificates (ERCs)
- Validate ERCs for trading
- Manage governance parameters
- Emergency system controls

**Functions Authorized**:
```rust
// Governance Program
issue_erc()
validate_erc_for_trading()
emergency_pause()
emergency_unpause()
update_governance_config()
update_erc_limits()
update_authority_info()

// Cross-Program Calls
// → Energy-Token.transfer_tokens() (via CPI)
```

#### 3. API Gateway
**Purpose**: Automated meter reading submission and market operations

**Permissions**:
- Submit meter readings from AMI infrastructure
- Trigger periodic market clearing
- Read system status data

**Functions Authorized**:
```rust
// Oracle Program
submit_meter_reading()
trigger_market_clearing()

// Cross-Program Calls
// → Registry.update_meter_reading() (via CPI)
// → Trading.match_orders() (via CPI)
```

#### 4. Registered User
**Purpose**: Individual energy producers/consumers participating in the platform

**Permissions**:
- Self-register on platform
- Register owned energy meters
- Create trading orders
- Manage personal token holdings

**Functions Authorized**:
```rust
// Registry Program
register_user() // Self-registration
register_meter() // Own meters only

// Energy-Token Program
transfer_tokens() // Own tokens only
burn_tokens() // Own tokens only

// Trading Program
create_sell_order() // Own energy/tokens
create_buy_order() // Own tokens
cancel_order() // Own orders only
```

#### 5. Public/Anonymous
**Purpose**: General public access to read-only system information

**Permissions**:
- View public system statistics
- Read market data
- Access governance information

**Functions Authorized**:
```rust
// All Read-Only Functions
get_oracle_data()
get_governance_stats()
get_erc_certificate()
is_valid_user()
is_valid_meter()
get_user_account() // Public data only
get_meter_account() // Public data only
get_token_info()
get_market_data()
get_order() // Public orders only
get_trade_record()
```

---

## Account-Level Permissions

### Program-Derived Address (PDA) Access Control

#### Oracle PDAs
```rust
// OracleData PDA [seed: "oracle_data"]
struct OraclePermissions {
    create: Authority Only,
    read: Anyone,
    update: Authority + API Gateway,
    delete: None (Permanent)
}
```

#### Governance PDAs
```rust
// PoAConfig PDA [seed: "poa_config"] 
struct PoAConfigPermissions {
    create: Authority Only,
    read: Anyone,
    update: Authority Only,
    delete: None (Permanent)
}

// ErcCertificate PDA [seed: "erc_certificate", cert_id]
struct ErcPermissions {
    create: Authority Only,
    read: Anyone,
    update: Authority Only,
    delete: None (Permanent)
}
```

#### Registry PDAs
```rust
// Registry PDA [seed: "registry"]
struct RegistryPermissions {
    create: Authority Only,
    read: Anyone,
    update: Authority Only,
    delete: None (Permanent)
}

// UserAccount PDA [seed: "user", user_pubkey]
struct UserAccountPermissions {
    create: User (Self) Only,
    read: Anyone (Public Data), Owner (Private Data),
    update: Authority + Owner (Limited),
    delete: None (Permanent)
}

// MeterAccount PDA [seed: "meter", meter_id]
struct MeterAccountPermissions {
    create: User (Owner) Only,
    read: Anyone,
    update: Authority + Owner + Oracle,
    delete: None (Permanent)
}
```

#### Energy-Token PDAs
```rust
// TokenInfo PDA [seed: "token_info"]
struct TokenInfoPermissions {
    create: Authority Only,
    read: Anyone,
    update: Authority + Cross-Program Calls,
    delete: None (Permanent)
}

// SPL Token Accounts (User Token Holdings)
struct TokenAccountPermissions {
    create: SPL Token Program,
    read: Anyone (Balance), Owner (Full),
    update: SPL Token Program Only,
    delete: SPL Token Program (If Zero Balance)
}
```

#### Trading PDAs
```rust
// Market PDA [seed: "market"]
struct MarketPermissions {
    create: Authority Only,
    read: Anyone,
    update: Authority Only,
    delete: None (Permanent)
}

// Order PDA [seed: "order", order_id]  
struct OrderPermissions {
    create: User (Order Creator) Only,
    read: Anyone,
    update: Authority + Order Creator (Cancel Only),
    delete: System (On Completion/Expiration)
}
```

---

## Cross-Program Authorization

### Cross-Program Invocation (CPI) Security

#### Authorized CPI Patterns

##### 1. Oracle → Registry
```rust
// Oracle Program calls Registry Program
Authority: Oracle Program ID
Target: Registry.update_meter_reading()
Validation: 
├─ Caller must be Oracle Program
├─ Oracle must be active
└─ Meter must exist in Registry

Security Checks:
├─ invoke_signed() with Oracle PDA as signer
├─ Registry validates caller program ID
└─ Meter ownership verification
```

##### 2. Registry → Governance  
```rust
// Registry Program calls Governance Program
Authority: Registry Program ID
Target: Governance.issue_erc()
Validation:
├─ Caller must be Registry Program
├─ Energy production qualifies for ERC
└─ Governance not in emergency pause

Security Checks:
├─ invoke_signed() with Registry PDA as signer
├─ Governance validates caller program ID
└─ ERC issuance limits verification
```

##### 3. Governance → Energy-Token
```rust
// Governance Program calls Energy-Token Program
Authority: Governance Program ID
Target: EnergyToken.transfer_tokens()
Validation:
├─ Caller must be Governance Program
├─ ERC validated for trading
└─ Token supply limits not exceeded

Security Checks:
├─ invoke_signed() with Governance PDA as signer
├─ Energy-Token validates caller program ID
└─ Recipient account validation
```

##### 4. Trading → SPL Token Program
```rust
// Trading Program calls SPL Token Program
Authority: Trading Program ID
Target: SPL Token transfer/mint/burn functions
Validation:
├─ Caller must be Trading Program
├─ Sufficient token balances
└─ Valid token accounts

Security Checks:
├─ invoke_signed() with Trading PDA as signer
├─ SPL Token Program validates signatures
└─ Token account ownership verification
```

### Unauthorized CPI Prevention

#### Blocked Cross-Program Calls
```rust
// These calls are NOT authorized:

❌ User → Oracle.submit_meter_reading()
   // Only API Gateway can submit readings

❌ Any Program → Governance.emergency_pause() 
   // Only Governance Authority can pause

❌ User → Trading.match_orders()
   // Only Market Authority can trigger matching

❌ External Program → Any Internal Program
   // Only authorized programs can make CPIs
```

---

## Emergency Override Procedures

### Emergency Pause Authority

#### Activation Conditions
```rust
Emergency Pause Triggers:
├─ Security breach detected
├─ Smart contract vulnerability discovered  
├─ Regulatory compliance requirement
├─ Market manipulation suspected
├─ System integrity compromise
└─ External infrastructure failure
```

#### Emergency Pause Effects
```rust
When Emergency Paused:
├─ All trading functions disabled
│  ├─ create_sell_order() → Blocked
│  ├─ create_buy_order() → Blocked
│  ├─ match_orders() → Blocked
│  └─ cancel_order() → Allowed (Recovery)
├─ Token transfers restricted
│  ├─ transfer_tokens() → Blocked
│  └─ burn_tokens() → Allowed (User Protection)
├─ ERC issuance halted
│  └─ issue_erc() → Blocked
├─ Registry functions limited
│  ├─ register_user() → Blocked
│  ├─ register_meter() → Blocked
│  └─ update_meter_reading() → Allowed (Continuity)
└─ Oracle functions maintained
   ├─ submit_meter_reading() → Allowed
   └─ trigger_market_clearing() → Allowed
```

#### Recovery Procedures
```rust
Emergency Recovery Steps:
1. Authority investigates the emergency trigger
2. Issue is identified and resolved
3. System integrity is verified
4. emergency_unpause() is called
5. All functions return to normal operation
6. Post-incident analysis is conducted

Recovery Validation:
├─ No active threats detected
├─ All system components operational
├─ Data integrity confirmed
└─ Regulatory compliance maintained
```

### Maintenance Mode

#### Maintenance Mode Effects
```rust
When Maintenance Mode Active:
├─ User registration disabled
├─ New meter registration disabled
├─ ERC issuance paused (non-emergency)
├─ Market parameter updates allowed
├─ Emergency functions remain available
└─ Read operations continue normally

Purpose:
├─ System upgrades
├─ Configuration changes
├─ Performance optimization
└─ Routine maintenance
```

---

## Security Implementation Details

### Signature Verification

#### Account Signature Requirements
```rust
// Program Authority Signatures
Oracle Authority: Required for admin functions
Governance Authority: Required for PoA operations
Registry Authority: Required for user management
Energy-Token Authority: Required for token setup
Trading Authority: Required for market operations

// User Signatures
User Registration: Self-signature required
Meter Registration: Meter owner signature required
Order Creation: Order creator signature required  
Order Cancellation: Order owner signature required
Token Transfers: Token owner signature required

// Cross-Program Signatures
CPI Calls: Program PDA signatures via invoke_signed()
SPL Token Calls: Program authority signatures
System Program Calls: Program PDA signatures
```

### Access Control Implementation

#### Runtime Permission Checks
```rust
// Example: Oracle Function Authorization
pub fn submit_meter_reading(
    ctx: Context<SubmitMeterReading>,
    meter_id: String,
    reading_data: MeterReadingData,
) -> Result<()> {
    // 1. Verify caller is authorized API Gateway
    let oracle_data = &ctx.accounts.oracle_data;
    require_keys_eq!(
        ctx.accounts.api_gateway.key(),
        oracle_data.api_gateway,
        ErrorCode::UnauthorizedApiGateway
    );
    
    // 2. Verify Oracle is active
    require!(
        oracle_data.active,
        ErrorCode::OracleInactive
    );
    
    // 3. Verify emergency pause not active
    let governance_data = &ctx.accounts.governance_data;
    require!(
        !governance_data.emergency_paused,
        ErrorCode::SystemEmergencyPaused
    );
    
    // Function implementation...
    Ok(())
}

// Example: User Function Authorization  
pub fn register_user(
    ctx: Context<RegisterUser>,
    user_type: UserType,
    location: String,
) -> Result<()> {
    // 1. Verify user is signing for themselves
    require_keys_eq!(
        ctx.accounts.user.key(),
        ctx.accounts.user_account.owner,
        ErrorCode::UnauthorizedUser
    );
    
    // 2. Verify user account doesn't already exist
    require!(
        ctx.accounts.user_account.data_is_empty(),
        ErrorCode::UserAlreadyRegistered
    );
    
    // Function implementation...
    Ok(())
}
```

#### PDA Authority Verification
```rust
// Example: Cross-Program Call Authorization
pub fn issue_erc_from_registry(
    ctx: Context<IssueErcFromRegistry>,
    certificate_id: String,
    energy_amount: u64,
) -> Result<()> {
    // 1. Verify caller is Registry Program
    let caller_program = ctx.accounts.registry_program.key();
    require_keys_eq!(
        caller_program,
        REGISTRY_PROGRAM_ID,
        ErrorCode::UnauthorizedCrossProgram
    );
    
    // 2. Verify Registry PDA signature
    let seeds = &[b"registry"];
    let (expected_pda, _) = Pubkey::find_program_address(
        seeds, 
        &REGISTRY_PROGRAM_ID
    );
    require_keys_eq!(
        ctx.accounts.registry_pda.key(),
        expected_pda,
        ErrorCode::InvalidRegistryPda
    );
    
    // Function implementation...
    Ok(())
}
```

### Security Error Codes

#### Access Control Error Definitions
```rust
#[error_code]
pub enum SecurityError {
    #[msg("Unauthorized: Function requires Authority signature")]
    UnauthorizedAuthority,
    
    #[msg("Unauthorized: Function requires API Gateway signature")]
    UnauthorizedApiGateway,
    
    #[msg("Unauthorized: Function requires User signature")]
    UnauthorizedUser,
    
    #[msg("Unauthorized: Cross-program call not allowed")]
    UnauthorizedCrossProgram,
    
    #[msg("System paused: Emergency pause is active")]
    SystemEmergencyPaused,
    
    #[msg("System maintenance: Maintenance mode is active")]
    SystemMaintenanceMode,
    
    #[msg("Invalid PDA: Program derived address validation failed")]
    InvalidPda,
    
    #[msg("Account ownership: Account not owned by expected program")]
    InvalidAccountOwnership,
    
    #[msg("Signature verification: Required signature missing or invalid")]
    InvalidSignature,
    
    #[msg("Authority verification: Authority account validation failed")]
    InvalidAuthority,
}
```

---

**[← Back to Architecture Overview](./README.md)**