# 🏛️ System Architecture

**GridTokenX Platform - System Design & Dependencies**

---

## Table of Contents

1. [High-Level System Diagram](#high-level-system-diagram)
2. [Program Dependencies](#program-dependencies)
3. [Architecture Components](#architecture-components)
4. [External Integrations](#external-integrations)
5. [Network Architecture](#network-architecture)

---

## High-Level System Diagram

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

---

## Program Dependencies

> **📘 For detailed sequence diagrams and CPI patterns, see [Anchor Architecture Diagrams](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml)**

```
Registry ◄──── Oracle (submit_meter_reading → update_meter_reading)
   │
   │
   ├───► Energy Token (mint_grid_tokens → settle_meter_balance)
   │
   │
   ├───► Governance (issue_erc → read MeterAccount)
   │
   │
   └───► Trading (validate users)
            │
            └───► SPL Token (transfer operations)
```

### Dependency Descriptions

#### Oracle → Registry (CPI)
- **Purpose**: Updates meter readings in Registry after validation
- **Functions**: `submit_meter_reading()` triggers `update_meter_reading()`
- **Data Flow**: AMI meter data flows from Oracle to Registry for user/meter updates
- **Security**: Only authorized API Gateway can submit readings

> **📘 See [Oracle → Registry Sequence Diagram](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml#ANCHOR_SEQUENCE_TOKEN_MINTING)**

#### Energy Token → Registry (CPI)
- **Purpose**: Validate and settle meter balance before minting
- **Functions**: `mint_grid_tokens()` calls `settle_meter_balance()`
- **Data Flow**: Registry calculates unsettled energy and prevents double-minting
- **Security**: settled_net_generation tracker prevents re-minting same energy

> **📘 See [Token Minting Flow](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#token-minting-flow)**

#### Governance → Registry (Read)
- **Purpose**: Validate meter data before issuing ERCs
- **Functions**: `issue_erc()` reads MeterAccount data
- **Data Flow**: Governance reads total_generation and claimed_erc_generation
- **Security**: claimed_erc_generation tracker prevents double-claiming

> **📘 See [ERC Certification Flow](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#erc-certification-flow)**

#### Trading → Energy Token (CPI)
- **Purpose**: Escrow and settle token transfers for trades
- **Functions**: Order creation/matching triggers token transfers
- **Data Flow**: Atomic multi-transfer for seller, buyer, platform fee
- **Security**: Escrow prevents double-spend, atomic settlement prevents partial execution

> **📘 See [P2P Trading Flow](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#p2p-trading-flow)**

#### Energy Token → SPL Token Program (CPI)
- **Purpose**: Standard token operations (mint, transfer, burn)
- **Integration**: Energy Token program acts as wrapper around SPL Token Program
- **Benefits**: Leverages battle-tested Solana token infrastructure
- **Security**: PDA signing ensures only Energy Token program can mint

> **📘 See [CPI Security Patterns](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#cpi-security)**

---

## Architecture Components

### Core Programs

> **📘 For complete program documentation with all instructions, account structures, and error codes, see [Anchor Architecture Overview](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md)**

#### 1. Registry Program
**Purpose**: User and meter registration/validation
- **Program ID**: `Bxvy5YBKoADe1BSTnj4cd117RLzfjUKG2WEk2iqcmVJE`
- **Authority**: System Administrator
- **User Types**: Prosumer, Consumer

**Key Responsibilities**:
- Register and manage user accounts
- Register and manage energy meters
- Validate user and meter status
- Track energy production/consumption
- Prevent double-minting via `settled_net_generation`
- Prevent double-claiming via `claimed_erc_generation`

**Key Instructions**:
- `register_user()` - Register new prosumers/consumers
- `register_meter()` - Register energy meters
- `update_meter_reading()` - Update meter data (CPI from Oracle)
- `settle_meter_balance()` - Calculate tokens to mint (CPI from Energy Token)

> **📘 See [Registry Program Details](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#registry-program) for complete instruction reference**

#### 2. Energy Token Program
**Purpose**: Energy tokenization via SPL tokens
- **Program ID**: `6LgvcJ8pxzSbzWCdaTWB2gUg4WazJv46eSjzj6LCNjNd`
- **Token**: GRID Token
- **Standard**: SPL Token
- **Decimals**: 9

**Key Responsibilities**:
- Initialize and manage GRID token mint
- Mint tokens for validated energy generation
- Transfer tokens for trading
- Burn tokens when needed
- Integrate with SPL Token Program via CPI

**Key Instructions**:
- `initialize_token()` - Initialize GRID token mint
- `mint_grid_tokens()` - Mint tokens for prosumers (with double-mint prevention)
- `transfer_tokens()` - Transfer GRID tokens
- `burn_tokens()` - Burn GRID tokens

> **📘 See [Energy Token Program Details](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#energy-token-program) for complete instruction reference**

#### 3. Oracle Program
**Purpose**: External data ingestion and market triggers
- **Program ID**: `2Jqh9JkxpJuWyqdzSDv3gskgMN9fT4K73P88a6xYAy4i`
- **Authority**: System Administrator
- **API Gateway**: Authorized external service

**Key Responsibilities**:
- Receive and validate AMI meter readings
- Trigger market clearing processes
- Maintain oracle status and configuration
- Authenticate API Gateway requests

**Key Instructions**:
- `initialize()` - Initialize oracle with API Gateway
- `submit_meter_reading()` - Submit meter data (API Gateway only)
- `trigger_market_clearing()` - Trigger market matching
- `update_oracle_status()` - Enable/disable oracle

> **📘 See [Oracle Program Details](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#oracle-program) for complete instruction reference**

#### 4. Trading Program
**Purpose**: Peer-to-peer energy marketplace
- **Program ID**: `Hzmt59DnHUKa8h8MJADgAf4zjREhvwZXW5ew5gnTnFPH`
- **Authority**: System Administrator
- **Fee Structure**: 0.25% trading fee (25 basis points)

**Key Responsibilities**:
- Create and manage buy/sell orders
- Execute order matching with automated clearing
- Handle escrow for secure trading
- Collect and distribute trading fees
- Atomic settlement via CPI

**Key Instructions**:
- `initialize_market()` - Initialize trading market
- `create_sell_order()` - Create sell order with escrow
- `create_buy_order()` - Create buy order with escrow
- `match_orders()` - Match and settle orders
- `cancel_order()` - Cancel order and return escrow

> **📘 See [Trading Program Details](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#trading-program) for complete instruction reference**

#### 5. Governance Program  
**Purpose**: System governance and ERC management
- **Program ID**: `83V1DXgURKYkPURCJbBKU3VzkqVjYcPKDuL6DRLKAGvw`
- **Authority**: University Engineering Department (REC Authority)
- **Model**: Proof of Authority (PoA)

**Key Responsibilities**:
- Issue Energy Renewable Certificates (ERCs)
- Validate ERCs for trading eligibility
- Emergency system pause/unpause controls
- Governance configuration management
- Prevent double-claiming via MeterAccount tracking

**Key Instructions**:
- `initialize_poa()` - Initialize PoA governance
- `issue_erc()` - Issue renewable energy certificates
- `validate_erc_for_trading()` - Validate ERC for marketplace
- `emergency_pause()` - Pause all operations
- `emergency_unpause()` - Resume operations

> **📘 See [Governance Program Details](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#governance-program) for complete instruction reference**

---

## External Integrations

### AMI (Advanced Metering Infrastructure)
```
AMI Meters ──readings──► API Gateway ──authenticated──► Oracle Program
```

**Integration Points**:
- **Data Source**: Smart meters (Solar, Wind, Battery, Grid)
- **Protocol**: RESTful API via API Gateway
- **Authentication**: API Gateway pubkey validation
- **Data Format**: Structured meter readings with timestamps

### SPL Token Program
```
Energy-Token Program ──calls──► SPL Token Program
```

**Integration Benefits**:
- **Battle-tested**: Leverages Solana's proven token infrastructure
- **Interoperability**: Compatible with existing Solana wallets and DeFi
- **Efficiency**: Optimized for high-throughput transactions
- **Standards**: Follows SPL token specifications

### User Interfaces
```
Web/Mobile Apps ──transactions──► GridTokenX Programs
```

**Client Integration**:
- **Wallet Connection**: Standard Solana wallet adapters
- **Transaction Signing**: User authorization for all operations
- **Real-time Updates**: Event listening for state changes
- **Error Handling**: Comprehensive error code handling

---

## Network Architecture

### Solana Blockchain Configuration
- **Consensus**: Proof of Authority (PoA)
- **Authority**: University Engineering Department
- **Network**: Custom/Private Solana deployment
- **Block Time**: ~400ms (Solana standard)
- **Finality**: Immediate (PoA consensus)

### Account Structure
```
Program Derived Addresses (PDAs):
├── Registry:
│   ├── [b"registry"] - Global registry state
│   ├── [b"user", user_pubkey] - User accounts
│   └── [b"meter", meter_id] - Meter accounts
├── Energy Token:
│   └── [b"token_info"] - Token mint info & authority
├── Oracle:
│   └── [b"oracle_data"] - Oracle configuration
├── Trading:
│   ├── [b"market"] - Market configuration
│   ├── [b"order", order_id] - Order accounts
│   └── [b"trade", trade_id] - Trade records
└── Governance:
    ├── [b"poa_config"] - PoA configuration
    └── [b"erc_certificate", cert_id] - ERC certificates
```

> **📘 For complete account structures with field details and sizes, see [Anchor Account Structure Diagram](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml#ANCHOR_ACCOUNT_STRUCTURE)**

### Security Model
- **Program Authority**: Multi-signature or single authority per program
- **Emergency Controls**: System-wide pause capability via Governance
- **Access Control**: Function-level authorization matrix
- **Data Validation**: Comprehensive input validation and business rules
- **Double-Spend Prevention**: 
  - Token Minting: `settled_net_generation` tracker
  - ERC Claiming: `claimed_erc_generation` tracker
  - Trading: Escrow-based atomic settlement
- **CPI Security**: 
  - Program ID validation
  - PDA signing for authorized operations
  - Account ownership verification

> **📘 For complete security analysis with 6 security layers, see [Anchor Security Model](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#security-model)**

---

## Scalability Considerations

### Performance Optimizations
- **PDA Design**: Optimized seed structures for efficient lookups
- **Event Emissions**: Comprehensive logging for off-chain processing
- **State Management**: Minimal on-chain state with strategic caching
- **Transaction Batching**: Support for bulk operations where applicable

### Growth Planning
- **Horizontal Scaling**: Additional meter and user capacity
- **Market Expansion**: Support for multiple trading pairs
- **Geographic Distribution**: Multi-region deployment capability
- **Integration Ready**: APIs for third-party integrations

---

## 📚 Related Documentation

### Comprehensive Anchor Documentation
- **[Anchor Documentation Hub](../anchor/README.md)** - Complete blockchain smart contract documentation
- **[Architecture Overview](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md)** - 85KB comprehensive technical reference
- **[PlantUML Diagrams](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml)** - 9 professional diagrams
- **[Mermaid Diagrams](../anchor/ANCHOR_ARCHITECTURE_MERMAID.md)** - GitHub-compatible visuals
- **[Quick Reference](../anchor/ANCHOR_QUICK_REFERENCE.md)** - Developer cheat sheet
- **[Complete Index](../anchor/INDEX.md)** - Role-based navigation

### Architecture Documentation
- **[Code Structure](./02-code-structure.md)** - Detailed program internals
- **[Data Flow Diagrams](./03-data-flow-diagrams.md)** - Function-level flows
- **[Transaction Flows](./04-transaction-flows.md)** - Business process sequences
- **[Authorization Matrix](./05-authorization-matrix.md)** - Access control reference
- **[Call Sequence Diagrams](./06-call-sequence-diagrams.md)** - Inter-program communication
- **[Account Relationships](./07-account-relationships.md)** - PDA dependencies

---

**[← Back to Architecture Overview](./README.md)**