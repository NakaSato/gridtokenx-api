# 🔄 Data Flow Diagrams

**GridTokenX Platform - Function-Level Data Flow Analysis**

> **📘 For complete sequence diagrams with all interactions, see [Anchor Architecture Diagrams](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml)**
> 
> **Note**: This document provides ASCII-based data flows. For professional UML sequence diagrams, refer to the Anchor documentation.

---

## Table of Contents

1. [Oracle Program Functions](#oracle-program-functions)
2. [Governance Program Functions](#governance-program-functions)
3. [Registry Program Functions](#registry-program-functions)
4. [Energy-Token Program Functions](#energy-token-program-functions)
5. [Trading Program Functions](#trading-program-functions)
6. [Complete System Flows](#complete-system-flows)

---

## Oracle Program Functions

### Oracle.initialize()

```
Authority                    Blockchain
──────────                   ──────────

Authority Signer ──────────► Oracle.initialize(api_gateway)
    │                               │
    │                               │ [Validation: Authority signer]
    │                               │
    │                               ▼
    │                        Create OracleData Account
    │                               │
    │                               ├─ oracle_data.authority = authority.key()
    │                               ├─ oracle_data.api_gateway = api_gateway
    │                               ├─ oracle_data.total_readings = 0
    │                               ├─ oracle_data.last_clearing = 0
    │                               ├─ oracle_data.active = true
    │                               ├─ oracle_data.created_at = NOW
    │                               │
    │                               ▼
    │                        PDA Created [seed: "oracle_data"]
    │                               │
    │                               ▼
    │                        msg!("Oracle program initialized")
    │                               │
    │◄──────────────────────────────┘
    │
    ▼
Oracle Ready ✓
```

### Oracle.submit_meter_reading()

```
API Gateway                  Blockchain                     External Systems
───────────                  ──────────                     ─────────────

AMI Meter ──readings──► API Gateway ──────────► Oracle.submit_meter_reading()
    │                       │                           │
    │                       │                           │ [Validation: oracle.active]
    │                       │                           │ [Validation: caller == api_gateway]
    │                       │                           │
    │                       │                           ▼
    │                       │                    Update OracleData
    │                       │                           │
    │                       │                           ├─ total_readings += 1
    │                       │                           ├─ last_reading_timestamp = reading_timestamp
    │                       │                           │
    │                       │                           ▼
    │                       │                    Emit MeterReadingSubmitted Event
    │                       │                           │
    │                       │                           ├─ meter_id
    │                       │                           ├─ energy_produced
    │                       │                           ├─ energy_consumed
    │                       │                           ├─ timestamp
    │                       │                           ├─ submitter
    │                       │                           │
    │                       │◄──────────────────────────┘
    │                       │
    │◄──────success─────────┘
    │
    ▼
Reading Stored ✓
```

### Oracle.trigger_market_clearing()

```
API Gateway                  Blockchain                     Trading System
───────────                  ──────────                     ──────────────

Market Timer ──signal──► API Gateway ──────────► Oracle.trigger_market_clearing()
    │                       │                           │
    │                       │                           │ [Validation: oracle.active]
    │                       │                           │ [Validation: caller == api_gateway]
    │                       │                           │
    │                       │                           ▼
    │                       │                    Update OracleData
    │                       │                           │
    │                       │                           ├─ last_clearing = current_time
    │                       │                           │
    │                       │                           ▼
    │                       │                    Emit MarketClearingTriggered Event
    │                       │                           │
    │                       │                           ├─ authority
    │                       │                           ├─ timestamp
    │                       │                           │
    │                       │◄──────────────────────────┘
    │                       │
    │◄──────success─────────┘
    │
    ▼
Market Clearing Triggered ✓ ──────────────────────────────► Trading.match_orders()
```

### Oracle.update_oracle_status()

```
Authority                    Blockchain
──────────                   ──────────

Authority ──────────► Oracle.update_oracle_status(active: bool)
    │                               │
    │                               │ [Validation: caller == oracle.authority]
    │                               │
    │                               ▼
    │                        Update OracleData
    │                               │
    │                               ├─ oracle_data.active = active
    │                               │
    │                               ▼
    │                        Emit OracleStatusUpdated Event
    │                               │
    │                               ├─ authority
    │                               ├─ active
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┘
    │
    ▼
Status Updated ✓
```

### Oracle.update_api_gateway()

```
Authority                    Blockchain
──────────                   ──────────

Authority ──────────► Oracle.update_api_gateway(new_api_gateway)
    │                               │
    │                               │ [Validation: caller == oracle.authority]
    │                               │
    │                               ▼
    │                        Update OracleData
    │                               │
    │                               ├─ old_gateway = oracle_data.api_gateway
    │                               ├─ oracle_data.api_gateway = new_api_gateway
    │                               │
    │                               ▼
    │                        Emit ApiGatewayUpdated Event
    │                               │
    │                               ├─ authority
    │                               ├─ old_gateway
    │                               ├─ new_gateway
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┘
    │
    ▼
Gateway Updated ✓
```

---

## Governance Program Functions

### Governance.initialize_poa()

```
Authority                    Blockchain
──────────                   ──────────

Engineering Dept ──────────► Governance.initialize_poa()
    │                               │
    │                               │ [Authority: Engineering Department]
    │                               │
    │                               ▼
    │                        Create PoAConfig Account
    │                               │
    │                               ├─ authority = authority.key()
    │                               ├─ authority_name = "University Engineering Department"
    │                               ├─ contact_info = "engineering_erc@utcc.ac.th"
    │                               ├─ emergency_paused = false
    │                               ├─ erc_validation_enabled = true
    │                               ├─ max_erc_amount = 1_000_000
    │                               ├─ min_energy_amount = 100
    │                               ├─ erc_validity_period = 31_536_000 (1 year)
    │                               ├─ total_ercs_issued = 0
    │                               ├─ total_ercs_validated = 0
    │                               ├─ maintenance_mode = false
    │                               ├─ created_at = NOW
    │                               │
    │                               ▼
    │                        PDA Created [seed: "poa_config"]
    │                               │
    │                               ▼
    │                        Emit PoAInitialized Event
    │                               │
    │                               ├─ authority
    │                               ├─ authority_name
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┘
    │
    ▼
PoA Governance Ready ✓
```

### Governance.emergency_pause()

```
Authority                    Blockchain                     System Impact
──────────                   ──────────                     ─────────────

Emergency Detected
    │
    ▼
Engineering Dept ──────────► Governance.emergency_pause()
    │                               │
    │                               │ [Validation: caller == poa_config.authority]
    │                               │ [Validation: !already_paused]
    │                               │
    │                               ▼
    │                        Update PoAConfig
    │                               │
    │                               ├─ emergency_paused = true
    │                               ├─ emergency_timestamp = NOW
    │                               │
    │                               ▼
    │                        Emit EmergencyPauseActivated Event
    │                               │
    │                               ├─ authority
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┼──────────────────────────► All Programs Frozen
    │                               │                             │
    │                               │                             ├─ No new ERC issuance
    │                               │                             ├─ No ERC validation
    │                               │                             ├─ Block governance updates
    │                               │                             │
    ▼                               │                             ▼
Emergency Pause Active ✓            │                      System Protection Active ✓
```

### Governance.emergency_unpause()

```
Authority                    Blockchain                     System Impact
──────────                   ──────────                     ─────────────

Issue Resolved
    │
    ▼
Engineering Dept ──────────► Governance.emergency_unpause()
    │                               │
    │                               │ [Validation: caller == poa_config.authority]
    │                               │ [Validation: currently_paused]
    │                               │
    │                               ▼
    │                        Update PoAConfig
    │                               │
    │                               ├─ emergency_paused = false
    │                               ├─ emergency_timestamp = None
    │                               │
    │                               ▼
    │                        Emit EmergencyPauseDeactivated Event
    │                               │
    │                               ├─ authority
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┼──────────────────────────► All Programs Resume
    │                               │                             │
    │                               │                             ├─ ERC issuance enabled
    │                               │                             ├─ ERC validation enabled
    │                               │                             ├─ Normal operations
    │                               │                             │
    ▼                               │                             ▼
Emergency Resolved ✓                │                      System Normal ✓
```

### Governance.issue_erc()

```
Authority                    Blockchain                     Certificate Creation
──────────                   ──────────                     ────────────────────

Energy Generated ──────────► Engineering Dept ──────────► Governance.issue_erc()
    │                               │                           │
    │                               │                           │ [Validation: !emergency_paused]
    │                               │                           │ [Validation: !maintenance_mode]
    │                               │                           │ [Validation: erc_validation_enabled]
    │                               │                           │ [Validation: energy >= min_amount]
    │                               │                           │ [Validation: energy <= max_amount]
    │                               │                           │ [Validation: certificate_id.len() <= 64]
    │                               │                           │
    │                               │                           ▼
    │                               │                    Create ErcCertificate Account
    │                               │                           │
    │                               │                           ├─ certificate_id
    │                               │                           ├─ authority = authority.key()
    │                               │                           ├─ energy_amount
    │                               │                           ├─ renewable_source
    │                               │                           ├─ validation_data
    │                               │                           ├─ status = Valid
    │                               │                           ├─ validated_for_trading = false
    │                               │                           ├─ created_at = NOW
    │                               │                           ├─ expires_at = NOW + validity_period
    │                               │                           │
    │                               │                           ▼
    │                               │                    Update PoAConfig
    │                               │                           │
    │                               │                           ├─ total_ercs_issued += 1
    │                               │                           │
    │                               │                           ▼
    │                               │                    PDA Created [seed: "erc_certificate", certificate_id]
    │                               │                           │
    │                               │                           ▼
    │                               │                    Emit ErcIssued Event
    │                               │                           │
    │                               │                           ├─ certificate_id
    │                               │                           ├─ authority
    │                               │                           ├─ energy_amount
    │                               │                           ├─ renewable_source
    │                               │                           ├─ timestamp
    │                               │                           │
    │                               │◄──────────────────────────┘
    │                               │
    │◄──────────────────────────────┘
    │
    ▼
ERC Certificate Issued ✓
```

### Governance.validate_erc_for_trading()

```
Authority                    Blockchain                     Trading Enable
──────────                   ──────────                     ──────────────

ERC Verification ──────────► Engineering Dept ──────────► Governance.validate_erc_for_trading()
    │                               │                           │
    │                               │                           │ [Validation: !emergency_paused]
    │                               │                           │ [Validation: !maintenance_mode]
    │                               │                           │ [Validation: erc_validation_enabled]
    │                               │                           │ [Validation: certificate exists]
    │                               │                           │ [Validation: status == Valid]
    │                               │                           │ [Validation: !already_validated]
    │                               │                           │ [Validation: !expired]
    │                               │                           │
    │                               │                           ▼
    │                               │                    Update ErcCertificate
    │                               │                           │
    │                               │                           ├─ validated_for_trading = true
    │                               │                           ├─ trading_validated_at = NOW
    │                               │                           ├─ trading_validator = authority.key()
    │                               │                           │
    │                               │                           ▼
    │                               │                    Update PoAConfig
    │                               │                           │
    │                               │                           ├─ total_ercs_validated += 1
    │                               │                           │
    │                               │                           ▼
    │                               │                    Emit ErcValidatedForTrading Event
    │                               │                           │
    │                               │                           ├─ certificate_id
    │                               │                           ├─ authority
    │                               │                           ├─ energy_amount
    │                               │                           ├─ timestamp
    │                               │                           │
    │                               │◄──────────────────────────┼──────────────────────────► Energy-Token.transfer_tokens()
    │                               │                           │                             │
    │                               │                           │                             ├─ Issue tokens to user
    │                               │                           │                             ├─ Enable trading
    │                               │                           │                             │
    │◄──────────────────────────────┘                           │                             ▼
    │                                                           │                      Trading Enabled ✓
    ▼
ERC Trading Validated ✓
```

---

## Registry Program Functions

### Registry.initialize()

```
Authority                    Blockchain
──────────                   ──────────

System Admin ──────────► Registry.initialize()
    │                               │
    │                               │ [Authority: System administrator]
    │                               │
    │                               ▼
    │                        Create Registry Account
    │                               │
    │                               ├─ authority = authority.key()
    │                               ├─ user_count = 0
    │                               ├─ meter_count = 0
    │                               ├─ created_at = NOW
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        PDA Created [seed: "registry"]
    │                               │
    │                               ▼
    │                        Emit RegistryInitialized Event
    │                               │
    │                               ├─ authority
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┘
    │
    ▼
Registry Ready ✓
```

### Registry.register_user()

```
User                         Blockchain                     User Creation
────                         ──────────                     ─────────────

New User ──────────► Registry.register_user(user_type, location)
    │                               │
    │                               │ [Validation: valid user_type]
    │                               │ [Validation: valid location string]
    │                               │
    │                               ▼
    │                        Create UserAccount
    │                               │
    │                               ├─ owner = user.key()
    │                               ├─ user_type = user_type (Prosumer/Consumer)
    │                               ├─ location = location
    │                               ├─ status = Active
    │                               ├─ meter_count = 0
    │                               ├─ total_energy_produced = 0
    │                               ├─ total_energy_consumed = 0
    │                               ├─ created_at = NOW
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        Update Registry
    │                               │
    │                               ├─ user_count += 1
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        PDA Created [seed: "user", user.key()]
    │                               │
    │                               ▼
    │                        Emit UserRegistered Event
    │                               │
    │                               ├─ user_pubkey
    │                               ├─ user_type
    │                               ├─ location
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┘
    │
    ▼
User Registered ✓ ──────────────────────────────────────► Can register meters
                                                         ► Can create trading orders
```

### Registry.register_meter()

```
User                         Blockchain                     Meter Creation
────                         ──────────                     ──────────────

User with Account ──────────► Registry.register_meter(meter_id, meter_type)
    │                               │
    │                               │ [Validation: user account exists]
    │                               │ [Validation: user is active]
    │                               │ [Validation: valid meter_id]
    │                               │ [Validation: valid meter_type]
    │                               │
    │                               ▼
    │                        Create MeterAccount
    │                               │
    │                               ├─ meter_id = meter_id
    │                               ├─ owner = user.key()
    │                               ├─ meter_type = meter_type
    │                               ├─ status = Active
    │                               ├─ total_energy_produced = 0
    │                               ├─ total_energy_consumed = 0
    │                               ├─ last_reading_timestamp = 0
    │                               ├─ created_at = NOW
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        Update UserAccount
    │                               │
    │                               ├─ meter_count += 1
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        Update Registry
    │                               │
    │                               ├─ meter_count += 1
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        PDA Created [seed: "meter", meter_id]
    │                               │
    │                               ▼
    │                        Emit MeterRegistered Event
    │                               │
    │                               ├─ meter_id
    │                               ├─ owner
    │                               ├─ meter_type
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┘
    │
    ▼
Meter Registered ✓ ──────────────────────────────────────► Can receive readings
                                                          ► Can generate ERCs
```

### Registry.update_meter_reading()

```
Oracle/API Gateway           Blockchain                     Meter Data Update
──────────────────           ──────────                     ─────────────────

AMI Reading ──────────► Registry.update_meter_reading()
    │                               │
    │                               │ [Validation: meter exists]
    │                               │ [Validation: meter is active]
    │                               │ [Validation: valid timestamp]
    │                               │
    │                               ▼
    │                        Update MeterAccount
    │                               │
    │                               ├─ total_energy_produced += energy_produced
    │                               ├─ total_energy_consumed += energy_consumed
    │                               ├─ last_reading_timestamp = timestamp
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        Update UserAccount
    │                               │
    │                               ├─ total_energy_produced += energy_produced
    │                               ├─ total_energy_consumed += energy_consumed
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        Emit MeterReadingUpdated Event
    │                               │
    │                               ├─ meter_id
    │                               ├─ energy_produced
    │                               ├─ energy_consumed
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┼──────────────────────────► Governance.issue_erc()
    │                               │                             │
    │                               │                             ├─ Trigger ERC issuance
    │                               │                             ├─ For energy_produced > 0
    │                               │                             │
    ▼                               │                             ▼
Reading Updated ✓                   │                      ERC Process Started ✓
```

---

## Energy-Token Program Functions

### Energy-Token.initialize()

```
Authority                    Blockchain
──────────                   ──────────

System Admin ──────────► Energy-Token.initialize()
    │                               │
    │                               │ [Authority: System administrator]
    │                               │
    │                               ▼
    │                        Create TokenInfo Account
    │                               │
    │                               ├─ authority = authority.key()
    │                               ├─ total_supply = 0
    │                               ├─ total_burned = 0
    │                               ├─ created_at = NOW
    │                               ├─ mint = None (will be set in initialize_token)
    │                               │
    │                               ▼
    │                        PDA Created [seed: "token_info"]
    │                               │
    │                               ▼
    │                        msg!("Energy token program initialized")
    │                               │
    │◄──────────────────────────────┘
    │
    ▼
Token Program Ready ✓
```

### Energy-Token.initialize_token()

```
Authority                    Blockchain                     SPL Token System
──────────                   ──────────                     ────────────────

Authority ──────────► Energy-Token.initialize_token()
    │                               │
    │                               │ [Validation: caller == token_info.authority]
    │                               │
    │                               ▼
    │                        Create SPL Token Mint
    │                               │
    │                               ├─ decimals = 6
    │                               ├─ mint_authority = token_info PDA
    │                               ├─ freeze_authority = None
    │                               │
    │                               ▼
    │                        Update TokenInfo
    │                               │
    │                               ├─ mint = mint.key()
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        Emit TokenInitialized Event
    │                               │
    │                               ├─ mint
    │                               ├─ authority
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┼──────────────────────────► SPL Token Mint Created
    │                               │                             │
    │                               │                             ├─ GridToken (GTX) ready
    │                               │                             ├─ Can mint tokens
    │                               │                             │
    ▼                               │                             ▼
Token System Ready ✓                │                      SPL Integration ✓
```

### Energy-Token.transfer_tokens()

```
User/System                  Blockchain                     SPL Token Transfer
───────────                  ──────────                     ──────────────────

ERC Validated ──────────► Energy-Token.transfer_tokens(amount, recipient)
    │                               │
    │                               │ [Validation: token_info exists]
    │                               │ [Validation: mint exists]
    │                               │ [Validation: amount > 0]
    │                               │
    │                               ▼
    │                        Get/Create Recipient Token Account
    │                               │
    │                               │ [via SPL Token Program]
    │                               │
    │                               ▼
    │                        Mint Tokens to Recipient
    │                               │
    │                               │ [Mint authority: token_info PDA]
    │                               │ [Amount: equivalent to energy_amount]
    │                               │
    │                               ▼
    │                        Update TokenInfo
    │                               │
    │                               ├─ total_supply += amount
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        Emit TokenTransferred Event
    │                               │
    │                               ├─ recipient
    │                               ├─ amount
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┼──────────────────────────► User Token Account
    │                               │                             │
    │                               │                             ├─ Balance += amount
    │                               │                             ├─ Can trade tokens
    │                               │                             │
    ▼                               │                             ▼
Tokens Received ✓                   │                      Trading Ready ✓
```

### Energy-Token.burn_tokens()

```
User                         Blockchain                     SPL Token Burn
────                         ──────────                     ──────────────

User Consumption ──────────► Energy-Token.burn_tokens(amount)
    │                               │
    │                               │ [Validation: user has sufficient balance]
    │                               │ [Validation: amount > 0]
    │                               │
    │                               ▼
    │                        Burn Tokens from User Account
    │                               │
    │                               │ [via SPL Token Program]
    │                               │ [Reduce user token balance]
    │                               │
    │                               ▼
    │                        Update TokenInfo
    │                               │
    │                               ├─ total_burned += amount
    │                               ├─ total_supply -= amount
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        Emit TokenBurned Event
    │                               │
    │                               ├─ user
    │                               ├─ amount
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┼──────────────────────────► User Token Account
    │                               │                             │
    │                               │                             ├─ Balance -= amount
    │                               │                             ├─ Energy consumed
    │                               │                             │
    ▼                               │                             ▼
Energy Consumed ✓                   │                      Tokens Burned ✓
```

---

## Trading Program Functions

### Trading.initialize()

```
Authority                    Blockchain
──────────                   ──────────

System Admin ──────────► Trading.initialize()
    │                               │
    │                               │ [Authority: System administrator]
    │                               │
    │                               ▼
    │                        Setup Trading Program
    │                               │
    │                               ├─ Initialize program state
    │                               ├─ Set authority
    │                               │
    │                               ▼
    │                        msg!("Trading program initialized")
    │                               │
    │◄──────────────────────────────┘
    │
    ▼
Trading Program Ready ✓
```

### Trading.initialize_market()

```
Authority                    Blockchain
──────────                   ──────────

Authority ──────────► Trading.initialize_market()
    │                               │
    │                               │ [Validation: caller == authority]
    │                               │
    │                               ▼
    │                        Create Market Account
    │                               │
    │                               ├─ authority = authority.key()
    │                               ├─ total_orders = 0
    │                               ├─ total_trades = 0
    │                               ├─ total_volume = 0
    │                               ├─ trading_fee = 25 (0.25%)
    │                               ├─ min_order_size = 100 (100 Wh)
    │                               ├─ max_order_size = 1_000_000 (1 MWh)
    │                               ├─ clearing_enabled = true
    │                               ├─ created_at = NOW
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        PDA Created [seed: "market"]
    │                               │
    │                               ▼
    │                        Emit MarketInitialized Event
    │                               │
    │                               ├─ authority
    │                               ├─ trading_fee
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┘
    │
    ▼
Market Ready ✓ ──────────────────────────────────────► Users can trade
                                                       ► Orders can be created
```

### Trading.create_sell_order()

```
Seller (User)                Blockchain                     Order Book
─────────────                ──────────                     ──────────

User with Tokens ──────────► Trading.create_sell_order(amount, price_per_kwh)
    │                               │
    │                               │ [Validation: user has valid account]
    │                               │ [Validation: user has sufficient token balance]
    │                               │ [Validation: amount >= min_order_size]
    │                               │ [Validation: amount <= max_order_size]
    │                               │ [Validation: price_per_kwh > 0]
    │                               │
    │                               ▼
    │                        Transfer Tokens to Escrow
    │                               │
    │                               │ [Amount: amount tokens]
    │                               │ [From: user token account]
    │                               │ [To: order escrow account]
    │                               │
    │                               ▼
    │                        Create Order Account
    │                               │
    │                               ├─ order_id = unique_id
    │                               ├─ seller = user.key()
    │                               ├─ buyer = None
    │                               ├─ order_type = Sell
    │                               ├─ amount = amount
    │                               ├─ price_per_kwh = price_per_kwh
    │                               ├─ status = Active
    │                               ├─ escrow_amount = amount
    │                               ├─ created_at = NOW
    │                               ├─ expires_at = NOW + 24h
    │                               │
    │                               ▼
    │                        Update Market
    │                               │
    │                               ├─ total_orders += 1
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        PDA Created [seed: "order", order_id]
    │                               │
    │                               ▼
    │                        Emit SellOrderCreated Event
    │                               │
    │                               ├─ order_id
    │                               ├─ seller
    │                               ├─ amount
    │                               ├─ price_per_kwh
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┼──────────────────────────► Order Book Updated
    │                               │                             │
    │                               │                             ├─ New sell order available
    │                               │                             ├─ Ready for matching
    │                               │                             │
    ▼                               │                             ▼
Sell Order Created ✓                │                      Available for Buyers ✓
```

### Trading.create_buy_order()

```
Buyer (User)                 Blockchain                     Order Book
────────────                 ──────────                     ──────────

User with Tokens ──────────► Trading.create_buy_order(amount, max_price_per_kwh)
    │                               │
    │                               │ [Validation: user has valid account]
    │                               │ [Validation: amount >= min_order_size]
    │                               │ [Validation: amount <= max_order_size]
    │                               │ [Validation: max_price_per_kwh > 0]
    │                               │
    │                               ▼
    │                        Calculate Required Escrow
    │                               │
    │                               │ [escrow_amount = amount * max_price_per_kwh]
    │                               │ [Validation: user has sufficient balance]
    │                               │
    │                               ▼
    │                        Transfer Tokens to Escrow
    │                               │
    │                               │ [Amount: escrow_amount tokens]
    │                               │ [From: user token account]
    │                               │ [To: order escrow account]
    │                               │
    │                               ▼
    │                        Create Order Account
    │                               │
    │                               ├─ order_id = unique_id
    │                               ├─ seller = None
    │                               ├─ buyer = user.key()
    │                               ├─ order_type = Buy
    │                               ├─ amount = amount
    │                               ├─ max_price_per_kwh = max_price_per_kwh
    │                               ├─ status = Active
    │                               ├─ escrow_amount = escrow_amount
    │                               ├─ created_at = NOW
    │                               ├─ expires_at = NOW + 24h
    │                               │
    │                               ▼
    │                        Update Market
    │                               │
    │                               ├─ total_orders += 1
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        PDA Created [seed: "order", order_id]
    │                               │
    │                               ▼
    │                        Emit BuyOrderCreated Event
    │                               │
    │                               ├─ order_id
    │                               ├─ buyer
    │                               ├─ amount
    │                               ├─ max_price_per_kwh
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┼──────────────────────────► Order Book Updated
    │                               │                             │
    │                               │                             ├─ New buy order available
    │                               │                             ├─ Ready for matching
    │                               │                             │
    ▼                               │                             ▼
Buy Order Created ✓                 │                      Available for Sellers ✓
```

### Trading.match_orders()

```
Market Maker/Authority       Blockchain                     Order Execution
──────────────────          ──────────                     ───────────────

Order Matching ──────────► Trading.match_orders()
    │                               │
    │                               │ [Find compatible buy/sell orders]
    │                               │ [Validation: sell_order.active]
    │                               │ [Validation: buy_order.active]
    │                               │ [Validation: buy_price >= sell_price]
    │                               │ [Validation: amounts compatible]
    │                               │
    │                               ▼
    │                        Calculate Trade Details
    │                               │
    │                               ├─ trade_amount = min(sell_amount, buy_amount)
    │                               ├─ trade_price = sell_order.price_per_kwh
    │                               ├─ total_value = trade_amount * trade_price
    │                               ├─ fee_amount = total_value * market.trading_fee / 10000
    │                               ├─ seller_receives = total_value - fee_amount
    │                               │
    │                               ▼
    │                        Execute Token Transfers
    │                               │
    │                               ├─ Transfer energy tokens to buyer
    │                               │ [Amount: trade_amount]
    │                               │ [From: sell_order escrow]
    │                               │ [To: buyer token account]
    │                               │
    │                               ├─ Transfer payment to seller
    │                               │ [Amount: seller_receives]
    │                               │ [From: buy_order escrow]
    │                               │ [To: seller token account]
    │                               │
    │                               ├─ Transfer fee to market
    │                               │ [Amount: fee_amount]
    │                               │ [From: buy_order escrow]
    │                               │ [To: market fee account]
    │                               │
    │                               ▼
    │                        Update Order Statuses
    │                               │
    │                               ├─ sell_order.status = Completed/PartiallyFilled
    │                               ├─ buy_order.status = Completed/PartiallyFilled
    │                               ├─ sell_order.filled_amount += trade_amount
    │                               ├─ buy_order.filled_amount += trade_amount
    │                               │
    │                               ▼
    │                        Create Trade Record
    │                               │
    │                               ├─ seller = sell_order.seller
    │                               ├─ buyer = buy_order.buyer
    │                               ├─ amount = trade_amount
    │                               ├─ price_per_kwh = trade_price
    │                               ├─ total_value = total_value
    │                               ├─ fee_amount = fee_amount
    │                               ├─ executed_at = NOW
    │                               │
    │                               ▼
    │                        Update Market Statistics
    │                               │
    │                               ├─ total_trades += 1
    │                               ├─ total_volume += trade_amount
    │                               ├─ last_updated = NOW
    │                               │
    │                               ▼
    │                        Emit OrderMatched Event
    │                               │
    │                               ├─ sell_order_id
    │                               ├─ buy_order_id
    │                               ├─ seller
    │                               ├─ buyer
    │                               ├─ amount
    │                               ├─ price_per_kwh
    │                               ├─ total_value
    │                               ├─ fee_amount
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┼──────────────────────────► Trade Completed
    │                               │                             │
    │                               │                             ├─ Seller gets payment tokens
    │                               │                             ├─ Buyer gets energy tokens
    │                               │                             ├─ Market gets fee
    │                               │                             │
    ▼                               │                             ▼
Orders Matched ✓                    │                      Energy Trade Complete ✓
```

### Trading.cancel_order()

```
Order Creator                Blockchain                     Order Cancellation
─────────────                ──────────                     ───────────────────

User ──────────► Trading.cancel_order(order_id)
    │                               │
    │                               │ [Validation: order exists]
    │                               │ [Validation: caller == order.seller || caller == order.buyer]
    │                               │ [Validation: order.status == Active || PartiallyFilled]
    │                               │
    │                               ▼
    │                        Return Escrowed Tokens
    │                               │
    │                               │ [Return remaining escrow to user]
    │                               │ [Amount: order.escrow_amount - filled_amount]
    │                               │
    │                               ▼
    │                        Update Order
    │                               │
    │                               ├─ order.status = Cancelled
    │                               ├─ order.cancelled_at = NOW
    │                               ├─ order.escrow_amount = 0
    │                               │
    │                               ▼
    │                        Emit OrderCancelled Event
    │                               │
    │                               ├─ order_id
    │                               ├─ user
    │                               ├─ order_type
    │                               ├─ refunded_amount
    │                               ├─ timestamp
    │                               │
    │◄──────────────────────────────┼──────────────────────────► Order Book Updated
    │                               │                             │
    │                               │                             ├─ Order removed from matching
    │                               │                             ├─ Tokens returned to user
    │                               │                             │
    ▼                               │                             ▼
Order Cancelled ✓                   │                      Funds Returned ✓
```

---

## Complete System Flows

### Complete Energy Generation to Trading Flow

```
Physical World              Oracle/API Gateway           Governance               Registry                Energy-Token             Trading
──────────────              ──────────────────           ──────────               ────────                ────────────             ───────

Solar Panels
Generate 100 kWh
    │
    ▼
AMI Meter Reading ──────► Oracle.submit_meter_reading() ──► Registry.update_meter_reading()
    │                          │                                │
    │                          │ [MeterReadingSubmitted]        │ [MeterReadingUpdated]
    │                          │                                │ [meter.total_generation += 100]
    │                          │                                │
    │                          │                                ▼
    │                          │                        Governance.issue_erc() ──────► Energy-Token.transfer_tokens()
    │                          │                                │                             │
    │                          │                        [ErcIssued event]                    │ [TokenTransferred event]
    │                          │                        [certificate created]                │ [100 tokens to user]
    │                          │                                │                             │
    │                          │                                ▼                             ▼
    │                          │                        Governance.validate_erc_for_trading() User Ready to Trade
    │                          │                                │
    │                          │                        [ErcValidatedForTrading]
    │                          │                        [validated_for_trading = true]
    │                          │                                │
    │                          │                                ▼
    │                          │                        Trading.create_sell_order() ◄──────── User Creates Order
    │                          │                                │                             │
    │                          │                        [SellOrderCreated]                   │ [50 kWh @ 10 tokens/kWh]
    │                          │                        [order in escrow]                    │ [50 tokens escrowed]
    │                          │                                │                             │
    │                          │                                ▼                             ▼
    │                          │                        Order Available               Another User Creates
    │                          │                        for Matching                  Buy Order
    │                          │                                │                             │
    │                          │                                ▼                             ▼
    │                          │                        Trading.match_orders() ◄──────────── Market Maker/Timer
    │                          │                                │
    │                          │                        [OrderMatched event]
    │                          │                        [seller: 498.75 tokens]
    │                          │                        [buyer: 50 energy tokens]
    │                          │                        [market: 1.25 tokens fee]
    │                          │                                │
    │                          │                                ▼
    │                          │                        Trade Complete ✓
    │                          │                        Energy Economy Active
```

### Emergency Response Flow

```
Threat Detection            Governance                   All Programs                 System State
─────────────              ──────────                   ────────────                 ────────────

Security Alert ──────────► emergency_pause()
    │                             │
    │                             │ [emergency_paused = true]
    │                             │
    │                             ▼
    │                      Broadcast Pause Signal ──────────► Oracle: Block readings
    │                             │                           Registry: Block registrations
    │                             │                           Energy-Token: Block transfers
    │                             │                           Trading: Block orders
    │                             │                              │
    │                             │                              ▼
    │◄────────────────────────────┼◄─────────────────────── System Frozen ✓
    │                             │
    │                             │
Issue Resolved ──────────► emergency_unpause()
    │                             │
    │                             │ [emergency_paused = false]
    │                             │
    │                             ▼
    │                      Broadcast Resume Signal ──────────► Oracle: Resume readings
    │                             │                            Registry: Resume registrations
    │                             │                            Energy-Token: Resume transfers
    │                             │                            Trading: Resume orders
    │                             │                               │
    │                             │                               ▼
    │◄────────────────────────────┼◄─────────────────────── System Normal ✓
    │                             │
    ▼
System Operational ✓
```

---

**[← Back to Architecture Overview](./README.md)**