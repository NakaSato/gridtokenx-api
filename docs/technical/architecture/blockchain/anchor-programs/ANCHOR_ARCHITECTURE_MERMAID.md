# Anchor Architecture - Mermaid Diagrams

This document contains Mermaid diagrams that render directly in GitHub, VS Code, and many documentation platforms without additional tools.

## Table of Contents

1. [High-Level Architecture](#high-level-architecture)
2. [Program Relationships](#program-relationships)
3. [Account Structure](#account-structure)
4. [Token Minting Flow](#token-minting-flow)
5. [Trading Flow](#trading-flow)
6. [ERC Certification Flow](#erc-certification-flow)

---

## High-Level Architecture

```mermaid
graph TB
    subgraph "Off-Chain"
        Meter[Smart Meters]
        AMI[AMI System]
        API[API Gateway]
        User[Users/Prosumers]
    end
    
    subgraph "Solana Blockchain"
        subgraph "Anchor Programs"
            Registry[Registry Program<br/>Bxvy5Y...]
            Token[Energy Token Program<br/>6LgvcJ...]
            Oracle[Oracle Program<br/>2Jqh9J...]
            Trading[Trading Program<br/>Hzmt59...]
            Governance[Governance Program<br/>83V1DX...]
        end
        
        subgraph "Solana Ledger"
            PDAs[Program Accounts<br/>PDAs]
            UserAccts[User Accounts<br/>Keypairs]
            SPL[SPL Token Accounts]
        end
    end
    
    Meter --> AMI
    AMI --> API
    User --> API
    
    API --> Oracle
    API --> Registry
    API --> Token
    API --> Trading
    
    Oracle -.CPI.-> Registry
    Token -.CPI.-> Registry
    Trading -.CPI.-> Token
    Governance -.Read.-> Registry
    
    Registry --> PDAs
    Token --> PDAs
    Oracle --> PDAs
    Trading --> PDAs
    Governance --> PDAs
    
    Registry --> UserAccts
    Token --> SPL
    
    style Registry fill:#90EE90
    style Token fill:#87CEEB
    style Oracle fill:#FFD700
    style Trading fill:#FFA500
    style Governance fill:#FFB6C1
```

---

## Program Relationships

```mermaid
graph LR
    subgraph "External"
        APIGateway[API Gateway<br/>Rust Service]
        RECAuth[REC Authority]
    end
    
    subgraph "Registry Program"
        RegUser[register_user]
        RegMeter[register_meter]
        UpdateReading[update_meter_reading]
        Settle[settle_meter_balance]
    end
    
    subgraph "Energy Token Program"
        MintTokens[mint_grid_tokens]
        Transfer[transfer_tokens]
        Burn[burn_tokens]
    end
    
    subgraph "Oracle Program"
        Submit[submit_meter_reading]
        TriggerClearing[trigger_market_clearing]
    end
    
    subgraph "Trading Program"
        CreateSell[create_sell_order]
        CreateBuy[create_buy_order]
        MatchOrders[match_orders]
    end
    
    subgraph "Governance Program"
        IssueERC[issue_erc]
        ValidateERC[validate_erc_for_trading]
        EmergencyPause[emergency_pause]
    end
    
    APIGateway -->|Submit readings| Submit
    APIGateway -->|Register users| RegUser
    APIGateway -->|Mint tokens| MintTokens
    
    Submit -.CPI.-> UpdateReading
    MintTokens -.CPI.-> Settle
    MatchOrders -.CPI.-> Transfer
    
    RECAuth -->|Issue certificates| IssueERC
    RECAuth -->|Emergency control| EmergencyPause
    
    IssueERC -.Read.-> RegMeter
    
    style APIGateway fill:#DDD
    style RECAuth fill:#FFB6C1
```

---

## Account Structure

```mermaid
classDiagram
    class Registry {
        +Pubkey authority
        +u64 user_count
        +u64 meter_count
        +i64 created_at
    }
    
    class UserAccount {
        +Pubkey authority
        +UserType user_type
        +String location
        +UserStatus status
        +i64 registered_at
        +u32 meter_count
    }
    
    class MeterAccount {
        +String meter_id
        +Pubkey owner
        +MeterType meter_type
        +MeterStatus status
        +u64 total_generation
        +u64 total_consumption
        +u64 settled_net_generation
        +u64 claimed_erc_generation
    }
    
    class TokenInfo {
        +Pubkey authority
        +Pubkey mint
        +u64 total_supply
        +i64 created_at
    }
    
    class OracleData {
        +Pubkey authority
        +Pubkey api_gateway
        +u64 total_readings
        +i64 last_reading_timestamp
        +bool active
    }
    
    class Market {
        +Pubkey authority
        +u64 active_orders
        +u64 total_volume
        +u64 total_trades
        +bool clearing_enabled
        +u16 market_fee_bps
    }
    
    class Order {
        +Pubkey seller
        +Pubkey buyer
        +u64 amount
        +u64 filled_amount
        +u64 price_per_kwh
        +OrderType order_type
        +OrderStatus status
    }
    
    class PoAConfig {
        +Pubkey authority
        +String authority_name
        +bool emergency_paused
        +bool maintenance_mode
        +bool erc_validation_enabled
        +u64 min_energy_amount
        +u64 max_erc_amount
        +u64 total_ercs_issued
    }
    
    class ErcCertificate {
        +String certificate_id
        +Pubkey authority
        +u64 energy_amount
        +String renewable_source
        +String validation_data
        +ErcStatus status
        +bool validated_for_trading
    }
    
    Registry "1" --> "*" UserAccount : manages
    UserAccount "1" --> "*" MeterAccount : owns
    Market "1" --> "*" Order : contains
    PoAConfig "1" --> "*" ErcCertificate : issues
    MeterAccount "1" --> "*" ErcCertificate : certifies
```

---

## Token Minting Flow

```mermaid
sequenceDiagram
    participant Meter as Smart Meter
    participant AMI as AMI System
    participant API as API Gateway
    participant Oracle as Oracle Program
    participant Registry as Registry Program
    participant User as Prosumer
    participant Token as Energy Token Program
    participant SPL as SPL Token Program
    
    Note over Meter,Registry: Phase 1: Data Collection
    Meter->>AMI: Report: 5000Wh generated, 2000Wh consumed
    AMI->>API: POST /meter-reading
    API->>Oracle: submit_meter_reading(5000, 2000)
    Oracle->>Oracle: Verify: signer == api_gateway
    Oracle->>Registry: CPI: update_meter_reading()
    Registry->>Registry: Update meter:<br/>total_generation += 5000<br/>total_consumption += 2000
    Registry-->>Oracle: Success
    Oracle-->>API: Success
    
    Note over User,SPL: Phase 2: Token Minting
    User->>API: POST /tokens/mint
    API->>Token: mint_grid_tokens()
    Token->>Registry: CPI: settle_meter_balance()
    Registry->>Registry: Calculate:<br/>current_net = 5000 - 2000 = 3000<br/>settled = 0<br/>to_mint = 3000 - 0 = 3000
    Registry->>Registry: Update:<br/>settled_net_generation = 3000
    Registry-->>Token: Return: 3000 tokens
    Token->>SPL: CPI: mint_to(3000)
    SPL->>SPL: Mint 3000 GRID tokens
    SPL-->>Token: Success
    Token->>Token: Update total_supply += 3000
    Token-->>API: Success
    API-->>User: {minted: 3000 tokens}
```

---

## Trading Flow

```mermaid
sequenceDiagram
    participant SellerUser as Prosumer A
    participant BuyerUser as Consumer B
    participant API as API Gateway
    participant Trading as Trading Program
    participant Token as Energy Token Program
    
    Note over SellerUser,Token: Step 1: Create Sell Order
    SellerUser->>API: POST /trading/sell<br/>{amount: 2000Wh, price: 0.15}
    API->>Trading: create_sell_order()
    Trading->>Token: CPI: transfer(300 GRID to escrow)
    Token-->>Trading: Locked in escrow
    Trading->>Trading: Create Order PDA<br/>market.active_orders += 1
    Trading-->>API: sell_order_id
    API-->>SellerUser: Order created
    
    Note over BuyerUser,Token: Step 2: Create Buy Order
    BuyerUser->>API: POST /trading/buy<br/>{amount: 1500Wh, max_price: 0.16}
    API->>Trading: create_buy_order()
    Trading->>Token: CPI: transfer(240 GRID to escrow)
    Token-->>Trading: Locked in escrow
    Trading->>Trading: Create Order PDA<br/>market.active_orders += 1
    Trading-->>API: buy_order_id
    API-->>BuyerUser: Order created
    
    Note over API,Token: Step 3: Match Orders
    API->>Trading: match_orders()
    Trading->>Trading: Validate:<br/>buy_price >= sell_price ✓<br/>match_amount = 1500Wh<br/>total_value = 225 GRID<br/>fee = 0.5625 GRID
    Trading->>Token: CPI: transfer(224.44 GRID to seller)
    Token-->>Trading: Transferred
    Trading->>Token: CPI: transfer(0.5625 GRID to platform)
    Token-->>Trading: Transferred
    Trading->>Token: CPI: transfer(15 GRID excess to buyer)
    Token-->>Trading: Transferred
    Trading->>Trading: Update orders<br/>Create TradeRecord<br/>market.total_trades += 1
    Trading-->>API: Trade completed
    API-->>SellerUser: Sold 1500Wh for 224.44 GRID
    API-->>BuyerUser: Bought 1500Wh for 225 GRID
```

---

## ERC Certification Flow

```mermaid
sequenceDiagram
    participant User as Prosumer
    participant API as API Gateway
    participant REC as REC Authority
    participant Gov as Governance Program
    participant Reg as Registry Program
    
    Note over User,Reg: Prerequisite: Meter has 10,000Wh generated
    
    Note over User,Gov: Step 1: Request ERC Issuance
    User->>API: POST /governance/erc/issue<br/>{meter_id, amount: 5000Wh, source: "Solar"}
    API->>REC: Forward for approval
    REC->>REC: Manual verification:<br/>✓ Verify meter readings<br/>✓ Verify renewable source<br/>✓ Verify compliance
    
    Note over REC,Reg: Step 2: Issue ERC Certificate
    REC->>Gov: issue_erc(5000Wh, "Solar")
    Gov->>Gov: Validate:<br/>✓ System operational<br/>✓ ERC validation enabled<br/>✓ REC authority signature<br/>✓ Amount within limits
    Gov->>Reg: Read MeterAccount
    Reg-->>Gov: total_generation: 10,000Wh<br/>claimed_erc_generation: 0Wh
    Gov->>Gov: Check double-claim:<br/>available = 10,000 - 0 = 10,000Wh<br/>request = 5000Wh ✓
    Gov->>Gov: Create ErcCertificate PDA<br/>certificate_id: "ERC-2025-11-07-001"<br/>energy_amount: 5000Wh<br/>status: Valid
    Gov->>Reg: Update MeterAccount:<br/>claimed_erc_generation += 5000
    Reg-->>Gov: Updated (now claimed = 5000Wh)
    Gov->>Gov: Update PoAConfig:<br/>total_ercs_issued += 1<br/>total_energy_certified += 5000
    Gov-->>REC: ERC issued successfully
    REC-->>API: {certificate_id: "ERC-2025-11-07-001"}
    API-->>User: ERC certificate created
    
    Note over User,Gov: Step 3: Validate for Trading (Optional)
    User->>API: POST /governance/erc/validate
    API->>REC: Request validation
    REC->>Gov: validate_erc_for_trading()
    Gov->>Gov: Verify:<br/>✓ Status = Valid<br/>✓ Not expired<br/>✓ REC authority signature
    Gov->>Gov: Update ErcCertificate:<br/>validated_for_trading = true
    Gov->>Gov: Update PoAConfig:<br/>total_ercs_validated += 1
    Gov-->>REC: Validated
    REC-->>API: Success
    API-->>User: ERC validated for trading
```

---

## Security Layers

```mermaid
graph TB
    subgraph "Layer 6: Blockchain Guarantees"
        Immutable[Immutable History]
        Consensus[Distributed Consensus]
        Crypto[Cryptographic Signatures]
    end
    
    subgraph "Layer 5: Event Auditing"
        Events[Event Emission]
        Logs[Transaction Logs]
        Monitoring[Off-chain Monitoring]
    end
    
    subgraph "Layer 4: Double-Spend Prevention"
        MintTrack[settled_net_generation]
        ERCTrack[claimed_erc_generation]
        Escrow[Trading Escrow]
    end
    
    subgraph "Layer 3: Instruction Validation"
        AuthCheck[Authority Checks]
        StateCheck[State Validation]
        AmountCheck[Amount Validation]
        OwnerCheck[Ownership Verification]
    end
    
    subgraph "Layer 2: PDA Security"
        PDAs[Program Derived Addresses]
        Seeds[Deterministic Seeds]
        ProgramOwned[Program-Owned Only]
    end
    
    subgraph "Layer 1: Authority Hierarchy"
        SysAdmin[System Admin]
        RECAuth[REC Authority]
        APIGateway[API Gateway]
        Users[Users]
    end
    
    SysAdmin --> PDAs
    RECAuth --> AuthCheck
    APIGateway --> StateCheck
    Users --> OwnerCheck
    
    PDAs --> AuthCheck
    Seeds --> PDAs
    ProgramOwned --> PDAs
    
    AuthCheck --> MintTrack
    StateCheck --> ERCTrack
    AmountCheck --> Escrow
    OwnerCheck --> MintTrack
    
    MintTrack --> Events
    ERCTrack --> Events
    Escrow --> Events
    
    Events --> Logs
    Logs --> Monitoring
    Monitoring --> Immutable
    
    Immutable --> Consensus
    Consensus --> Crypto
    
    style SysAdmin fill:#FF6B6B
    style RECAuth fill:#FFB6C1
    style APIGateway fill:#FFD93D
    style Users fill:#6BCB77
    style MintTrack fill:#4D96FF
    style ERCTrack fill:#4D96FF
    style Escrow fill:#4D96FF
```

---

## CPI Pattern: Energy Token → Registry

```mermaid
sequenceDiagram
    participant User as User/Prosumer
    participant Token as Energy Token Program
    participant Registry as Registry Program
    participant Meter as MeterAccount
    participant SPL as SPL Token Program
    
    User->>Token: Call mint_grid_tokens()
    activate Token
    
    Token->>Token: 1. Validate user owns meter
    
    Note over Token,Registry: CPI Security: Cross-Program Invocation
    Token->>Registry: 2. CPI: settle_meter_balance()
    activate Registry
    
    Registry->>Meter: 3. Read meter data
    activate Meter
    Meter-->>Registry: current_net_gen = 3000Wh<br/>settled_net_gen = 0Wh
    deactivate Meter
    
    Registry->>Registry: 4. Calculate:<br/>tokens = 3000 - 0 = 3000
    
    Registry->>Meter: 5. Update:<br/>settled_net_gen = 3000
    activate Meter
    Meter-->>Registry: Updated (prevents double-mint)
    deactivate Meter
    
    Registry-->>Token: 6. Return: 3000 tokens to mint
    deactivate Registry
    
    Token->>Token: 7. Prepare PDA signer<br/>seeds: ["token_info", bump]
    
    Note over Token,SPL: PDA Signing: Only program can sign
    Token->>SPL: 8. CPI with signer: mint_to(3000)
    activate SPL
    SPL->>SPL: Mint 3000 tokens to user
    SPL-->>Token: Success
    deactivate SPL
    
    Token->>Token: 9. Update total_supply += 3000
    Token->>Token: 10. Emit GridTokensMinted event
    
    Token-->>User: Success: 3000 tokens minted
    deactivate Token
    
    Note over Token,Meter: Security: settled_net_generation prevents<br/>the same energy from being minted twice
```

---

## Emergency Controls

```mermaid
stateDiagram-v2
    [*] --> Normal: initialize_poa()
    
    Normal --> EmergencyPaused: emergency_pause()
    Normal --> Maintenance: set_maintenance_mode(true)
    
    EmergencyPaused --> Normal: emergency_unpause()
    Maintenance --> Normal: set_maintenance_mode(false)
    
    state Normal {
        [*] --> Active
        Active --> IssuingERCs: issue_erc()
        IssuingERCs --> ValidatingERCs: validate_erc_for_trading()
        ValidatingERCs --> Active
    }
    
    state EmergencyPaused {
        [*] --> AllOperationsStopped
        note right of AllOperationsStopped
            - No ERC issuance
            - No trading
            - No token minting
            - Only REC Authority can unpause
        end note
    }
    
    state Maintenance {
        [*] --> LimitedOperations
        note right of LimitedOperations
            - Read operations allowed
            - Write operations blocked
            - Configuration updates allowed
        end note
    }
    
    Normal: ✓ All operations allowed
    Normal: ✓ ERC issuance enabled
    Normal: ✓ Trading enabled
    Normal: ✓ Token minting enabled
```

---

**Note:** These Mermaid diagrams render automatically in:
- GitHub README files
- GitLab documentation
- VS Code with Mermaid extension
- Notion
- Many documentation platforms

For PlantUML diagrams with more advanced features, see `ANCHOR_ARCHITECTURE_DIAGRAMS.puml`.
