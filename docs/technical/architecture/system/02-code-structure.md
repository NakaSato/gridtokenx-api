# 📋 Code Structure

**GridTokenX Platform - Program Internal Structure & Components**

> **📘 For the most comprehensive and up-to-date code structure documentation, see [Anchor Architecture Overview](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md)**

---

## Table of Contents

1. [Registry Program Structure](#registry-program-structure)
2. [Energy Token Program Structure](#energy-token-program-structure)
3. [Oracle Program Structure](#oracle-program-structure)
4. [Trading Program Structure](#trading-program-structure)
5. [Governance Program Structure](#governance-program-structure)

---

## Registry Program Structure

> **📘 Complete documentation: [Registry Program](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#registry-program)**

```
registry/src/lib.rs
├── Module Declaration
│   └── mod registry { ... }
│
├── Program ID
│   └── declare_id!("Bxvy5YBKoADe1BSTnj4cd117RLzfjUKG2WEk2iqcmVJE")
│
├── Functions (7)
│   ├── fn initialize(ctx) -> Result<()>
│   ├── fn register_user(ctx, user_type, location) -> Result<()>
│   ├── fn register_meter(ctx, meter_id, meter_type) -> Result<()>
│   ├── fn update_user_status(ctx, new_status) -> Result<()>
│   ├── fn update_meter_reading(ctx, gen, cons, timestamp) -> Result<()>
│   ├── fn settle_meter_balance(ctx) -> Result<u64>
│   └── fn get_unsettled_balance(ctx) -> Result<u64>
│
├── Account Structs (7)
│   ├── struct Initialize<'info> { ... }
│   ├── struct RegisterUser<'info> { ... }
│   ├── struct RegisterMeter<'info> { ... }
│   ├── struct UpdateUserStatus<'info> { ... }
│   ├── struct UpdateMeterReading<'info> { ... }
│   ├── struct SettleMeterBalance<'info> { ... }
│   └── struct GetUnsettledBalance<'info> { ... }
│
├── Data Structs (3)
│   ├── struct Registry { ... }
│   ├── struct UserAccount { ... }
│   └── struct MeterAccount { ... }
│
├── Enums (4)
│   ├── enum UserType { Prosumer, Consumer }
│   ├── enum UserStatus { Active, Suspended, Inactive }
│   ├── enum MeterType { Solar, Wind, Battery, Grid }
│   └── enum MeterStatus { Active, Inactive, Maintenance }
│
├── Events (6)
│   ├── struct RegistryInitialized
│   ├── struct UserRegistered
│   ├── struct MeterRegistered
│   ├── struct UserStatusUpdated
│   ├── struct MeterReadingUpdated
│   └── struct MeterBalanceSettled
│
└── Error Codes (8)
    ├── UnauthorizedUser
    ├── UnauthorizedAuthority
    ├── InvalidUserStatus
    ├── InvalidMeterStatus
    ├── UserNotFound
    ├── MeterNotFound
    ├── NoUnsettledBalance
    └── InvalidMeterReading
```

### Registry Functions Detail

#### `register_user(ctx, user_type, location)`
- **Purpose**: Register new users in the system
- **Authority**: Any user (self-registration)
- **Parameters**: `UserType` (Prosumer/Consumer), location string
- **Creates**: UserAccount PDA with seeds `["user", authority]`

#### `register_meter(ctx, meter_id, meter_type)`
- **Purpose**: Register energy meters to user accounts
- **Authority**: Account owner
- **Parameters**: Unique meter ID, meter type (Solar/Wind/Battery/Grid)
- **Creates**: MeterAccount PDA with seeds `["meter", meter_id]` linked to user

#### `update_meter_reading(ctx, energy_produced, energy_consumed, timestamp)`
- **Purpose**: Update meter readings (called via CPI from Oracle)
- **Authority**: Oracle Program
- **Effect**: Updates meter totals and last reading timestamp
- **Security**: Only accessible via CPI, not direct invocation

#### `settle_meter_balance(ctx)`
- **Purpose**: Calculate unsettled energy for token minting
- **Authority**: Energy Token Program (CPI)
- **Returns**: Amount of tokens to mint
- **Updates**: `settled_net_generation` to prevent double-minting
- **Security**: Critical for preventing double-spend

> **📘 See [Double-Spend Prevention](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#double-spend-prevention) for security details**

---

## Energy Token Program Structure

> **📘 Complete documentation: [Energy Token Program](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#energy-token-program)**

```
energy-token/src/lib.rs
├── Module Declaration
│   └── mod energy_token { ... }
│
├── Program ID
│   └── declare_id!("6LgvcJ8pxzSbzWCdaTWB2gUg4WazJv46eSjzj6LCNjNd")
│
├── Functions (4)
│   ├── fn initialize_token(ctx) -> Result<()>
│   ├── fn mint_grid_tokens(ctx) -> Result<()>
│   ├── fn transfer_tokens(ctx, amount) -> Result<()>
│   └── fn burn_tokens(ctx, amount) -> Result<()>
│
├── Account Structs (4)
│   ├── struct InitializeToken<'info> { ... }
│   ├── struct MintGridTokens<'info> { ... }
│   ├── struct TransferTokens<'info> { ... }
│   └── struct BurnTokens<'info> { ... }
│
├── Data Struct (1)
│   └── struct TokenInfo { ... }
│
├── Events (4)
│   ├── struct TokenInitialized
│   ├── struct GridTokensMinted
│   ├── struct TokensTransferred
│   └── struct TokensBurned
│
└── Error Codes (6)
    ├── UnauthorizedAuthority
    ├── InvalidAmount
    ├── InsufficientBalance
    ├── TokenNotInitialized
    ├── InvalidMint
    └── MintFailed
```

### Energy Token Functions Detail

#### `initialize_token(ctx)`
- **Purpose**: Create SPL token mint for GRID tokens
- **Authority**: System Administrator
- **Configuration**: 9 decimals, mint authority = TokenInfo PDA
- **Integration**: SPL Token Program
- **Creates**: TokenInfo PDA with seeds `["token_info"]`

#### `mint_grid_tokens(ctx)`
- **Purpose**: Mint GRID tokens for validated energy generation
- **Authority**: Meter owner
- **Process**: 
  1. CPI to Registry.settle_meter_balance()
  2. Get amount of tokens to mint
  3. CPI to SPL Token Program with PDA signer
  4. Update total_supply
- **Security**: PDA signing prevents unauthorized minting
- **Double-Mint Prevention**: Uses Registry's `settled_net_generation` tracker

> **📘 See [Token Minting Flow](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml#ANCHOR_SEQUENCE_TOKEN_MINTING) for complete sequence**

#### `transfer_tokens(ctx, amount)`
- **Purpose**: Transfer GRID tokens between accounts
- **Authority**: Token holder
- **Integration**: SPL Token transfer operations
- **Use Cases**: Trading, transfers, payments

#### `burn_tokens(ctx, amount)`
- **Purpose**: Burn GRID tokens
- **Authority**: Token holder
- **Integration**: SPL Token burn operations
- **Effect**: Reduces total token supply

---

## Oracle Program Structure

> **📘 Complete documentation: [Oracle Program](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#oracle-program)**

## Oracle Program Structure

> **📘 Complete documentation: [Oracle Program](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#oracle-program)**

```
oracle/src/lib.rs
├── Module Declaration
│   └── mod oracle { ... }
│
├── Program ID
│   └── declare_id!("2Jqh9JkxpJuWyqdzSDv3gskgMN9fT4K73P88a6xYAy4i")
│
├── Functions (5)
│   ├── fn initialize(ctx, api_gateway) -> Result<()>
│   ├── fn submit_meter_reading(ctx, meter_id, ...) -> Result<()>
│   ├── fn trigger_market_clearing(ctx) -> Result<()>
│   ├── fn update_oracle_status(ctx, active) -> Result<()>
│   └── fn update_api_gateway(ctx, new_api_gateway) -> Result<()>
│
├── Account Structs (5)
│   ├── struct Initialize<'info> { ... }
│   ├── struct SubmitMeterReading<'info> { ... }
│   ├── struct TriggerMarketClearing<'info> { ... }
│   ├── struct UpdateOracleStatus<'info> { ... }
│   └── struct UpdateApiGateway<'info> { ... }
│
├── Data Struct (1)
│   └── struct OracleData { ... }
│
├── Events (4)
│   ├── struct MeterReadingSubmitted
│   ├── struct MarketClearingTriggered
│   ├── struct OracleStatusUpdated
│   └── struct ApiGatewayUpdated
│
└── Error Codes (5)
    ├── UnauthorizedAuthority
    ├── UnauthorizedGateway
    ├── OracleInactive
    ├── InvalidMeterReading
    └── MarketClearingInProgress
```

### Oracle Functions Detail

#### `initialize(ctx, api_gateway)`
- **Purpose**: Initialize Oracle program with API Gateway authorization
- **Authority**: System Administrator
- **Parameters**: `api_gateway: Pubkey`
- **Creates**: OracleData PDA with seeds `["oracle_data"]`

#### `submit_meter_reading(ctx, meter_id, energy_produced, energy_consumed, timestamp)`
- **Purpose**: Submit AMI meter readings for processing
- **Authority**: API Gateway only (validated by pubkey)
- **Parameters**: Meter data and timestamp
- **Process**:
  1. Verify signer == api_gateway
  2. Verify oracle.active == true
  3. CPI to Registry.update_meter_reading()
  4. Update oracle statistics
  5. Emit event
- **Security**: Only authorized API Gateway can submit

> **📘 See [Oracle → Registry CPI Pattern](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml#ANCHOR_CPI_PATTERNS) for details**

#### `trigger_market_clearing(ctx)`
- **Purpose**: Initiate market clearing/matching process
- **Authority**: API Gateway only
- **Effect**: Signals Trading program for order matching
- **Use Case**: Periodic market clearing (e.g., hourly)

#### `update_oracle_status(ctx, active)`
- **Purpose**: Enable/disable Oracle operations
- **Authority**: System Administrator
- **Parameters**: `active: bool`
- **Use Case**: Maintenance mode

#### `update_api_gateway(ctx, new_api_gateway)`
- **Purpose**: Update authorized API Gateway address
- **Authority**: System Administrator
- **Parameters**: `new_api_gateway: Pubkey`
- **Security**: Critical for access control

---

## Trading Program Structure

> **📘 Complete documentation: [Trading Program](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#trading-program)**

```
trading/src/lib.rs
├── Module Declaration
│   └── mod trading { ... }
│
├── Program ID
│   └── declare_id!("Hzmt59DnHUKa8h8MJADgAf4zjREhvwZXW5ew5gnTnFPH")
│
├── Functions (6)
│   ├── fn initialize_market(ctx) -> Result<()>
│   ├── fn create_sell_order(ctx, amount, price) -> Result<()>
│   ├── fn create_buy_order(ctx, amount, max_price) -> Result<()>
│   ├── fn match_orders(ctx) -> Result<()>
│   ├── fn cancel_order(ctx) -> Result<()>
│   └── fn update_market_fee(ctx, new_fee_bps) -> Result<()>
│
├── Account Structs (6)
│   ├── struct InitializeMarket<'info> { ... }
│   ├── struct CreateSellOrder<'info> { ... }
│   ├── struct CreateBuyOrder<'info> { ... }
│   ├── struct MatchOrders<'info> { ... }
│   ├── struct CancelOrder<'info> { ... }
│   └── struct UpdateMarketFee<'info> { ... }
│
├── Data Structs (3)
│   ├── struct Market { ... }
│   ├── struct Order { ... }
│   └── struct TradeRecord { ... }
│
├── Enums (2)
│   ├── enum OrderType { Sell, Buy }
│   └── enum OrderStatus { Active, PartiallyFilled, Completed, Cancelled, Expired }
│
├── Events (6)
│   ├── struct MarketInitialized
│   ├── struct SellOrderCreated
│   ├── struct BuyOrderCreated
│   ├── struct OrderMatched
│   ├── struct OrderCancelled
│   └── struct MarketFeeUpdated
│
└── Error Codes (10)
    ├── UnauthorizedAuthority
    ├── InvalidAmount
    ├── InvalidPrice
    ├── InactiveSellOrder
    ├── InactiveBuyOrder
    ├── PriceMismatch
    ├── OrderNotCancellable
    ├── InsufficientEscrowBalance
    ├── MarketNotActive
    └── InvalidOrderStatus
```

### Trading Functions Detail

#### `create_sell_order(ctx, amount, price_per_kwh)`
- **Purpose**: Create energy sell orders with escrow
- **Authority**: Token holders (users)
- **Process**:
  1. Create Order PDA with seeds `["order", order_id]`
  2. CPI to Energy Token: transfer tokens to escrow
  3. Initialize order with Active status
  4. Update market statistics
- **Escrow**: Locks GRID tokens until order completion/cancellation
- **Creates**: Order PDA with seller details

> **📘 See [P2P Trading Flow](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml#ANCHOR_SEQUENCE_P2P_TRADING) for complete sequence**

#### `create_buy_order(ctx, amount, max_price_per_kwh)`
- **Purpose**: Create energy buy orders with payment escrow
- **Authority**: Token holders (users)
- **Process**: Similar to sell order but escrows payment tokens
- **Escrow**: Locks payment (GRID tokens or SOL) until order completion
- **Creates**: Order PDA with buyer details

#### `match_orders(ctx)`
- **Purpose**: Execute compatible buy/sell order pairs
- **Authority**: Market makers or automated system
- **Process**:
  1. Verify price compatibility (buy_price >= sell_price)
  2. Calculate match amount and fees (0.25% = 25 bps)
  3. CPI to Energy Token: atomic multi-transfer
     - Transfer to seller (amount - fee)
     - Transfer fee to platform
     - Return excess to buyer
  4. Update order statuses
  5. Create TradeRecord PDA
  6. Update market statistics
- **Security**: Atomic settlement prevents partial execution
- **Creates**: TradeRecord for audit trail

#### `cancel_order(ctx)`
- **Purpose**: Cancel active orders and return escrow
- **Authority**: Order creator
- **Process**:
  1. Verify order is cancellable (Active or PartiallyFilled)
  2. CPI to Energy Token: return escrowed tokens
  3. Update order status to Cancelled
- **Effect**: Returns escrowed tokens to user

---

## Governance Program Structure

> **📘 Complete documentation: [Governance Program](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#governance-program)**

## Governance Program Structure

> **📘 Complete documentation: [Governance Program](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#governance-program)**

```
governance/src/lib.rs
├── Module Declaration
│   └── mod governance { ... }
│
├── Program ID
│   └── declare_id!("83V1DXgURKYkPURCJbBKU3VzkqVjYcPKDuL6DRLKAGvw")
│
├── Functions (10)
│   ├── fn initialize_poa(ctx, authority_name, contact_info) -> Result<()>
│   ├── fn issue_erc(ctx, cert_id, energy_amt, ...) -> Result<()>
│   ├── fn validate_erc_for_trading(ctx) -> Result<()>
│   ├── fn revoke_erc(ctx, reason) -> Result<()>
│   ├── fn emergency_pause(ctx, reason) -> Result<()>
│   ├── fn emergency_unpause(ctx) -> Result<()>
│   ├── fn set_maintenance_mode(ctx, enabled) -> Result<()>
│   ├── fn update_erc_validation(ctx, enabled) -> Result<()>
│   ├── fn update_erc_limits(ctx, min, max, period) -> Result<()>
│   └── fn update_authority_info(ctx, name, contact) -> Result<()>
│
├── Account Structs (10)
│   ├── struct InitializePoa<'info> { ... }
│   ├── struct IssueErc<'info> { ... }
│   ├── struct ValidateErcForTrading<'info> { ... }
│   ├── struct RevokeErc<'info> { ... }
│   ├── struct EmergencyPause<'info> { ... }
│   ├── struct EmergencyUnpause<'info> { ... }
│   ├── struct SetMaintenanceMode<'info> { ... }
│   ├── struct UpdateErcValidation<'info> { ... }
│   ├── struct UpdateErcLimits<'info> { ... }
│   └── struct UpdateAuthorityInfo<'info> { ... }
│
├── Data Structs (2)
│   ├── struct PoAConfig { ... }  // 474 bytes
│   └── struct ErcCertificate { ... }  // 459 bytes
│
├── Enums (1)
│   └── enum ErcStatus { Valid, Expired, Revoked, Pending }
│
├── Events (11)
│   ├── struct PoAInitialized
│   ├── struct ErcIssued
│   ├── struct ErcValidatedForTrading
│   ├── struct ErcRevoked
│   ├── struct EmergencyPauseActivated
│   ├── struct EmergencyPauseDeactivated
│   ├── struct MaintenanceModeUpdated
│   ├── struct ErcValidationUpdated
│   ├── struct ErcLimitsUpdated
│   ├── struct AuthorityInfoUpdated
│   └── struct PoaConfigUpdated
│
└── Error Codes (18)
    ├── UnauthorizedAuthority
    ├── SystemPaused
    ├── MaintenanceMode
    ├── AlreadyPaused
    ├── NotPaused
    ├── ErcValidationDisabled
    ├── InvalidAmount
    ├── InsufficientAvailableEnergy
    ├── BelowMinimumEnergy
    ├── ExceedsMaximumEnergy
    ├── InvalidErcStatus
    ├── AlreadyValidated
    ├── CertificateExpired
    ├── CertificateRevoked
    ├── CertificateIdTooLong
    ├── SourceNameTooLong
    ├── ValidationDataTooLong
    └── InvalidValidityPeriod
```

### Governance Functions Detail

#### `initialize_poa(ctx, authority_name, contact_info)`
- **Purpose**: Initialize Proof of Authority governance
- **Authority**: REC Authority (Engineering Department)
- **Parameters**: Authority name and contact information
- **Creates**: PoAConfig PDA with seeds `["poa_config"]`
- **Configuration**: Sets initial ERC limits, validity periods

#### `issue_erc(ctx, certificate_id, energy_amount, renewable_source, validation_data)`
- **Purpose**: Issue Energy Renewable Certificates
- **Authority**: REC Authority only
- **Process**:
  1. Verify system operational (not paused, not maintenance)
  2. Verify ERC validation enabled
  3. Check energy amount within limits (min/max)
  4. Read MeterAccount from Registry
  5. Verify available energy (total_generation - claimed_erc_generation)
  6. Create ErcCertificate PDA
  7. Update MeterAccount: claimed_erc_generation += amount
  8. Update PoAConfig statistics
- **Creates**: ErcCertificate PDA with seeds `["erc_certificate", cert_id]`
- **Security**: Prevents double-claiming via `claimed_erc_generation` tracker

> **📘 See [ERC Certification Flow](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml#ANCHOR_SEQUENCE_ERC_CERTIFICATION) for complete sequence**

#### `validate_erc_for_trading(ctx)`
- **Purpose**: Validate ERC for trading eligibility
- **Authority**: REC Authority only
- **Process**:
  1. Verify ERC status is Valid
  2. Verify not expired
  3. Set validated_for_trading = true
  4. Record trading_validated_at timestamp
  5. Update PoAConfig statistics
- **Effect**: Enables use of ERC in trading marketplace
- **Use Case**: Required before ERC-backed trading

#### `emergency_pause(ctx, reason)` / `emergency_unpause(ctx)`
- **Purpose**: System-wide emergency controls
- **Authority**: REC Authority only
- **Scope**: Blocks all critical operations across programs
- **Use Case**: Security incidents, maintenance, regulatory compliance
- **Parameters**: Optional reason string for pause

> **📘 See [Emergency Controls](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#emergency-controls) for system-wide impact**

#### `update_erc_limits(ctx, min_energy_amount, max_erc_amount, validity_period)`
- **Purpose**: Update ERC issuance constraints
- **Authority**: REC Authority only
- **Parameters**: Minimum energy, maximum per ERC, validity period
- **Validation**: Ensures min < max, validity > 0
- **Use Case**: Policy adjustments

---

## Common Patterns

### Account Validation
All programs implement consistent patterns for:
- **Authority Checks**: Verify caller permissions via `require!(signer == authority)`
- **Account Existence**: Ensure required accounts exist and initialized
- **Status Validation**: Check account states (active, suspended, etc.)
- **Business Rules**: Enforce program-specific constraints

### Cross-Program Invocations (CPI)
- **PDA Signing**: Programs use PDA-derived signers for secure CPI
- **Account Validation**: Verify account ownership and program ownership
- **Error Propagation**: Handle CPI errors appropriately
- **Atomic Operations**: CPI calls are atomic with parent transaction

> **📘 See [CPI Security Patterns](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#cpi-security) for implementation details**

### Error Handling
- **Descriptive Errors**: Meaningful error codes for debugging
- **Security First**: Authorization errors prevent unauthorized access
- **Input Validation**: Parameter validation with specific error messages
- **State Consistency**: Prevents invalid state transitions

### Event Emission
- **Comprehensive Logging**: All state changes emit events
- **Off-chain Processing**: Events enable external system integration
- **Audit Trail**: Complete transaction history via events
- **Real-time Updates**: Client applications can listen for events

### Double-Spend Prevention
- **Token Minting**: `settled_net_generation` in MeterAccount tracks minted tokens
- **ERC Claiming**: `claimed_erc_generation` in MeterAccount tracks claimed ERCs
- **Trading Escrow**: Atomic multi-transfer prevents partial execution
- **Order States**: Order status transitions prevent replay attacks

> **📘 See [Security Model](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#security-model) for comprehensive security analysis**

---

## 📚 Related Documentation

- **[Anchor Architecture Overview](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md)** - Complete technical reference
- **[Anchor Quick Reference](../anchor/ANCHOR_QUICK_REFERENCE.md)** - Instruction quick lookup
- **[Architecture Diagrams](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml)** - Visual program structure
- **[System Architecture](./01-system-architecture.md)** - High-level design
- **[Data Flow Diagrams](./03-data-flow-diagrams.md)** - Function-level flows

---

**[← Back to Architecture Overview](./README.md)**