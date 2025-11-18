# 📚 Anchor Programs Documentation

**GridTokenX Platform - Smart Contract Programs Architecture**

---

## Table of Contents

1. [Overview](#overview)
2. [Oracle Program](#oracle-program)
3. [Governance Program](#governance-program)
4. [Registry Program](#registry-program)
5. [Energy-Token Program](#energy-token-program)
6. [Trading Program](#trading-program)
7. [Program Interaction Flow](#program-interaction-flow)
8. [Security Considerations](#security-considerations)

---

## Overview

The GridTokenX platform consists of 5 interconnected Anchor programs that work together to create a peer-to-peer (P2P) energy trading system on the Solana blockchain. Each program has a specific responsibility in the ecosystem.

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    GridTokenX Platform                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐      ┌──────────────┐     ┌──────────────┐   │
│  │   Oracle     │      │  Governance  │     │   Registry   │   │
│  │  Program     │      │   Program    │     │   Program    │   │
│  └──────────────┘      └──────────────┘     └──────────────┘   │
│         △                     △                    △             │
│         │                     │                    │             │
│         └─────────────────────┼────────────────────┘             │
│                               │                                 │
│                          ┌────▼────────┐                        │
│                          │   Trading    │                        │
│                          │   Program    │                        │
│                          └──────────────┘                        │
│                               △                                 │
│                               │                                 │
│                        ┌──────▼──────┐                         │
│                        │  Energy     │                         │
│                        │  Token      │                         │
│                        │  Program    │                         │
│                        └─────────────┘                         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

# Oracle Program

**Program ID:** `ApwexmUbEZMpez5dJXKza4V7gqSqWvAA9BPbok2psxXg`

## Purpose

The Oracle program acts as the **data input layer** for the GridTokenX platform. It securely receives meter readings from AMI (Advanced Metering Infrastructure) systems via an API Gateway and triggers market clearing operations.

### Key Responsibilities
- Receive meter reading data (energy produced/consumed)
- Validate API Gateway authorization
- Trigger market clearing processes
- Maintain oracle status and statistics

## Core Functions

### 1. `initialize(api_gateway: Pubkey)`
**Access:** Authority (admin)

Initializes the oracle program with configuration.

```rust
Parameters:
  - api_gateway: Pubkey - The API Gateway address authorized to submit readings

Initializes:
  - Authority: The caller's account
  - API Gateway: The external data source
  - Total readings counter: 0
  - Active status: true
  - Timestamps for creation
```

**Events Emitted:**
```
Oracle program initialized with API Gateway: {api_gateway}
```

### 2. `submit_meter_reading(meter_id, energy_produced, energy_consumed, reading_timestamp)`
**Access:** API Gateway only (enforced)

Submits AMI meter reading data to the blockchain.

```rust
Parameters:
  - meter_id: String - Unique meter identifier
  - energy_produced: u64 - Energy generated (kWh)
  - energy_consumed: u64 - Energy used (kWh)
  - reading_timestamp: i64 - When the reading was taken

Updates:
  - Increments total_readings counter
  - Records last_reading_timestamp
  - Emits MeterReadingSubmitted event
```

**Security Checks:**
- ✅ Oracle must be active
- ✅ Only API Gateway can call this function
- ✅ Timestamp validation

### 3. `trigger_market_clearing()`
**Access:** API Gateway only (enforced)

Triggers the market clearing process for energy trading.

```rust
Effects:
  - Records the current timestamp as last_clearing
  - Emits MarketClearingTriggered event
  - Notifies trading system to settle trades
```

### 4. `update_oracle_status(active: bool)`
**Access:** Authority (admin)

Enables or disables the oracle.

```rust
Parameters:
  - active: bool - Enable/disable oracle

Use Cases:
  - Maintenance: Disable oracle for updates
  - Emergency: Pause oracle in crisis
```

### 5. `update_api_gateway(new_api_gateway: Pubkey)`
**Access:** Authority (admin)

Updates the authorized API Gateway address.

```rust
Parameters:
  - new_api_gateway: Pubkey - New gateway address

Effects:
  - Old gateway loses authorization
  - New gateway gains authorization
  - Previous gateway address recorded for auditing
```

## Data Structures

### OracleData Account
```rust
{
  authority: Pubkey,              // Admin who controls oracle
  api_gateway: Pubkey,            // Authorized data source
  total_readings: u64,            // Total submissions received
  last_reading_timestamp: i64,    // Most recent reading time
  last_clearing: i64,             // Last market clearing time
  active: bool,                   // Is oracle operational
  created_at: i64,                // Initialization timestamp
}
```

## Events

| Event | Triggered By | Contains |
|-------|--------------|----------|
| `MeterReadingSubmitted` | submit_meter_reading | meter_id, energy_produced, energy_consumed, timestamp, submitter |
| `MarketClearingTriggered` | trigger_market_clearing | authority, timestamp |
| `OracleStatusUpdated` | update_oracle_status | authority, active status, timestamp |
| `ApiGatewayUpdated` | update_api_gateway | old_gateway, new_gateway, authority, timestamp |

## Error Codes

```rust
UnauthorizedAuthority       // Caller is not the admin
UnauthorizedGateway         // Caller is not the API Gateway
OracleInactive             // Oracle is disabled
InvalidMeterReading        // Reading data invalid
MarketClearingInProgress   // Clearing already in progress
```

---

# Governance Program

**Program ID:** `Dy8JFn95L1E7NoUkXbFQtW1kGR7Ja21CkNcirNgv4ghe`

## Purpose

The Governance program implements a **Proof-of-Authority (PoA)** system where the University Engineering Department has sole authority to:
- Issue Energy Renewable Certificates (ERCs)
- Validate certificates for trading
- Control system parameters

### Key Responsibilities
- ERC (Energy Renewable Certificate) issuance
- ERC validation for trading
- Emergency pause/unpause functionality
- Governance parameter management
- System maintenance mode control

## Core Functions

### 1. `initialize_poa()`
**Access:** Authority (admin)

Initializes the PoA governance system.

```rust
Initializes:
  - Authority: University Engineering Department
  - Authority name: "University Engineering Department"
  - Contact info: "engineering_erc@utcc.ac.th"
  - ERC Validation: Enabled
  - Max ERC amount: 1,000,000 kWh per transaction
  - Min energy amount: 100 kWh
  - ERC validity period: 1 year (31,536,000 seconds)
  - Emergency paused: false
  - Maintenance mode: false
```

**Constraints:**
- Single authority implementation (not multi-sig)
- Engineering Department has ultimate control

### 2. `issue_erc(certificate_id, energy_amount, renewable_source, validation_data)`
**Access:** Authority only

Issues a new Energy Renewable Certificate.

```rust
Parameters:
  - certificate_id: String - Unique certificate identifier (max 64 chars)
  - energy_amount: u64 - Energy amount in kWh
  - renewable_source: String - "Solar", "Wind", "Hydro", etc. (max 64 chars)
  - validation_data: String - Additional validation information

Validation:
  ✓ System not paused
  ✓ Not in maintenance mode
  ✓ ERC validation enabled
  ✓ energy_amount >= min_energy_amount (100 kWh)
  ✓ energy_amount <= max_erc_amount (1,000,000 kWh)

Creates:
  - ErcCertificate account
  - Status: Valid
  - Expires at: current_time + erc_validity_period (1 year)
  - validated_for_trading: false (needs validation step)
```

**Events Emitted:**
```
ErcIssued {
  certificate_id,
  authority,
  energy_amount,
  renewable_source,
  timestamp
}
```

### 3. `validate_erc_for_trading()`
**Access:** Authority only

Validates an ERC for trading on the platform.

```rust
Preconditions:
  - System not paused
  - Not in maintenance mode
  - ERC status == Valid
  - ERC not already validated
  - ERC not expired

Effects:
  - Sets validated_for_trading = true
  - Records trading_validated_at timestamp
  - Increments total_ercs_validated
  - Certificate can now be traded

Note: This is a separate step from issuance for additional control
```

### 4. `emergency_pause()`
**Access:** Authority only

Activates emergency pause (system-wide freeze).

```rust
Effects:
  - emergency_paused = true
  - Records emergency_timestamp
  - Blocks all ERC issuance and validation
  - Blocks governance config updates

Use Cases:
  - Security incident response
  - Regulatory halt
  - System maintenance emergency
```

### 5. `emergency_unpause()`
**Access:** Authority only

Deactivates emergency pause.

```rust
Effects:
  - emergency_paused = false
  - Clears emergency_timestamp
  - Resumes normal operations
```

### 6. `update_governance_config(erc_validation_enabled: bool)`
**Access:** Authority only

Enables/disables ERC validation.

```rust
Parameters:
  - erc_validation_enabled: bool

Use Cases:
  - Temporarily stop certificate validation
  - Maintenance of validation system
```

### 7. `set_maintenance_mode(maintenance_enabled: bool)`
**Access:** Authority only

Puts system in/out of maintenance mode.

```rust
Effects:
  - Blocks ERC issuance and validation when enabled
  - Used for system updates and patches
```

### 8. `update_erc_limits(min_energy, max_erc, validity_period)`
**Access:** Authority only

Updates ERC constraints.

```rust
Parameters:
  - min_energy_amount: u64 - Minimum kWh per ERC
  - max_erc_amount: u64 - Maximum kWh per ERC
  - erc_validity_period: i64 - Validity in seconds

Validation:
  ✓ min_energy > 0
  ✓ max_erc > min_energy
  ✓ validity_period > 0
```

### 9. `update_authority_info(contact_info: String)`
**Access:** Authority only

Updates Engineering Department contact information.

```rust
Parameters:
  - contact_info: String - Contact details (max 128 chars)
```

### 10. `get_governance_stats()`
**Access:** Public (read-only)

Returns current governance statistics.

```rust
Returns:
  {
    total_ercs_issued: u64,
    total_ercs_validated: u64,
    erc_validation_enabled: bool,
    emergency_paused: bool,
    maintenance_mode: bool,
    min_energy_amount: u64,
    max_erc_amount: u64,
    erc_validity_period: i64,
    created_at: i64,
    last_updated: i64,
  }
```

## Data Structures

### PoAConfig Account
```rust
{
  authority: Pubkey,              // Engineering Department
  authority_name: String,         // "University Engineering Department"
  contact_info: String,           // Department contact info
  emergency_paused: bool,         // System pause status
  emergency_timestamp: Option<i64>, // When paused
  emergency_reason: Option<String>, // Why paused
  created_at: i64,
  last_updated: i64,
  erc_validation_enabled: bool,
  max_erc_amount: u64,            // 1M kWh default
  total_ercs_issued: u64,
  total_ercs_validated: u64,
  version: u8,
  delegation_enabled: bool,
  oracle_authority: Option<Pubkey>,
  min_energy_amount: u64,         // 100 kWh default
  erc_validity_period: i64,       // 1 year default
  maintenance_mode: bool,
}
```

### ErcCertificate Account
```rust
{
  certificate_id: String,         // Unique ID (max 64 chars)
  authority: Pubkey,              // Issuing authority
  energy_amount: u64,             // Amount in kWh
  renewable_source: String,       // "Solar", "Wind", etc.
  validation_data: String,        // Additional info
  issued_at: i64,
  expires_at: Option<i64>,
  status: ErcStatus,              // Valid | Expired | Revoked | Pending
  validated_for_trading: bool,
  trading_validated_at: Option<i64>,
}

enum ErcStatus {
  Valid,       // Active and tradable
  Expired,     // Past expiration
  Revoked,     // Revoked by authority
  Pending,     // Awaiting validation
}
```

## Events

| Event | Triggered By | Contains |
|-------|--------------|----------|
| `PoAInitialized` | initialize_poa | authority, authority_name, timestamp |
| `ErcIssued` | issue_erc | certificate_id, authority, energy_amount, renewable_source, timestamp |
| `ErcValidatedForTrading` | validate_erc_for_trading | certificate_id, authority, timestamp |
| `EmergencyPauseActivated` | emergency_pause | authority, timestamp |
| `EmergencyPauseDeactivated` | emergency_unpause | authority, timestamp |
| `GovernanceConfigUpdated` | update_governance_config | authority, erc_validation_enabled, old_enabled, timestamp |
| `MaintenanceModeUpdated` | set_maintenance_mode | authority, maintenance_enabled, timestamp |
| `ErcLimitsUpdated` | update_erc_limits | old/new min/max/validity, timestamp |
| `AuthorityInfoUpdated` | update_authority_info | authority, old_contact, new_contact, timestamp |

## Error Codes

```rust
UnauthorizedAuthority          // Not the Engineering Department
AlreadyPaused                  // Already paused
NotPaused                      // Not paused
SystemPaused                   // System is paused
MaintenanceMode                // System in maintenance
ErcValidationDisabled          // ERC validation disabled
InvalidErcStatus               // ERC not in Valid state
AlreadyValidated              // ERC already validated
BelowMinimumEnergy            // Amount < minimum
ExceedsMaximumEnergy          // Amount > maximum
CertificateIdTooLong          // ID > 64 chars
SourceNameTooLong             // Source > 64 chars
ErcExpired                    // Certificate expired
InvalidMinimumEnergy          // Invalid min value
InvalidMaximumEnergy          // Invalid max value
InvalidValidityPeriod         // Invalid period
ContactInfoTooLong            // Info > 128 chars
```

---

# Registry Program

**Program ID:** `42LoRKPphBBdvaCDx2ZjNuZFqzXuJziiiNXyiV6FhBY5`

## Purpose

The Registry program manages **user and smart meter registration** for the P2P energy trading system.

### Key Responsibilities
- User registration (Prosumers and Consumers)
- Smart meter registration and tracking
- User status management
- Meter reading updates
- User and meter validation

## Core Functions

### 1. `initialize()`
**Access:** Authority (admin)

Initializes the registry system.

```rust
Initializes:
  - Registry owner (university authority)
  - User counter: 0
  - Meter counter: 0
  - Creation timestamp
```

### 2. `register_user(user_type: UserType, location: String)`
**Access:** Any user (public)

Registers a new user in the system.

```rust
Parameters:
  - user_type: UserType - Prosumer or Consumer
  - location: String - User's location/address (max 100 chars)

Enum UserType:
  - Prosumer: Can produce AND consume energy
  - Consumer: Can only consume energy

Initializes UserAccount:
  - Authority: The registering user's pubkey
  - user_type: Prosumer or Consumer
  - location: User's location
  - status: Active
  - meter_count: 0
  - registered_at: Current timestamp

Effects:
  - Increments registry.user_count
  - Creates individual user account PDA (Program Derived Account)
```

**Events Emitted:**
```
UserRegistered {
  user: user_pubkey,
  user_type: UserType,
  location: location,
  timestamp: current_time,
}
```

### 3. `register_meter(meter_id: String, meter_type: MeterType)`
**Access:** Registered user (via signer)

Registers a smart meter for a user.

```rust
Parameters:
  - meter_id: String - Unique meter identifier (max 50 chars)
  - meter_type: MeterType - Type of meter

Enum MeterType:
  - Solar: Solar panel generation meter
  - Wind: Wind turbine meter
  - Battery: Battery storage meter
  - Grid: Grid connection meter

Validation:
  ✓ User must be the account signer
  ✓ User must exist in registry

Initializes MeterAccount:
  - meter_id: Provided identifier
  - owner: User's pubkey
  - meter_type: Solar | Wind | Battery | Grid
  - status: Active
  - total_generation: 0
  - total_consumption: 0

Effects:
  - Increments user.meter_count
  - Increments registry.meter_count
  - Creates meter account PDA
```

**Events Emitted:**
```
MeterRegistered {
  meter_id: meter_id,
  owner: user_pubkey,
  meter_type: MeterType,
  timestamp: current_time,
}
```

### 4. `update_meter_reading(energy_generated, energy_consumed, reading_timestamp)`
**Access:** Oracle/API Gateway (authorized)

Updates meter readings from oracle data.

```rust
Parameters:
  - energy_generated: u64 - Energy produced (kWh)
  - energy_consumed: u64 - Energy used (kWh)
  - reading_timestamp: i64 - When reading was taken

Updates MeterAccount:
  - last_reading_at: reading_timestamp
  - total_generation: += energy_generated
  - total_consumption: += energy_consumed

Events Emitted:
  MeterReadingUpdated {
    meter_id,
    owner,
    energy_generated,
    energy_consumed,
    timestamp,
  }
```

### 5. `update_user_status(new_status: UserStatus)`
**Access:** Registry authority only

Changes user's status (e.g., suspend account).

```rust
Parameters:
  - new_status: UserStatus

Enum UserStatus:
  - Active: Can trade normally
  - Suspended: Trading blocked
  - Inactive: Account disabled

Validation:
  ✓ Caller must be registry authority
```

### 6. `is_valid_user()`
**Access:** Public (read-only)

Checks if a user account is active.

```rust
Returns: bool (true if status == Active)
```

### 7. `is_valid_meter()`
**Access:** Public (read-only)

Checks if a meter is active.

```rust
Returns: bool (true if status == Active)
```

## Data Structures

### Registry Account
```rust
{
  authority: Pubkey,    // University authority (admin)
  user_count: u64,      // Total users registered
  meter_count: u64,     // Total meters registered
  created_at: i64,
}
```

### UserAccount PDA (seeds: [b"user", user_pubkey])
```rust
{
  authority: Pubkey,    // User's wallet address
  user_type: UserType,  // Prosumer or Consumer
  location: String,     // User's location (max 100 chars)
  status: UserStatus,   // Active | Suspended | Inactive
  registered_at: i64,
  meter_count: u32,     // Number of meters owned
  created_at: i64,      // Backward compatibility
}

enum UserType {
  Prosumer,  // Can produce and consume
  Consumer,  // Can only consume
}

enum UserStatus {
  Active,
  Suspended,
  Inactive,
}
```

### MeterAccount PDA (seeds: [b"meter", meter_id])
```rust
{
  meter_id: String,           // Unique ID (max 50 chars)
  owner: Pubkey,              // User who owns meter
  meter_type: MeterType,      // Solar | Wind | Battery | Grid
  status: MeterStatus,        // Active | Inactive | Maintenance
  registered_at: i64,
  last_reading_at: i64,
  total_generation: u64,      // Cumulative kWh produced
  total_consumption: u64,     // Cumulative kWh consumed
}

enum MeterType {
  Solar,
  Wind,
  Battery,
  Grid,
}

enum MeterStatus {
  Active,
  Inactive,
  Maintenance,
}
```

## Events

| Event | Triggered By | Contains |
|-------|--------------|----------|
| `RegistryInitialized` | initialize | authority, timestamp |
| `UserRegistered` | register_user | user, user_type, location, timestamp |
| `MeterRegistered` | register_meter | meter_id, owner, meter_type, timestamp |
| `UserStatusUpdated` | update_user_status | user, old_status, new_status, timestamp |
| `MeterReadingUpdated` | update_meter_reading | meter_id, owner, energy_generated, energy_consumed, timestamp |

## Error Codes

```rust
UnauthorizedUser           // User not authorized for operation
UnauthorizedAuthority      // Caller is not authority
InvalidUserStatus          // Invalid status value
InvalidMeterStatus         // Invalid meter status
UserNotFound              // User doesn't exist
MeterNotFound             // Meter doesn't exist
```

---

# Energy-Token Program

**Program ID:** `2CVWTnckn5TXUWXdZoZE6LydiQJGMYHVVPipkoy1LVqr`

## Purpose

The Energy-Token program manages the **native energy token** (likely on Solana SPL Token standard) that represents tradeable energy on the platform.

### Key Responsibilities
- Token initialization and configuration
- Token transfers between accounts
- Token burning (energy consumption)
- REC validator management
- Total supply tracking

## Core Functions

### 1. `initialize()`
**Access:** Authority (admin)

Basic program initialization.

### 2. `initialize_token()`
**Access:** Authority (admin)

Initializes the energy token system.

```rust
Initializes TokenInfo:
  - authority: The caller
  - mint: SPL Token mint address
  - total_supply: 0
  - created_at: Current timestamp

Note: Uses Solana SPL Token program for actual token operations
```

### 3. `add_rec_validator(validator_pubkey, authority_name)`
**Access:** Authority only

Adds an authorized REC validator.

```rust
Parameters:
  - validator_pubkey: Pubkey - Validator's account
  - authority_name: String - Validator's name

Purpose:
  - Authorizes validators to mint/validate energy tokens
  - Tracks validator information
```

### 4. `transfer_tokens(amount: u64)`
**Access:** Token holder

Transfers energy tokens between accounts.

```rust
Parameters:
  - amount: u64 - Number of tokens to transfer

Uses SPL Token Program via CPI:
  - Transfers from user's token account
  - Sends to recipient's token account
  - Authority must sign the transfer

Events:
  "Transferred {amount} tokens"
```

### 5. `burn_tokens(amount: u64)`
**Access:** Token holder

Burns tokens (simulates energy consumption).

```rust
Parameters:
  - amount: u64 - Number of tokens to burn

Uses SPL Token Program via CPI:
  - Burns tokens from user's account
  - Decreases total_supply
  - Reduces user's balance

Events:
  "Burned {amount} tokens"
```

## Data Structures

### TokenInfo Account
```rust
{
  authority: Pubkey,    // Program authority
  mint: Pubkey,         // SPL Token mint address
  total_supply: u64,    // Current token supply
  created_at: i64,
}
```

## Error Codes

```rust
UnauthorizedAuthority      // Not the authority
InvalidMeter              // Invalid meter reference
InsufficientBalance       // Not enough tokens
```

---

# Trading Program

**Program ID:** `dS3zvp95PFVrNNBfZDXn78QL5MvhUqDCFR4rn8z9Jgh`

## Purpose

The Trading program implements the **P2P energy trading marketplace** where users can buy and sell energy.

### Key Responsibilities
- Market initialization and management
- Order creation (buy/sell orders)
- Order matching and execution
- Order cancellation
- Market parameter management
- Fee collection

## Core Functions

### 1. `initialize()`
**Access:** Authority (admin)

Basic program initialization.

### 2. `initialize_market()`
**Access:** Authority (admin)

Initializes the trading market.

```rust
Initializes Market:
  - authority: The caller
  - active_orders: 0
  - total_volume: 0
  - total_trades: 0
  - clearing_enabled: true
  - market_fee_bps: 25 (0.25% fee)
  - created_at: Current timestamp

Events:
  MarketInitialized {
    authority,
    timestamp,
  }
```

### 3. `create_sell_order(energy_amount: u64, price_per_kwh: u64)`
**Access:** Seller with energy to sell

Creates a sell order for energy.

```rust
Parameters:
  - energy_amount: u64 - kWh to sell
  - price_per_kwh: u64 - Price in tokens per kWh

Expected Behavior:
  - Creates Order account
  - Sets order_type: Sell
  - Sets status: Active
  - Records seller's pubkey
  - Escrows energy tokens

Events:
  SellOrderCreated {
    seller,
    order_id,
    amount,
    price_per_kwh,
    timestamp,
  }
```

### 4. `create_buy_order(energy_amount: u64, max_price_per_kwh: u64)`
**Access:** Buyer with tokens to spend

Creates a buy order for energy.

```rust
Parameters:
  - energy_amount: u64 - kWh desired
  - max_price_per_kwh: u64 - Maximum price per kWh

Expected Behavior:
  - Creates Order account
  - Sets order_type: Buy
  - Sets status: Active
  - Records buyer's pubkey
  - Escrows payment tokens

Events:
  BuyOrderCreated {
    buyer,
    order_id,
    amount,
    price_per_kwh,
    timestamp,
  }
```

### 5. `match_orders()`
**Access:** Market operator

Matches a buy order with a sell order.

```rust
Preconditions:
  - Buy order active
  - Sell order active
  - Price compatibility (buy_price >= sell_price)
  - Sufficient escrow balance both sides

Effects:
  - Transfers energy tokens to buyer
  - Transfers payment tokens to seller
  - Deducts market fee (0.25%)
  - Updates trade record
  - Updates order status

Events:
  OrderMatched {
    sell_order,
    buy_order,
    seller,
    buyer,
    amount,
    price,
    total_value,
    fee_amount,
    timestamp,
  }
```

### 6. `cancel_order(order_id: u64)`
**Access:** Order owner

Cancels an active order.

```rust
Preconditions:
  - Order must be Active or PartiallyFilled
  - Caller must be order owner

Effects:
  - Returns escrowed tokens to original owner
  - Sets order status to Cancelled
  - Frees up order slot

Events:
  OrderCancelled {
    order_id,
    user,
    timestamp,
  }
```

### 7. `update_market_params(market_fee_bps: u16, clearing_enabled: bool)`
**Access:** Market authority only

Updates market configuration.

```rust
Parameters:
  - market_fee_bps: u16 - Fee in basis points (25 = 0.25%)
  - clearing_enabled: bool - Enable/disable market clearing

Validation:
  ✓ Caller must be market authority

Events:
  MarketParamsUpdated {
    authority,
    market_fee_bps,
    clearing_enabled,
    timestamp,
  }
```

## Data Structures

### Market Account
```rust
{
  authority: Pubkey,          // Market operator
  active_orders: u64,         // Number of open orders
  total_volume: u64,          // Total kWh traded
  total_trades: u64,          // Number of completed trades
  created_at: i64,
  clearing_enabled: bool,     // Market clearing active
  market_fee_bps: u16,        // Fee in basis points (25 = 0.25%)
}
```

### Order Account
```rust
{
  seller: Pubkey,             // Seller's address
  buyer: Pubkey,              // Buyer's address (if matched)
  amount: u64,                // Order size in kWh
  filled_amount: u64,         // Amount already matched
  price_per_kwh: u64,         // Price in tokens/kWh
  order_type: OrderType,      // Sell or Buy
  status: OrderStatus,        // Active, PartiallyFilled, etc.
  created_at: i64,
  expires_at: i64,
}

enum OrderType {
  Sell,  // Offering energy
  Buy,   // Requesting energy
}

enum OrderStatus {
  Active,
  PartiallyFilled,
  Completed,
  Cancelled,
  Expired,
}
```

### TradeRecord Account
```rust
{
  sell_order: Pubkey,         // Sell order that filled
  buy_order: Pubkey,          // Buy order that filled
  seller: Pubkey,
  buyer: Pubkey,
  amount: u64,                // kWh traded
  price_per_kwh: u64,         // Execution price
  total_value: u64,           // amount * price
  fee_amount: u64,            // Platform fee
  executed_at: i64,
}
```

## Events

| Event | Triggered By | Contains |
|-------|--------------|----------|
| `MarketInitialized` | initialize_market | authority, timestamp |
| `SellOrderCreated` | create_sell_order | seller, order_id, amount, price, timestamp |
| `BuyOrderCreated` | create_buy_order | buyer, order_id, amount, price, timestamp |
| `OrderMatched` | match_orders | sell_order, buy_order, seller, buyer, amount, price, total_value, fee, timestamp |
| `OrderCancelled` | cancel_order | order_id, user, timestamp |
| `MarketParamsUpdated` | update_market_params | authority, fee_bps, clearing_enabled, timestamp |

## Error Codes

```rust
UnauthorizedAuthority          // Not market authority
InvalidAmount                  // Invalid order amount
InvalidPrice                   // Invalid price
InactiveSellOrder             // Sell order not active
InactiveBuyOrder              // Buy order not active
PriceMismatch                 // Buy price < sell price
OrderNotCancellable           // Order cannot be cancelled
InsufficientEscrowBalance     // Not enough tokens escrowed
```

---

# Program Interaction Flow

## Complete User Journey

```
1. User Registration
   └─> Registry.register_user(Prosumer, "Location")
       └─> Creates UserAccount
       └─> User is now in system

2. Meter Registration
   └─> Registry.register_meter("METER-001", Solar)
       └─> Creates MeterAccount
       └─> Meter type: Solar (production)

3. Energy Generation
   └─> Physical meter generates energy
   └─> AMI system measures: 100 kWh produced

4. Oracle Submission
   └─> Oracle.submit_meter_reading("METER-001", 100, 0, timestamp)
       └─> API Gateway sends data
       └─> Registry automatically updated

5. ERC Issuance
   └─> Governance.issue_erc("CERT-001", 100, "Solar", "valid_data")
       └─> Engineering Department creates certificate
       └─> Status: Valid
       └─> Expires in 1 year

6. ERC Validation
   └─> Governance.validate_erc_for_trading()
       └─> Certificate now tradable
       └─> Status: ValidatedForTrading

7. Energy Tokens Received
   └─> Energy-Token.transfer_tokens(100) from Oracle to User
       └─> User receives 100 energy tokens

8. Order Creation (Seller)
   └─> Trading.create_sell_order(50, 10)
       └─> Offering 50 kWh at 10 tokens/kWh
       └─> Seller's tokens escrowed

9. Order Creation (Buyer)
   └─> Trading.create_buy_order(50, 10)
       └─> Requesting 50 kWh at max 10 tokens/kWh
       └─> Buyer's tokens escrowed

10. Order Matching
    └─> Trading.match_orders()
        └─> Prices match: 10 == 10
        └─> Energy tokens → Buyer
        └─> Payment tokens → Seller
        └─> 0.25% fee → Market
        └─> Trade record created

11. Market Clearing
    └─> Oracle.trigger_market_clearing()
        └─> Settles all trades
        └─> Updates balances
```

## Cross-Program Calls (CPC)

The programs communicate using Cross-Program Invocations:

```
Energy-Token Program
    └─ Uses SPL Token Program
       └─ Transfers tokens via CPI

Trading Program
    └─ Calls Energy-Token
       └─ Updates market state

Registry Program
    └─ Called by Oracle
       └─ Updates meter readings

Governance Program
    └─ Validates ERCs
    └─ Controls system parameters
```

---

# Security Considerations

## Access Control

### Multi-Level Authorization

1. **Program Authority (Admin)**
   - Initialize and configure programs
   - Emergency controls
   - Parameter updates
   - Examples: Oracle authority, Governance authority, Registry authority

2. **API Gateway**
   - Submit meter readings
   - Trigger market clearing
   - Only Oracle program allows this
   - Examples: AMI systems, smart meters

3. **Individual Users**
   - Register themselves
   - Create trading orders
   - Own their accounts
   - Examples: Prosumers, consumers

## Key Security Patterns

### 1. Program-Derived Accounts (PDAs)
- Deterministic account addresses
- Cannot be replayed by unauthorized parties
- Seeds include user pubkey to prevent cross-user access
- Example: `Registry User PDA = hash(b"user", user_pubkey)`

### 2. Signer Requirement
- Critical operations require cryptographic signatures
- Cannot be forged without private key
- Examples: User registration, order creation

### 3. Validation Checks
```rust
// Always verify critical invariants
require!(oracle_data.active, ErrorCode::OracleInactive);
require!(
    ctx.accounts.authority.key() == oracle_data.api_gateway,
    ErrorCode::UnauthorizedGateway
);
```

### 4. Immutable Historical Records
- All changes emit events
- Cannot delete records (immutable ledger)
- Full audit trail available

### 5. Rate Limiting Consideration
- Market clearing limits order matching frequency
- Emergency pause prevents rapid state changes
- Maintenance mode for controlled updates

## Attack Prevention

| Attack | Prevention |
|--------|-----------|
| Unauthorized API calls | API Gateway whitelist in Oracle |
| Double spending | SPL Token program's state machine |
| Unauthorized trading | User signature requirement |
| Market manipulation | Price validation, order size limits |
| Replay attacks | Unique timestamps in each transaction |
| Account takeover | PDA seeds prevent account confusion |

## Audit Points

**For each transaction verify:**
1. ✅ All signers are authorized
2. ✅ Account ownership is correct
3. ✅ State transitions are valid
4. ✅ No mathematical overflows
5. ✅ Events are properly emitted

---

## Summary Table

| Program | Purpose | Authority | Key Accounts |
|---------|---------|-----------|--------------|
| **Oracle** | Data input | API Gateway | OracleData |
| **Governance** | Certificates | Engineering Dept | PoAConfig, ErcCertificate |
| **Registry** | Users & Meters | University | Registry, UserAccount, MeterAccount |
| **Energy-Token** | Token mgmt | Authority | TokenInfo, SPL Token accounts |
| **Trading** | Marketplace | Market authority | Market, Order, TradeRecord |

---

**Document Version:** 1.0  
**Anchor Framework:** 0.32.1  
**Date:** November 1, 2025

---

