# Data Flow Diagrams Index

## Overview

Data Flow Diagrams (DFDs) showing how data moves through the GridTokenX system at different levels of granularity.

## Diagram Hierarchy

### Level 0: Context Diagram

#### DFD_LEVEL_0.puml
**Description**: Highest level view of the entire system  
**Shows**:
- GridTokenX system as a single process
- External entities (users, operators, meters)
- Major data flows in and out

**Use Case**: Understanding system scope and external interfaces  
**Audience**: Stakeholders, business analysts

---

### Level 1: Major Processes

#### DFD_LEVEL_1.puml
**Description**: Main processes within the system  
**Shows**:
- User registration
- Energy generation and tokenization
- Trading and order management
- Market clearing and settlement
- Data stores (PostgreSQL, TimescaleDB, Redis)

**Use Case**: Understanding major functional areas  
**Audience**: System architects, developers

---

### Level 2: Detailed Subsystems

#### DFD_LEVEL_2_AUTH.puml
**Description**: Authentication and authorization flows  
**Shows**:
- Wallet connection
- JWT token generation
- User verification
- Authorization checks

**Use Case**: Security implementation and audit  
**Audience**: Security engineers, backend developers

#### DFD_LEVEL_2_BLOCKCHAIN.puml
**Description**: Blockchain interaction details  
**Shows**:
- Anchor program calls
- Account creation and updates
- Transaction signing
- On-chain data storage

**Use Case**: Blockchain integration understanding  
**Audience**: Blockchain developers

#### DFD_LEVEL_2_SMARTMETER.puml
**Description**: Smart meter data collection and processing  
**Shows**:
- AMI data submission
- Oracle validation
- Energy calculation
- Token minting
- TimescaleDB storage

**Use Case**: IoT integration and energy tokenization  
**Audience**: IoT developers, oracle maintainers

#### DFD_LEVEL_2_TRADING.puml
**Description**: Trading workflow details  
**Shows**:
- Order creation (buy/sell)
- Order book management
- Order matching algorithm
- Settlement process
- Trade logging

**Use Case**: Trading platform implementation  
**Audience**: Trading engine developers

## DFD Notation

### Elements
- **Process**: Rounded rectangle (transforms data)
- **External Entity**: Rectangle (source/destination of data)
- **Data Store**: Open rectangle (data repository)
- **Data Flow**: Arrow (data movement)

### Reading DFDs
1. Follow data flows from source to destination
2. Understand what each process does to the data
3. Note where data is stored
4. Track data transformations

## DFD Levels Explained

```
Level 0 (Context)
    ↓ Decompose
Level 1 (Major Processes)
    ↓ Decompose
Level 2 (Detailed Subsystems)
    ↓ Could decompose further
Level 3+ (Implementation details - not needed)
```

## Usage

### For LLMs
```bash
# Load in hierarchical order for understanding
cat DFD_LEVEL_0.puml          # System scope
cat DFD_LEVEL_1.puml          # Major processes

# Then load specific subsystems
cat DFD_LEVEL_2_AUTH.puml
cat DFD_LEVEL_2_BLOCKCHAIN.puml
cat DFD_LEVEL_2_SMARTMETER.puml
cat DFD_LEVEL_2_TRADING.puml
```

### For Developers

#### New Developer Onboarding
```
1. DFD_LEVEL_0 - System overview
2. DFD_LEVEL_1 - Functional areas
3. Relevant Level 2 - Your domain
```

#### Security Review
```
1. DFD_LEVEL_2_AUTH.puml - Authentication flows
2. All Level 2 diagrams - Authorization points
```

#### Integration Work
```
1. DFD_LEVEL_1 - Where your system fits
2. Relevant Level 2 - Interface details
```

## Diagram Files

| File | Level | Subsystem | Focus | Status |
|------|-------|-----------|-------|--------|
| DFD_LEVEL_0.puml | 0 | System | Context | ✅ Migrated |
| DFD_LEVEL_1.puml | 1 | All | Major processes | ✅ Migrated |
| DFD_LEVEL_2_AUTH.puml | 2 | Auth | User authentication | ✅ Migrated |
| DFD_LEVEL_2_BLOCKCHAIN.puml | 2 | Blockchain | On-chain operations | ✅ Migrated |
| DFD_LEVEL_2_SMARTMETER.puml | 2 | IoT | Energy data | ✅ Migrated |
| DFD_LEVEL_2_TRADING.puml | 2 | Trading | Order management | ✅ Migrated |

## Data Stores Reference

### PostgreSQL
- Users
- Orders
- Meters
- Settlements

### TimescaleDB
- Energy readings (time-series)
- Market data (time-series)
- Historical trades

### Redis
- Session cache
- Order book cache
- Real-time balances

### Solana Blockchain
- User accounts (PDAs)
- Token accounts (ATAs)
- Program state
- Transaction history

## Related Documentation

- **Sequence Diagrams**: `/technical/diagrams/sequence/` - Time-based flows
- **Component Diagrams**: `/technical/diagrams/component/` - Structure
- **Architecture**: `/technical/architecture/` - Design decisions
- **Specifications**: `/technical/specifications/processes/` - Detailed specs

## Best Practices

### Reading DFDs
1. Start at Level 0 for context
2. Move to Level 1 for process understanding
3. Dive into Level 2 for implementation details
4. Cross-reference with sequence diagrams for timing

### Using DFDs
- **Planning**: Identify data requirements
- **Implementation**: Understand data transformations
- **Testing**: Verify data flows
- **Documentation**: Show system logic

---
**Category**: Diagrams > Flow  
**Count**: 6 diagrams  
**Status**: ✅ Migration Complete
