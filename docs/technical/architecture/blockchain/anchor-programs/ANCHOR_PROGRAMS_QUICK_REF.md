# ⚡ Anchor Programs - Quick Reference Guide

**GridTokenX Platform - Developer Cheat Sheet**

---

## Program IDs & Locations

```
Oracle Program
  ID: ApwexmUbEZMpez5dJXKza4V7gqSqWvAA9BPbok2psxXg
  Path: programs/oracle/src/lib.rs
  Role: Data input layer (meter readings, market clearing)

Governance Program
  ID: Dy8JFn95L1E7NoUkXbFQtW1kGR7Ja21CkNcirNgv4ghe
  Path: programs/governance/src/lib.rs
  Role: PoA governance (ERC issuance & validation)

Registry Program
  ID: 42LoRKPphBBdvaCDx2ZjNuZFqzXuJziiiNXyiV6FhBY5
  Path: programs/registry/src/lib.rs
  Role: User & meter registration

Energy-Token Program
  ID: 2CVWTnckn5TXUWXdZoZE6LydiQJGMYHVVPipkoy1LVqr
  Path: programs/energy-token/src/lib.rs
  Role: Token transfers & burning

Trading Program
  ID: dS3zvp95PFVrNNBfZDXn78QL5MvhUqDCFR4rn8z9Jgh
  Path: programs/trading/src/lib.rs
  Role: P2P marketplace (buy/sell orders)
```

---

## Function Reference by Program

### 🔮 ORACLE PROGRAM

| Function | Access | Parameters | Returns | Purpose |
|----------|--------|-----------|---------|---------|
| `initialize` | Authority | api_gateway: Pubkey | () | Initialize oracle |
| `submit_meter_reading` | API Gateway | meter_id, energy_produced, energy_consumed, timestamp | () | Submit AMI reading |
| `trigger_market_clearing` | API Gateway | - | () | Trigger clearing |
| `update_oracle_status` | Authority | active: bool | () | Enable/disable oracle |
| `update_api_gateway` | Authority | new_api_gateway: Pubkey | () | Change API Gateway |

### 🏛️ GOVERNANCE PROGRAM

| Function | Access | Parameters | Returns | Purpose |
|----------|--------|-----------|---------|---------|
| `initialize_poa` | Authority | - | () | Initialize PoA |
| `issue_erc` | Authority | cert_id, energy_amt, source, data | () | Issue certificate |
| `validate_erc_for_trading` | Authority | - | () | Approve trading |
| `emergency_pause` | Authority | - | () | Pause system |
| `emergency_unpause` | Authority | - | () | Resume system |
| `update_governance_config` | Authority | erc_validation_enabled: bool | () | Update config |
| `set_maintenance_mode` | Authority | maintenance_enabled: bool | () | Maintenance mode |
| `update_erc_limits` | Authority | min, max, validity_period | () | Update limits |
| `update_authority_info` | Authority | contact_info: String | () | Update contact |
| `get_governance_stats` | Public | - | GovernanceStats | Get statistics |

### 📋 REGISTRY PROGRAM

| Function | Access | Parameters | Returns | Purpose |
|----------|--------|-----------|---------|---------|
| `initialize` | Authority | - | () | Initialize registry |
| `register_user` | Public | user_type, location | () | Register user |
| `register_meter` | User | meter_id, meter_type | () | Register meter |
| `update_meter_reading` | Oracle | energy_gen, energy_cons, timestamp | () | Update readings |
| `update_user_status` | Authority | new_status: UserStatus | () | Change status |
| `is_valid_user` | Public | - | bool | Check user status |
| `is_valid_meter` | Public | - | bool | Check meter status |

### ⚡ ENERGY-TOKEN PROGRAM

| Function | Access | Parameters | Returns | Purpose |
|----------|--------|-----------|---------|---------|
| `initialize` | Authority | - | () | Initialize program |
| `initialize_token` | Authority | - | () | Initialize token |
| `add_rec_validator` | Authority | validator_pubkey, name | () | Add validator |
| `transfer_tokens` | Holder | amount: u64 | () | Transfer tokens |
| `burn_tokens` | Holder | amount: u64 | () | Burn tokens |

### 🛒 TRADING PROGRAM

| Function | Access | Parameters | Returns | Purpose |
|----------|--------|-----------|---------|---------|
| `initialize` | Authority | - | () | Initialize program |
| `initialize_market` | Authority | - | () | Initialize market |
| `create_sell_order` | User | energy_amt, price_per_kwh | () | Create sell order |
| `create_buy_order` | User | energy_amt, max_price | () | Create buy order |
| `match_orders` | Operator | - | () | Match buy/sell |
| `cancel_order` | Owner | order_id: u64 | () | Cancel order |
| `update_market_params` | Authority | fee_bps, clearing_enabled | () | Update params |

---

## Data Types Quick Reference

### Enums

**UserType**
```rust
Prosumer,   // Can produce & consume
Consumer,   // Can only consume
```

**UserStatus**
```rust
Active,     // Normal operation
Suspended,  // Blocked from trading
Inactive,   // Disabled
```

**MeterType**
```rust
Solar,      // Solar generation
Wind,       // Wind generation
Battery,    // Battery storage
Grid,       // Grid connection
```

**MeterStatus**
```rust
Active,         // Operational
Inactive,       // Not active
Maintenance,    // Under maintenance
```

**ErcStatus**
```rust
Valid,      // Active & tradable
Expired,    // Past expiration
Revoked,    // Cancelled by authority
Pending,    // Awaiting validation
```

**OrderType**
```rust
Sell,   // Offering energy
Buy,    // Requesting energy
```

**OrderStatus**
```rust
Active,             // Open for matching
PartiallyFilled,    // Partially matched
Completed,          // Fully filled
Cancelled,          // User cancelled
Expired,            // Time expired
```

---

## Common Error Codes

```rust
// Authorization Errors
UnauthorizedAuthority       // Not the admin/owner
UnauthorizedUser           // User not authorized
UnauthorizedGateway        // Not the API Gateway

// Status Errors
OracleInactive             // Oracle is disabled
SystemPaused               // Governance paused
MaintenanceMode            // System under maintenance
ErcValidationDisabled      // ERC validation off

// Validation Errors
InvalidAmount              // Invalid amount
InvalidPrice               // Invalid price
InvalidUserStatus          // Bad status value
InvalidMeterStatus         // Bad meter status
InvalidErcStatus           // Bad ERC status

// State Errors
AlreadyPaused              // Already paused
NotPaused                  // Not paused
AlreadyValidated           // Already validated
ErcExpired                 // Certificate expired

// Constraint Errors
BelowMinimumEnergy         // Amount too small
ExceedsMaximumEnergy       // Amount too large
InsufficientBalance        // Not enough tokens
InsufficientEscrowBalance  // Not enough escrowed
PriceMismatch              // Prices don't match
```

---

## PDA (Program Derived Account) Seeds

These are deterministic addresses derived from seeds:

```rust
// Registry
UserAccount PDA
  Seed: [b"user", user_pubkey.as_ref()]
  
MeterAccount PDA
  Seed: [b"meter", meter_id.as_bytes()]

// Governance
PoAConfig PDA
  Seed: [b"poa_config"]
  
ErcCertificate PDA
  Seed: [b"erc_certificate", certificate_id.as_bytes()]

// Oracle
OracleData PDA
  Seed: [b"oracle_data"]

// Energy-Token
TokenInfo PDA
  Seed: [b"token_info"]

// Trading
Market PDA
  Seed: [b"market"]
```

---

## Event Types by Program

### Oracle Events
```
MeterReadingSubmitted
MarketClearingTriggered
OracleStatusUpdated
ApiGatewayUpdated
```

### Governance Events
```
PoAInitialized
ErcIssued
ErcValidatedForTrading
EmergencyPauseActivated
EmergencyPauseDeactivated
GovernanceConfigUpdated
MaintenanceModeUpdated
ErcLimitsUpdated
AuthorityInfoUpdated
```

### Registry Events
```
RegistryInitialized
UserRegistered
MeterRegistered
UserStatusUpdated
MeterReadingUpdated
```

### Trading Events
```
MarketInitialized
SellOrderCreated
BuyOrderCreated
OrderMatched
OrderCancelled
MarketParamsUpdated
```

---

## Default Configuration Values

### Governance (PoAConfig)
```
Max ERC Amount:         1,000,000 kWh
Min Energy Amount:      100 kWh
ERC Validity Period:    31,536,000 seconds (1 year)
```

### Trading (Market)
```
Market Fee:             25 basis points (0.25%)
Clearing Enabled:       true
```

### Registry
```
User Count:             0 (at init)
Meter Count:            0 (at init)
```

---

## Call Chain Examples

### Example 1: User Registration → Meter Registration

```
1. User calls Registry.register_user(Prosumer, "Bangkok")
   └─ Creates UserAccount PDA
   └─ Increments user_count
   
2. User calls Registry.register_meter("METER-001", Solar)
   └─ Creates MeterAccount PDA
   └─ Increments meter_count & user.meter_count
```

### Example 2: Energy Generation → Trading

```
1. Meter generates 50 kWh
2. Oracle calls Oracle.submit_meter_reading(...)
   └─ Updates Registry meter reading
   
3. Governance issues ERC: Governance.issue_erc(...)
   └─ Creates ErcCertificate
   
4. Governance validates: Governance.validate_erc_for_trading()
   └─ ERC now tradable
   
5. Seller creates order: Trading.create_sell_order(50, 10)
   └─ 50 kWh at 10 tokens/kWh
   
6. Buyer creates order: Trading.create_buy_order(50, 10)
   └─ Wants 50 kWh, will pay up to 10/kWh
   
7. Match orders: Trading.match_orders()
   └─ Trade executed
   └─ Tokens transferred
   └─ Fee collected (0.25%)
```

### Example 3: Emergency Response

```
1. Security issue detected
2. Authority calls Governance.emergency_pause()
   └─ Blocks all ERC issuance
   └─ Blocks all trading
   
3. Issue resolved
4. Authority calls Governance.emergency_unpause()
   └─ System resumes
```

---

## Testing Patterns

### To Test Oracle
```
1. Initialize oracle with API Gateway
2. Call submit_meter_reading with valid data
3. Verify event emitted
4. Trigger market clearing
5. Verify timestamp updated
```

### To Test Governance
```
1. Initialize PoA
2. Issue an ERC
3. Validate ERC for trading
4. Verify status changed
5. Try to validate again (should fail)
```

### To Test Registry
```
1. Initialize registry
2. Register user as Prosumer
3. Register meter of type Solar
4. Update meter reading
5. Check user/meter are valid
```

### To Test Trading
```
1. Initialize market
2. Create sell order
3. Create buy order
4. Match orders
5. Verify trade record created
```

---

## Common Checks Before Calling

### Before Oracle.submit_meter_reading
- [ ] Oracle is initialized
- [ ] Oracle is active
- [ ] Caller is API Gateway
- [ ] Meter readings are valid
- [ ] Timestamp is reasonable

### Before Governance.issue_erc
- [ ] System not paused
- [ ] Not in maintenance mode
- [ ] Energy amount >= min (100 kWh)
- [ ] Energy amount <= max (1M kWh)
- [ ] Certificate ID <= 64 chars

### Before Trading.create_sell_order
- [ ] User has energy tokens
- [ ] Amount > 0
- [ ] Price > 0
- [ ] User has sufficient balance

### Before Trading.match_orders
- [ ] Buy order is active
- [ ] Sell order is active
- [ ] Buy price >= Sell price
- [ ] Both orders have sufficient escrow

---

## Token Values & Units

```
Energy Unit: kWh (kilowatt-hours)
Token Unit: 1 energy token = 1 kWh (typically)
Trading Fee: 25 basis points = 0.25% = 0.0025x
Timestamps: Unix timestamp (seconds)
```

### Fee Calculation Example
```
Trade: 100 kWh at 10 tokens/kWh
Total Value: 100 * 10 = 1,000 tokens
Fee (0.25%): 1,000 * 0.0025 = 2.5 tokens
Seller receives: 1,000 - 2.5 = 997.5 tokens
```

---

## Version Information

```
Anchor Framework:  0.32.1 (Latest)
Solana Program:    v2.3.0
Borsh:            v1.5.7
SPL Token:        v9.0.0
```

---

**Quick Reference Guide v1.0**  
**Generated:** November 1, 2025  
**For detailed info see:** ANCHOR_PROGRAMS_DETAILED_GUIDE.md
