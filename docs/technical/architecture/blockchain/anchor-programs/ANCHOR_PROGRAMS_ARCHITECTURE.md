# 🏗️ Anchor Programs - Architecture & Code Structure

**GridTokenX Platform - Technical Architecture**

---

## Table of Contents

1. [System Architecture](#system-architecture)
2. [Data Flow Diagrams](#data-flow-diagrams)
3. [Code Structure](#code-structure)
4. [Account Relationships](#account-relationships)
5. [Transaction Flows](#transaction-flows)
6. [Authorization Matrix](#authorization-matrix)

---

## System Architecture

### High-Level System Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                    Solana Blockchain (PoA)                       │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              Anchor Smart Contracts Layer                  │ │
│  ├────────────────────────────────────────────────────────────┤ │
│  │                                                            │ │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │ │
│  │  │   Oracle    │  │ Governance   │  │    Registry     │  │ │
│  │  │  Program    │  │   Program    │  │    Program      │  │ │
│  │  │             │  │              │  │                 │  │ │
│  │  │ • Meter     │  │ • ERC        │  │ • Users         │  │ │
│  │  │   readings  │  │   issuance   │  │ • Meters        │  │ │
│  │  │ • Market    │  │ • ERC        │  │ • Validation    │  │ │
│  │  │   clearing  │  │   validation │  │                 │  │ │
│  │  │ • API Gw    │  │ • Pause/     │  │                 │  │ │
│  │  │   auth      │  │   Unpause    │  │                 │  │ │
│  │  └─────────────┘  └──────────────┘  └─────────────────┘  │ │
│  │        │                 │                  │             │ │
│  │        └─────────────────┼──────────────────┘             │ │
│  │                          │                                 │ │
│  │        ┌─────────────────▼──────────────────┐            │ │
│  │        │      Energy Token Program         │            │ │
│  │        │  • Token transfers                │            │ │
│  │        │  • Token burning                  │            │ │
│  │        │  • SPL Token integration          │            │ │
│  │        └─────────────────┬──────────────────┘            │ │
│  │                          │                                 │ │
│  │        ┌─────────────────▼──────────────────┐            │ │
│  │        │      Trading Program              │            │ │
│  │        │  • Buy orders                     │            │ │
│  │        │  • Sell orders                    │            │ │
│  │        │  • Order matching                 │            │ │
│  │        │  • Market clearing                │            │ │
│  │        └───────────────────────────────────┘            │ │
│  │                                                            │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
         △                                          △
         │                                          │
    ┌────┴───────────────────────────────────┬─────┴──────┐
    │                                        │            │
    │                                        │            │
    ▼                                        ▼            ▼
┌─────────────┐                      ┌────────────┐  ┌──────────┐
│  AMI Data   │                      │  Solana    │  │  Users/  │
│  (Meters)   │                      │  Network   │  │  Clients │
└─────────────┘                      └────────────┘  └──────────┘
```

### Program Dependencies

```
Trading ────────────────┐
                        │
Energy-Token ◄──────────┴─ Uses SPL Token Program
   │                        │
   │                        │
   └─────────────►  Registry ◄──── Oracle
                        │
                        │
                   Governance
```

---

## Data Flow Diagrams

### Oracle Program Functions

#### Oracle.initialize()

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

#### Oracle.submit_meter_reading()

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

#### Oracle.trigger_market_clearing()

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

#### Oracle.update_oracle_status()

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

#### Oracle.update_api_gateway()

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

### Governance Program Functions

#### Governance.initialize_poa()

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

#### Governance.emergency_pause()

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

#### Governance.emergency_unpause()

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

#### Governance.issue_erc()

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

#### Governance.validate_erc_for_trading()

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

### Registry Program Functions

#### Registry.initialize()

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

#### Registry.register_user()

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

#### Registry.register_meter()

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

#### Registry.update_meter_reading()

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

### Energy-Token Program Functions

#### Energy-Token.initialize()

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

#### Energy-Token.initialize_token()

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

#### Energy-Token.transfer_tokens()

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

#### Energy-Token.burn_tokens()

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

### Trading Program Functions

#### Trading.initialize()

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

#### Trading.initialize_market()

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

#### Trading.create_sell_order()

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

#### Trading.create_buy_order()

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

#### Trading.match_orders()

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

#### Trading.cancel_order()

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

---

## Code Structure

### Oracle Program Structure

```
oracle/src/lib.rs
├── Module Declaration
│   └── mod oracle { ... }
│
├── Program ID
│   └── declare_id!("ApwexmUbEZMpez5dJXKza4V7gqSqWvAA9BPbok2psxXg")
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

### Governance Program Structure

```
governance/src/lib.rs
├── Module Declaration
│   └── mod governance { ... }
│
├── Program ID
│   └── declare_id!("Dy8JFn95L1E7NoUkXbFQtW1kGR7Ja21CkNcirNgv4ghe")
│
├── Functions (10)
│   ├── fn initialize_poa(ctx) -> Result<()>
│   ├── fn emergency_pause(ctx) -> Result<()>
│   ├── fn emergency_unpause(ctx) -> Result<()>
│   ├── fn issue_erc(ctx, cert_id, energy_amt, ...) -> Result<()>
│   ├── fn validate_erc_for_trading(ctx) -> Result<()>
│   ├── fn update_governance_config(ctx, enabled) -> Result<()>
│   ├── fn set_maintenance_mode(ctx, enabled) -> Result<()>
│   ├── fn update_erc_limits(ctx, min, max, period) -> Result<()>
│   ├── fn update_authority_info(ctx, contact) -> Result<()>
│   └── fn get_governance_stats(ctx) -> Result<GovernanceStats>
│
├── Account Structs (6)
│   ├── struct InitializePoa<'info> { ... }
│   ├── struct EmergencyControl<'info> { ... }
│   ├── struct IssueErc<'info> { ... }
│   ├── struct ValidateErc<'info> { ... }
│   ├── struct UpdateGovernanceConfig<'info> { ... }
│   └── struct GetGovernanceStats<'info> { ... }
│
├── Data Structs (3)
│   ├── struct PoAConfig { ... }
│   ├── struct ErcCertificate { ... }
│   └── struct GovernanceStats { ... }
│
├── Enums (1)
│   └── enum ErcStatus { Valid, Expired, Revoked, Pending }
│
├── Events (8)
│   ├── struct PoAInitialized
│   ├── struct EmergencyPauseActivated
│   ├── struct EmergencyPauseDeactivated
│   ├── struct ErcIssued
│   ├── struct ErcValidatedForTrading
│   ├── struct GovernanceConfigUpdated
│   ├── struct MaintenanceModeUpdated
│   ├── struct ErcLimitsUpdated
│   └── struct AuthorityInfoUpdated
│
└── Error Codes (16)
    ├── UnauthorizedAuthority
    ├── AlreadyPaused
    ├── NotPaused
    ├── SystemPaused
    ├── MaintenanceMode
    ├── ErcValidationDisabled
    ├── InvalidErcStatus
    ├── AlreadyValidated
    ├── BelowMinimumEnergy
    ├── ExceedsMaximumEnergy
    ├── CertificateIdTooLong
    ├── SourceNameTooLong
    ├── ErcExpired
    ├── InvalidMinimumEnergy
    ├── InvalidMaximumEnergy
    ├── InvalidValidityPeriod
    └── ContactInfoTooLong
```

### Registry Program Structure

```
registry/src/lib.rs
├── Module Declaration
│   └── mod registry { ... }
│
├── Program ID
│   └── declare_id!("42LoRKPphBBdvaCDx2ZjNuZFqzXuJziiiNXyiV6FhBY5")
│
├── Functions (8)
│   ├── fn initialize(ctx) -> Result<()>
│   ├── fn register_user(ctx, user_type, location) -> Result<()>
│   ├── fn register_meter(ctx, meter_id, meter_type) -> Result<()>
│   ├── fn update_user_status(ctx, new_status) -> Result<()>
│   ├── fn update_meter_reading(ctx, gen, cons, timestamp) -> Result<()>
│   ├── fn is_valid_user(ctx) -> Result<bool>
│   ├── fn is_valid_meter(ctx) -> Result<bool>
│   └── fn assign_meter(ctx, meter_id) -> Result<()>
│
├── Account Structs (6)
│   ├── struct Initialize<'info> { ... }
│   ├── struct RegisterUser<'info> { ... }
│   ├── struct RegisterMeter<'info> { ... }
│   ├── struct UpdateUserStatus<'info> { ... }
│   ├── struct UpdateMeterReading<'info> { ... }
│   ├── struct IsValidUser<'info> { ... }
│   ├── struct IsValidMeter<'info> { ... }
│   └── struct AssignMeter<'info> { ... }
│
├── Data Structs (3)
│   ├── struct Registry { ... }
│   ├── struct UserAccount { ... }
│   └── struct MeterAccount { ... }
│
├── Enums (3)
│   ├── enum UserType { Prosumer, Consumer }
│   ├── enum UserStatus { Active, Suspended, Inactive }
│   └── enum MeterStatus { Active, Inactive, Maintenance }
│
├── Events (5)
│   ├── struct RegistryInitialized
│   ├── struct UserRegistered
│   ├── struct MeterRegistered
│   ├── struct UserStatusUpdated
│   └── struct MeterReadingUpdated
│
└── Error Codes (6)
    ├── UnauthorizedUser
    ├── UnauthorizedAuthority
    ├── InvalidUserStatus
    ├── InvalidMeterStatus
    ├── UserNotFound
    └── MeterNotFound
```

### Trading Program Structure

```
trading/src/lib.rs
├── Module Declaration
│   └── mod trading { ... }
│
├── Program ID
│   └── declare_id!("dS3zvp95PFVrNNBfZDXn78QL5MvhUqDCFR4rn8z9Jgh")
│
├── Functions (7)
│   ├── fn initialize(ctx) -> Result<()>
│   ├── fn initialize_market(ctx) -> Result<()>
│   ├── fn create_sell_order(ctx, amount, price) -> Result<()>
│   ├── fn create_buy_order(ctx, amount, max_price) -> Result<()>
│   ├── fn match_orders(ctx) -> Result<()>
│   ├── fn cancel_order(ctx, order_id) -> Result<()>
│   └── fn update_market_params(ctx, fee, clearing) -> Result<()>
│
├── Account Structs (7)
│   ├── struct Initialize<'info> { ... }
│   ├── struct InitializeMarket<'info> { ... }
│   ├── struct CreateSellOrder<'info> { ... }
│   ├── struct CreateBuyOrder<'info> { ... }
│   ├── struct MatchOrders<'info> { ... }
│   ├── struct CancelOrder<'info> { ... }
│   └── struct UpdateMarketParams<'info> { ... }
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
│   └── struct MarketParamsUpdated
│
└── Error Codes (8)
    ├── UnauthorizedAuthority
    ├── InvalidAmount
    ├── InvalidPrice
    ├── InactiveSellOrder
    ├── InactiveBuyOrder
    ├── PriceMismatch
    ├── OrderNotCancellable
    └── InsufficientEscrowBalance
```

---

## Account Relationships

### Account Dependency Graph

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
      ├─► Order (multiple) [seed: b"order_id"]
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

## Transaction Flows

### Transaction Flow 1: Complete Energy Trade

```
Step 1: Initialization
┌─────────────────────────────────────────────┐
│ Registry.initialize()                       │
│ Oracle.initialize()                         │
│ Governance.initialize_poa()                 │
│ Energy-Token.initialize_token()             │
│ Trading.initialize_market()                 │
└─────────────────────────────────────────────┘
         │
         ▼

Step 2: User Registration
┌─────────────────────────────────────────────┐
│ User A: Registry.register_user(Prosumer)    │
│ User B: Registry.register_user(Consumer)    │
│                                             │
│ Create UserAccounts:                        │
│ - UserA [PDA: "user", UserA_pubkey]        │
│ - UserB [PDA: "user", UserB_pubkey]        │
└─────────────────────────────────────────────┘
         │
         ▼

Step 3: Meter Registration
┌─────────────────────────────────────────────┐
│ UserA: Registry.register_meter(             │
│   "SOLAR-001", Solar                        │
│ )                                           │
│                                             │
│ Create MeterAccount:                        │
│ - Meter [PDA: "meter", "SOLAR-001"]        │
│   Owner: UserA                              │
└─────────────────────────────────────────────┘
         │
         ▼

Step 4: Energy Generation & Recording
┌─────────────────────────────────────────────┐
│ Physical: Solar panel generates 100 kWh     │
│                                             │
│ Oracle.submit_meter_reading(                │
│   "SOLAR-001", 100, 0, timestamp            │
│ )                                           │
│                                             │
│ Registry updates:                           │
│ - Meter.total_generation += 100             │
└─────────────────────────────────────────────┘
         │
         ▼

Step 5: ERC Issuance
┌─────────────────────────────────────────────┐
│ Governance.issue_erc(                       │
│   "CERT-001", 100, "Solar", "validated"     │
│ )                                           │
│                                             │
│ Create ErcCertificate:                      │
│ - Cert [PDA: "erc_cert", "CERT-001"]       │
│   Status: Valid                             │
│   Energy: 100 kWh                           │
└─────────────────────────────────────────────┘
         │
         ▼

Step 6: ERC Validation
┌─────────────────────────────────────────────┐
│ Governance.validate_erc_for_trading()       │
│                                             │
│ Updates ErcCertificate:                     │
│ - validated_for_trading = true              │
│ - trading_validated_at = NOW                │
└─────────────────────────────────────────────┘
         │
         ▼

Step 7: Token Issuance
┌─────────────────────────────────────────────┐
│ Energy-Token.transfer_tokens(100)           │
│                                             │
│ Via SPL Token Program:                      │
│ - Transfer 100 tokens to UserA              │
│ - UserA TokenAccount balance: 100           │
└─────────────────────────────────────────────┘
         │
         ▼

Step 8: Order Creation (Seller)
┌─────────────────────────────────────────────┐
│ UserA: Trading.create_sell_order(           │
│   50, 10                                    │
│ ) [50 kWh @ 10 tokens/kWh]                 │
│                                             │
│ Create Order:                               │
│ - OrderA [seed: order_id]                   │
│   seller: UserA                             │
│   amount: 50                                │
│   price_per_kwh: 10                         │
│   status: Active                            │
│   escrow: 50 tokens (deducted)             │
│   balance: 50 tokens remaining              │
└─────────────────────────────────────────────┘
         │
         ▼

Step 9: Order Creation (Buyer)
┌─────────────────────────────────────────────┐
│ UserB has 1000 tokens                       │
│                                             │
│ Trading.create_buy_order(                   │
│   50, 10                                    │
│ ) [want 50 kWh @ max 10 tokens/kWh]        │
│                                             │
│ Create Order:                               │
│ - OrderB [seed: order_id]                   │
│   buyer: UserB                              │
│   amount: 50                                │
│   max_price_per_kwh: 10                     │
│   status: Active                            │
│   escrow: 500 tokens (50 * 10)              │
│   balance: 500 tokens remaining             │
└─────────────────────────────────────────────┘
         │
         ▼

Step 10: Order Matching
┌─────────────────────────────────────────────┐
│ Trading.match_orders()                      │
│                                             │
│ Checks:                                     │
│ - OrderA (sell) active ✓                    │
│ - OrderB (buy) active ✓                     │
│ - Prices compatible ✓ (10 == 10)            │
│ - Sufficient escrow ✓                       │
│                                             │
│ Calculations:                               │
│ - total_value = 50 * 10 = 500 tokens        │
│ - fee = 500 * 0.0025 = 1.25 tokens          │
│ - seller_gets = 500 - 1.25 = 498.75 tokens │
│                                             │
│ Transfers:                                  │
│ - 50 tokens → UserB TokenAccount            │
│ - 498.75 tokens → UserA TokenAccount        │
│ - 1.25 tokens → Market (fee)                │
│                                             │
│ Create TradeRecord:                         │
│ - seller: UserA                             │
│ - buyer: UserB                              │
│ - amount: 50                                │
│ - price: 10                                 │
│ - total_value: 500                          │
│ - fee_amount: 1.25                          │
│                                             │
│ Update Orders:                              │
│ - OrderA.status = Completed                 │
│ - OrderB.status = Completed                 │
│ - Market.total_trades += 1                  │
│ - Market.total_volume += 50                 │
└─────────────────────────────────────────────┘
         │
         ▼

Final State:
┌─────────────────────────────────────────────┐
│ UserA (Seller):                             │
│ - Started: 100 tokens, 0 consumption        │
│ - Sold: 50 kWh @ 10 tokens                  │
│ - Final: 548.75 tokens (50 + 498.75)        │
│                                             │
│ UserB (Buyer):                              │
│ - Started: 1000 tokens, 0 consumption       │
│ - Bought: 50 kWh @ 10 tokens                │
│ - Final: 550 tokens (1000 - 500 + 50)       │
│            + 50 kWh energy                  │
│                                             │
│ Market:                                     │
│ - Fee collected: 1.25 tokens                │
│ - Total volume: 50 kWh                      │
│ - Total trades: 1                           │
└─────────────────────────────────────────────┘
```

---

## Authorization Matrix

### Who Can Call Each Function?

```
┌─────────────────────────────────┬───────────┬──────────┬────────┬───────┐
│ Function                        │ Authority │ API Gw   │ User   │ Other │
├─────────────────────────────────┼───────────┼──────────┼────────┼───────┤
│ ORACLE                          │           │          │        │       │
│ initialize                      │ ✅        │          │        │       │
│ submit_meter_reading            │           │ ✅       │        │       │
│ trigger_market_clearing         │           │ ✅       │        │       │
│ update_oracle_status            │ ✅        │          │        │       │
│ update_api_gateway              │ ✅        │          │        │       │
├─────────────────────────────────┼───────────┼──────────┼────────┼───────┤
│ GOVERNANCE                      │           │          │        │       │
│ initialize_poa                  │ ✅        │          │        │       │
│ issue_erc                       │ ✅        │          │        │       │
│ validate_erc_for_trading        │ ✅        │          │        │       │
│ emergency_pause                 │ ✅        │          │        │       │
│ emergency_unpause               │ ✅        │          │        │       │
│ update_governance_config        │ ✅        │          │        │       │
│ set_maintenance_mode            │ ✅        │          │        │       │
│ update_erc_limits               │ ✅        │          │        │       │
│ update_authority_info           │ ✅        │          │        │       │
│ get_governance_stats            │ ✅        │ ✅       │ ✅     │ ✅    │
├─────────────────────────────────┼───────────┼──────────┼────────┼───────┤
│ REGISTRY                        │           │          │        │       │
│ initialize                      │ ✅        │          │        │       │
│ register_user                   │           │          │ ✅     │       │
│ register_meter                  │           │          │ ✅     │       │
│ update_meter_reading            │           │ ✅       │        │       │
│ update_user_status              │ ✅        │          │        │       │
│ is_valid_user                   │ ✅        │ ✅       │ ✅     │ ✅    │
│ is_valid_meter                  │ ✅        │ ✅       │ ✅     │ ✅    │
├─────────────────────────────────┼───────────┼──────────┼────────┼───────┤
│ ENERGY-TOKEN                    │           │          │        │       │
│ initialize                      │ ✅        │          │        │       │
│ initialize_token                │ ✅        │          │        │       │
│ add_rec_validator               │ ✅        │          │        │       │
│ transfer_tokens                 │           │          │ ✅     │       │
│ burn_tokens                     │           │          │ ✅     │       │
├─────────────────────────────────┼───────────┼──────────┼────────┼───────┤
│ TRADING                         │           │          │        │       │
│ initialize                      │ ✅        │          │        │       │
│ initialize_market               │ ✅        │          │        │       │
│ create_sell_order               │           │          │ ✅     │       │
│ create_buy_order                │           │          │ ✅     │       │
│ match_orders                    │ ✅        │          │        │       │
│ cancel_order                    │           │          │ ✅     │       │
│ update_market_params            │ ✅        │          │        │       │
└─────────────────────────────────┴───────────┴──────────┴────────┴───────┘

Legend:
✅ = Authorized to call
(blank) = Not authorized

Authority = Program admin/authority account
API Gw = API Gateway (for Oracle)
User = Individual user/signer
Other = Public/anyone
```

---

## Call Sequence Diagrams

### Oracle Program Function Sequences

#### Oracle.initialize() Sequence

```
Authority           OracleProgram       SystemProgram       Blockchain
    │                     │                   │                │
    │──initialize()────────►│                   │                │
    │  (api_gateway)       │                   │                │
    │                      │                   │                │
    │                      │─create_pda()─────►│                │
    │                      │  [seed: "oracle_data"]             │
    │                      │                   │                │
    │                      │◄─pda_created──────│                │
    │                      │                   │                │
    │                      │─set_authority()───┼───────────────►│
    │                      │─set_api_gateway()─┼───────────────►│
    │                      │─set_active(true)──┼───────────────►│
    │                      │─set_created_at()──┼───────────────►│
    │                      │                   │                │
    │◄─success─────────────│                   │                │
    │                      │                   │                │
```

#### Oracle.submit_meter_reading() Sequence

```
APIGateway         OracleProgram       RegistryProgram     Blockchain
    │                     │                   │                │
    │──submit_reading()────►│                   │                │
    │  (meter_id, data)    │                   │                │
    │                      │                   │                │
    │                      │─validate_caller()─┼───────────────►│
    │                      │─check_active()────┼───────────────►│
    │                      │                   │                │
    │                      │─update_readings()─┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │                      │─update_meter()────►│                │
    │                      │                   │─update_data()──►│
    │                      │                   │◄─updated───────│
    │                      │◄─success──────────│                │
    │                      │                   │                │
    │◄─success─────────────│                   │                │
    │                      │                   │                │
```

#### Oracle.trigger_market_clearing() Sequence

```
APIGateway         OracleProgram       TradingProgram      Blockchain
    │                     │                   │                │
    │──trigger_clearing()──►│                   │                │
    │                      │                   │                │
    │                      │─validate_caller()─┼───────────────►│
    │                      │─check_active()────┼───────────────►│
    │                      │                   │                │
    │                      │─update_clearing()─┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │                      │─initiate_matching()►│               │
    │                      │                   │─match_orders()─►│
    │                      │                   │◄─trades_executed│
    │                      │◄─success──────────│                │
    │                      │                   │                │
    │◄─success─────────────│                   │                │
    │                      │                   │                │
```

### Governance Program Function Sequences

#### Governance.initialize_poa() Sequence

```
Authority          GovernanceProgram   SystemProgram       Blockchain
    │                     │                   │                │
    │──initialize_poa()────►│                   │                │
    │                      │                   │                │
    │                      │─create_pda()─────►│                │
    │                      │  [seed: "poa_config"]              │
    │                      │                   │                │
    │                      │◄─pda_created──────│                │
    │                      │                   │                │
    │                      │─set_authority()───┼───────────────►│
    │                      │─set_config()──────┼───────────────►│
    │                      │─set_limits()──────┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │◄─success─────────────│                   │                │
    │                      │                   │                │
```

#### Governance.issue_erc() Sequence

```
Authority          GovernanceProgram   SystemProgram       Blockchain
    │                     │                   │                │
    │──issue_erc()─────────►│                   │                │
    │  (cert_id, amount)   │                   │                │
    │                      │                   │                │
    │                      │─validate_authority()──────────────►│
    │                      │─check_emergency()─┼───────────────►│
    │                      │─validate_limits()─┼───────────────►│
    │                      │                   │                │
    │                      │─create_cert_pda()─►│                │
    │                      │  [seed: "erc_certificate", cert_id]│
    │                      │                   │                │
    │                      │◄─cert_created─────│                │
    │                      │                   │                │
    │                      │─set_cert_data()───┼───────────────►│
    │                      │─update_counters()─┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │◄─erc_issued──────────│                   │                │
    │                      │                   │                │
```

#### Governance.validate_erc_for_trading() Sequence

```
Authority          GovernanceProgram   EnergyTokenProgram  Blockchain
    │                     │                   │                │
    │──validate_erc()──────►│                   │                │
    │  (cert_id)           │                   │                │
    │                      │                   │                │
    │                      │─validate_authority()──────────────►│
    │                      │─check_emergency()─┼───────────────►│
    │                      │─check_cert_valid()┼───────────────►│
    │                      │                   │                │
    │                      │─update_cert()─────┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │                      │─trigger_tokens()──►│                │
    │                      │                   │─transfer_tokens()│
    │                      │                   │◄─tokens_issued─│
    │                      │◄─success──────────│                │
    │                      │                   │                │
    │◄─validated───────────│                   │                │
    │                      │                   │                │
```

#### Governance.emergency_pause() Sequence

```
Authority          GovernanceProgram   AllPrograms         Blockchain
    │                     │                   │                │
    │──emergency_pause()───►│                   │                │
    │                      │                   │                │
    │                      │─validate_authority()──────────────►│
    │                      │─check_not_paused()┼───────────────►│
    │                      │                   │                │
    │                      │─set_paused(true)──┼───────────────►│
    │                      │─set_timestamp()───┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │                      │─notify_pause()────►│                │
    │                      │                   │─block_functions()│
    │                      │                   │◄─paused────────│
    │                      │◄─system_paused────│                │
    │                      │                   │                │
    │◄─emergency_active────│                   │                │
    │                      │                   │                │
```

### Registry Program Function Sequences

#### Registry.register_user() Sequence

```
User              RegistryProgram     SystemProgram       Blockchain
    │                     │                   │                │
    │──register_user()─────►│                   │                │
    │  (type, location)    │                   │                │
    │                      │                   │                │
    │                      │─validate_input()──┼───────────────►│
    │                      │                   │                │
    │                      │─create_user_pda()─►│                │
    │                      │  [seed: "user", user.key()]        │
    │                      │                   │                │
    │                      │◄─user_created─────│                │
    │                      │                   │                │
    │                      │─set_user_data()───┼───────────────►│
    │                      │─update_registry()─┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │◄─user_registered─────│                   │                │
    │                      │                   │                │
```

#### Registry.register_meter() Sequence

```
User              RegistryProgram     SystemProgram       Blockchain
    │                     │                   │                │
    │──register_meter()────►│                   │                │
    │  (meter_id, type)    │                   │                │
    │                      │                   │                │
    │                      │─validate_user()───┼───────────────►│
    │                      │─validate_input()──┼───────────────►│
    │                      │                   │                │
    │                      │─create_meter_pda()►│                │
    │                      │  [seed: "meter", meter_id]         │
    │                      │                   │                │
    │                      │◄─meter_created────│                │
    │                      │                   │                │
    │                      │─set_meter_data()──┼───────────────►│
    │                      │─update_user()─────┼───────────────►│
    │                      │─update_registry()─┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │◄─meter_registered────│                   │                │
    │                      │                   │                │
```

#### Registry.update_meter_reading() Sequence

```
Oracle            RegistryProgram     GovernanceProgram   Blockchain
    │                     │                   │                │
    │──update_reading()────►│                   │                │
    │  (meter_id, data)    │                   │                │
    │                      │                   │                │
    │                      │─validate_meter()──┼───────────────►│
    │                      │─check_active()────┼───────────────►│
    │                      │                   │                │
    │                      │─update_meter()────┼───────────────►│
    │                      │─update_user()─────┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │                      │─trigger_erc()─────►│                │
    │                      │                   │─issue_erc()────►│
    │                      │                   │◄─erc_issued────│
    │                      │◄─processing───────│                │
    │                      │                   │                │
    │◄─reading_updated─────│                   │                │
    │                      │                   │                │
```

### Energy-Token Program Function Sequences

#### Energy-Token.initialize_token() Sequence

```
Authority         EnergyTokenProgram  SPLTokenProgram     Blockchain
    │                     │                   │                │
    │──initialize_token()──►│                   │                │
    │                      │                   │                │
    │                      │─validate_authority()──────────────►│
    │                      │                   │                │
    │                      │─create_mint()─────►│                │
    │                      │  (decimals: 6)    │                │
    │                      │                   │                │
    │                      │◄─mint_created─────│                │
    │                      │                   │                │
    │                      │─update_token_info()───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │◄─token_ready─────────│                   │                │
    │                      │                   │                │
```

#### Energy-Token.transfer_tokens() Sequence

```
User/System       EnergyTokenProgram  SPLTokenProgram     Blockchain
    │                     │                   │                │
    │──transfer_tokens()───►│                   │                │
    │  (amount, recipient) │                   │                │
    │                      │                   │                │
    │                      │─validate_amount()─┼───────────────►│
    │                      │                   │                │
    │                      │─get_token_account()►│               │
    │                      │                   │─create_if_needed│
    │                      │                   │◄─account_ready─│
    │                      │◄─account_exists───│                │
    │                      │                   │                │
    │                      │─mint_to()─────────►│                │
    │                      │                   │─mint_tokens()──►│
    │                      │                   │◄─tokens_minted─│
    │                      │◄─transfer_complete│                │
    │                      │                   │                │
    │                      │─update_supply()───┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │◄─tokens_transferred──│                   │                │
    │                      │                   │                │
```

#### Energy-Token.burn_tokens() Sequence

```
User              EnergyTokenProgram  SPLTokenProgram     Blockchain
    │                     │                   │                │
    │──burn_tokens()───────►│                   │                │
    │  (amount)            │                   │                │
    │                      │                   │                │
    │                      │─validate_balance()┼───────────────►│
    │                      │─validate_amount()─┼───────────────►│
    │                      │                   │                │
    │                      │─burn_from()───────►│                │
    │                      │                   │─burn_tokens()──►│
    │                      │                   │◄─tokens_burned─│
    │                      │◄─burn_complete────│                │
    │                      │                   │                │
    │                      │─update_supply()───┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │◄─tokens_burned───────│                   │                │
    │                      │                   │                │
```

### Trading Program Function Sequences

#### Trading.initialize_market() Sequence

```
Authority         TradingProgram      SystemProgram       Blockchain
    │                     │                   │                │
    │──initialize_market()─►│                   │                │
    │                      │                   │                │
    │                      │─validate_authority()──────────────►│
    │                      │                   │                │
    │                      │─create_market_pda()►│               │
    │                      │  [seed: "market"]  │                │
    │                      │                   │                │
    │                      │◄─market_created───│                │
    │                      │                   │                │
    │                      │─set_market_config()───────────────►│
    │                      │─set_trading_fee()─┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │◄─market_ready────────│                   │                │
    │                      │                   │                │
```

#### Trading.create_sell_order() Sequence

```
Seller            TradingProgram      SPLTokenProgram     Blockchain
    │                     │                   │                │
    │──create_sell_order()─►│                   │                │
    │  (amount, price)     │                   │                │
    │                      │                   │                │
    │                      │─validate_user()───┼───────────────►│
    │                      │─validate_amount()─┼───────────────►│
    │                      │─check_balance()───┼───────────────►│
    │                      │                   │                │
    │                      │─transfer_escrow()─►│                │
    │                      │                   │─transfer_tokens()│
    │                      │                   │◄─tokens_escrowed│
    │                      │◄─escrow_complete──│                │
    │                      │                   │                │
    │                      │─create_order()────┼───────────────►│
    │                      │─update_market()───┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │◄─order_created───────│                   │                │
    │                      │                   │                │
```

#### Trading.create_buy_order() Sequence

```
Buyer             TradingProgram      SPLTokenProgram     Blockchain
    │                     │                   │                │
    │──create_buy_order()──►│                   │                │
    │  (amount, max_price) │                   │                │
    │                      │                   │                │
    │                      │─validate_user()───┼───────────────►│
    │                      │─validate_amount()─┼───────────────►│
    │                      │─calculate_escrow()┼───────────────►│
    │                      │─check_balance()───┼───────────────►│
    │                      │                   │                │
    │                      │─transfer_escrow()─►│                │
    │                      │                   │─transfer_tokens()│
    │                      │                   │◄─tokens_escrowed│
    │                      │◄─escrow_complete──│                │
    │                      │                   │                │
    │                      │─create_order()────┼───────────────►│
    │                      │─update_market()───┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │◄─order_created───────│                   │                │
    │                      │                   │                │
```

#### Trading.match_orders() Sequence

```
MarketMaker       TradingProgram      SPLTokenProgram     Blockchain
    │                     │                   │                │
    │──match_orders()──────►│                   │                │
    │                      │                   │                │
    │                      │─find_orders()─────┼───────────────►│
    │                      │─validate_match()──┼───────────────►│
    │                      │─calculate_trade()─┼───────────────►│
    │                      │                   │                │
    │                      │─transfer_energy()─►│                │
    │                      │  (to buyer)       │─transfer_tokens()│
    │                      │                   │◄─energy_transferred│
    │                      │◄─transfer_1───────│                │
    │                      │                   │                │
    │                      │─transfer_payment()►│                │
    │                      │  (to seller)      │─transfer_tokens()│
    │                      │                   │◄─payment_transferred│
    │                      │◄─transfer_2───────│                │
    │                      │                   │                │
    │                      │─transfer_fee()────►│                │
    │                      │  (to market)      │─transfer_tokens()│
    │                      │                   │◄─fee_transferred│
    │                      │◄─transfer_3───────│                │
    │                      │                   │                │
    │                      │─update_orders()───┼───────────────►│
    │                      │─create_trade()────┼───────────────►│
    │                      │─update_market()───┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │◄─trade_executed──────│                   │                │
    │                      │                   │                │
```

#### Trading.cancel_order() Sequence

```
User              TradingProgram      SPLTokenProgram     Blockchain
    │                     │                   │                │
    │──cancel_order()──────►│                   │                │
    │  (order_id)          │                   │                │
    │                      │                   │                │
    │                      │─validate_owner()──┼───────────────►│
    │                      │─check_cancellable()───────────────►│
    │                      │                   │                │
    │                      │─return_escrow()───►│                │
    │                      │                   │─transfer_tokens()│
    │                      │                   │◄─tokens_returned│
    │                      │◄─escrow_returned──│                │
    │                      │                   │                │
    │                      │─update_order()────┼───────────────►│
    │                      │─emit_event()──────┼───────────────►│
    │                      │                   │                │
    │◄─order_cancelled─────│                   │                │
    │                      │                   │                │
```

### End-to-End Sequence Diagrams

#### Complete Energy Generation to Trading Sequence

```
Physical     AMI      APIGateway   Oracle    Registry   Governance   EnergyToken   Trading    Users
   │          │           │          │         │           │            │           │         │
   │─generate─►│           │          │         │           │            │           │         │
   │ 100kWh   │           │          │         │           │            │           │         │
   │          │─reading───►│          │         │           │            │           │         │
   │          │           │─submit───►│         │           │            │           │         │
   │          │           │          │─update──►│           │            │           │         │
   │          │           │          │         │─trigger───►│            │           │         │
   │          │           │          │         │           │─issue_erc──►│           │         │
   │          │           │          │         │           │◄─erc_issued─│           │         │
   │          │           │          │         │           │─validate───►│           │         │
   │          │           │          │         │           │─trigger─────►│           │         │
   │          │           │          │         │           │             │─transfer──►│         │
   │          │           │          │         │           │             │◄─tokens───│         │
   │          │           │          │         │           │             │           │◄─ready──│
   │          │           │          │         │           │             │           │─create──►│
   │          │           │          │         │           │             │           │ sell     │
   │          │           │          │         │           │             │           │◄─order───│
   │          │           │          │         │           │             │           │         │
   │          │           │          │         │           │             │           │◄─buy─────│
   │          │           │          │         │           │             │           │ order    │
   │          │           │          │         │           │             │           │         │
   │          │           │─trigger───────────────────────────────────────────────────►│         │
   │          │           │ clearing │         │           │             │           │         │
   │          │           │          │         │           │             │           │─match────►│
   │          │           │          │         │           │             │           │ orders   │
   │          │           │          │         │           │             │           │◄─trade───│
   │          │           │          │         │           │             │           │ complete │
   │          │           │          │         │           │             │           │         │
```

#### User Onboarding and First Trade Sequence

```
NewUser      Registry    Oracle    Governance   EnergyToken   Trading    Blockchain
   │             │          │          │            │           │            │
   │─register────►│          │          │            │           │            │
   │ user        │─create───┼──────────┼────────────┼───────────┼───────────►│
   │             │ account  │          │            │           │            │
   │◄─registered─│          │          │            │           │            │
   │             │          │          │            │           │            │
   │─register────►│          │          │            │           │            │
   │ meter       │─create───┼──────────┼────────────┼───────────┼───────────►│
   │             │ meter    │          │            │           │            │
   │◄─meter_ready│          │          │            │           │            │
   │             │          │          │            │           │            │
   │─generate────┼──────────►│          │            │           │            │
   │ energy      │          │─reading──►│            │           │            │
   │             │          │          │─issue_erc──►│           │            │
   │             │          │          │◄─erc───────│           │            │
   │             │          │          │─validate───►│           │            │
   │             │          │          │─trigger─────►│           │            │
   │             │          │          │             │─transfer──►│            │
   │◄─tokens─────┼──────────┼──────────┼─────────────│◄─tokens───│            │
   │ received    │          │          │             │           │            │
   │             │          │          │             │           │            │
   │─create──────┼──────────┼──────────┼─────────────┼───────────►│            │
   │ sell_order  │          │          │             │           │─escrow─────►│
   │◄─order──────┼──────────┼──────────┼─────────────┼───────────│◄─order─────│
   │ created     │          │          │             │           │ created    │
   │             │          │          │             │           │            │
```

#### Emergency Response Sequence

```
Monitor      Authority    Governance   Oracle    Registry   EnergyToken   Trading   
   │             │            │          │         │           │            │       
   │─detect──────►│            │          │         │           │            │       
   │ threat      │            │          │         │           │            │       
   │             │─emergency───►│          │         │           │            │       
   │             │ pause       │─pause────┼─────────┼───────────┼────────────►│       
   │             │             │          │         │           │            │       
   │             │             │─notify───►│         │           │            │       
   │             │             │          │─disable─┼───────────┼────────────►│       
   │             │             │          │         │           │            │       
   │             │             │─notify───┼─────────►│           │            │       
   │             │             │          │         │─disable───┼────────────►│       
   │             │             │          │         │           │            │       
   │             │             │─notify───┼─────────┼───────────►│            │       
   │             │             │          │         │           │─disable────►│       
   │             │             │          │         │           │            │       
   │             │             │─notify───┼─────────┼───────────┼────────────►│       
   │             │             │          │         │           │            │─block─
   │             │             │          │         │           │            │ all   
   │◄─system─────│◄─emergency──│◄─paused──│◄─paused─│◄─paused───│◄─paused────│◄─funcs
   │ frozen      │ active      │          │         │           │            │       
   │             │             │          │         │           │            │       
   │─threat──────►│            │          │         │           │            │       
   │ resolved    │            │          │         │           │            │       
   │             │─emergency───►│          │         │           │            │       
   │             │ unpause     │─unpause──┼─────────┼───────────┼────────────►│       
   │             │             │          │         │           │            │       
   │             │             │─enable───►│         │           │            │       
   │             │             │          │─enable──┼───────────┼────────────►│       
   │             │             │          │         │           │            │       
   │             │             │─enable───┼─────────►│           │            │       
   │             │             │          │         │─enable────┼────────────►│       
   │             │             │          │         │           │            │       
   │             │             │─enable───┼─────────┼───────────►│            │       
   │             │             │          │         │           │─enable─────►│       
   │             │             │          │         │           │            │       
   │             │             │─enable───┼─────────┼───────────┼────────────►│       
   │             │             │          │         │           │            │─resume
   │◄─system─────│◄─normal─────│◄─resumed─│◄─resumed│◄─resumed──│◄─resumed───│◄─funcs
   │ normal      │ operations  │          │         │           │            │       
   │             │             │          │         │           │            │       
```

---

---

**Architecture Documentation v1.0**  
**Generated:** November 1, 2025  
**Anchor Framework:** 0.32.1
