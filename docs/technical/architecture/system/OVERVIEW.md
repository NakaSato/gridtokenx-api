---
title: GridTokenX System Architecture Overview
category: architecture
subcategory: system
last_updated: 2025-11-08
status: active
related_docs:
  - ../blockchain/BLOCKCHAIN_GUIDE.md
  - ../../diagrams/component/C4_LEVEL_2_CONTAINERS.puml
  - ../../diagrams/sequence/ARCHITECTURE_OVERVIEW_SEQUENCE.puml
tags: [architecture, system-design, overview, llm-context]
---

# GridTokenX System Architecture Overview

## Executive Summary

GridTokenX is a blockchain-based peer-to-peer energy trading platform for campus microgrids, enabling renewable energy producers (prosumers) to trade excess energy with consumers through tokenization on the Solana blockchain.

## System Purpose

### Primary Goals
1. **Energy Tokenization**: Convert kilowatt-hours (kWh) to tradeable SPL tokens
2. **P2P Trading**: Enable direct energy trading between campus users
3. **Market Clearing**: Automated 15-minute epoch-based settlement
4. **Transparency**: Blockchain-verified energy credits and transactions

### Target Users
- **Prosumers**: Users with solar panels generating excess energy
- **Consumers**: Users purchasing energy from the grid
- **REC (Renewable Energy Center)**: Campus authority managing the system
- **Grid Operators**: Monitoring and maintaining the platform

## High-Level Architecture

### System Boundaries

```
External Systems          GridTokenX Platform         Solana Blockchain
─────────────────        ─────────────────────        ──────────────────
┌──────────────┐         ┌──────────────┐             ┌──────────────┐
│ Smart Meters │────────▶│ API Gateway  │────────────▶│   Anchor     │
│  (AMI Data)  │         │  (Rust/Axum) │             │   Programs   │
└──────────────┘         └──────────────┘             └──────────────┘
                                │                              │
┌──────────────┐                │                              │
│Campus Users  │────────────────┤                              │
│(Web Browser) │                │                              │
└──────────────┘                ▼                              │
                         ┌──────────────┐                      │
                         │  PostgreSQL  │                      │
                         │  TimescaleDB │◀─────────────────────┘
                         │    Redis     │
                         └──────────────┘
```

## Core Components

### 1. Frontend Layer (React + Vite)

**Technology**: React 18, TypeScript, Vite, TailwindCSS

**Key Features**:
- Wallet integration (Solana via Gill SDK)
- Real-time trading dashboard
- Energy production/consumption visualization
- Order book and trading interface

**State Management**: React Query for server state

### 2. API Gateway (Rust + Axum)

**Technology**: Rust, Axum framework, SQLx, Redis

**Responsibilities**:
- RESTful API endpoints
- JWT authentication
- Business logic orchestration
- RPC communication with Solana
- Database operations
- Caching layer management

**Key Endpoints**:
- `/api/auth/*` - Authentication
- `/api/users/*` - User management
- `/api/energy/*` - Energy data submission
- `/api/trading/*` - Order management
- `/api/admin/*` - Administration

### 3. Blockchain Layer (Solana PoA)

**Technology**: Solana, Anchor Framework, Rust

**Architecture**: Proof of Authority (PoA) with single validator

**Programs** (Smart Contracts):

#### Registry Program
- User account management
- Meter assignment
- Authority verification

#### Energy Token Program (SPL)
- Token minting (1 kWh = 1,000,000,000 tokens)
- Token transfers
- Balance management

#### Trading Program
- Order creation (buy/sell)
- Order book management
- Trade execution
- Settlement coordination

#### Oracle Program
- AMI data validation
- Energy calculation
- Token minting triggers
- Off-chain data bridging

#### Governance Program
- System parameter updates
- Authority management
- Emergency controls

### 4. Data Storage Layer

#### PostgreSQL
**Purpose**: Relational data storage

**Schema**:
- `users` - User accounts and profiles
- `meters` - Smart meter registrations
- `orders` - Trading orders
- `settlements` - Trade settlements
- `audit_logs` - System audit trail

#### TimescaleDB
**Purpose**: Time-series data for energy metrics

**Hypertables**:
- `energy_readings` - AMI data (15-min intervals)
- `market_prices` - Historical pricing
- `trade_history` - Trade execution records

**Retention**: 2 years of historical data

#### Redis
**Purpose**: High-performance caching

**Data Structures**:
- Session cache (JWT tokens)
- Order book cache (sorted sets)
- User balance cache (hash maps)
- Real-time meter readings (strings)

**TTL**: 5-60 minutes depending on data type

### 5. Smart Meter System

**Technology**: Python simulator (production: actual AMI devices)

**Data Flow**:
1. Collect energy generation/consumption data
2. Submit to API Gateway every 15 minutes
3. Oracle validates and processes
4. Triggers token minting for generation

**Data Format**:
```json
{
  "meter_id": "METER_001",
  "timestamp": "2025-11-08T14:30:00Z",
  "kwh_generated": 2.5,
  "kwh_consumed": 1.8,
  "net_energy": 0.7
}
```

## System Workflows

### Workflow 1: User Registration
```
User → Connect Wallet → API Gateway → Registry Program → Blockchain
                                    ↓
                              PostgreSQL (user record)
```

### Workflow 2: Energy Generation & Tokenization
```
Smart Meter → API Gateway → Oracle Program → Energy Token Program
     ↓                            ↓                    ↓
TimescaleDB              Validation            Mint Tokens
                                                      ↓
                                              User's Token Account
```

### Workflow 3: Trading (15-min epochs)
```
Prosumer → Create Sell Order → Order Book (Redis + PostgreSQL)
Consumer → Create Buy Order → Order Book
                                    ↓
                          Epoch Timer (15 min)
                                    ↓
                          Matching Algorithm
                                    ↓
                     Trading Program (Settlement)
                                    ↓
                        Token Transfer + Payment
```

### Workflow 4: Market Clearing
```
Epoch End → API Gateway → Match Orders → Trading Program
                              ↓                ↓
                        Sort & Match     Execute Trades
                              ↓                ↓
                        PostgreSQL      Token Transfers
                              ↓                ↓
                        Update Status   Blockchain Record
```

## Technology Stack

### Frontend
- **Framework**: React 18
- **Build Tool**: Vite
- **Styling**: TailwindCSS
- **State**: React Query
- **Blockchain**: @solana/web3.js, Gill SDK
- **Charts**: Recharts

### Backend
- **Language**: Rust
- **Framework**: Axum
- **Database**: SQLx (PostgreSQL driver)
- **Cache**: Redis
- **Auth**: JWT (jsonwebtoken crate)

### Blockchain
- **Platform**: Solana
- **Framework**: Anchor
- **Language**: Rust
- **Consensus**: PoA (single validator)
- **Token**: SPL Token Standard

### Storage
- **Relational**: PostgreSQL 15
- **Time-Series**: TimescaleDB extension
- **Cache**: Redis 7
- **Files**: Local filesystem (development)

### Infrastructure
- **Containers**: Docker + Docker Compose
- **Monitoring**: Prometheus + Grafana
- **Logging**: Structured JSON logs
- **CI/CD**: GitHub Actions (planned)

## Security Architecture

### Authentication
- **Method**: Solana wallet signatures
- **Token**: JWT with 24-hour expiry
- **Storage**: HttpOnly cookies (web), secure storage (mobile)

### Authorization
- **Model**: Role-Based Access Control (RBAC)
- **Roles**: User, Admin, REC Authority
- **Validation**: On-chain PDAs + off-chain JWT claims

### Data Protection
- **In Transit**: TLS 1.3
- **At Rest**: Database encryption
- **Secrets**: Environment variables, never committed

### Blockchain Security
- **Consensus**: PoA with trusted validator
- **Programs**: Anchor security macros
- **Accounts**: PDA-based access control
- **Auditing**: Transaction logs on-chain

## Scalability Considerations

### Current Capacity
- **Users**: 1,000+ supported
- **Orders/epoch**: 500+ per 15-min window
- **Meter Readings**: 100 meters × 4 readings/hour
- **Transactions**: 50 TPS (Solana capability: 65,000 TPS)

### Horizontal Scaling
- API Gateway: Stateless, can run multiple instances
- PostgreSQL: Read replicas for queries
- Redis: Cluster mode for high availability

### Vertical Scaling
- Database connection pooling
- Redis memory optimization
- API Gateway thread pool tuning

## Monitoring & Observability

### Metrics (Prometheus)
- API response times
- Database query performance
- Blockchain transaction success rate
- Cache hit/miss ratios
- Order matching statistics

### Dashboards (Grafana)
- System health overview
- Trading volume and patterns
- Energy generation trends
- User activity metrics

### Logging
- Structured JSON logs
- Log levels: ERROR, WARN, INFO, DEBUG
- Correlation IDs for request tracing

### Alerts
- API downtime
- Blockchain RPC failures
- Database connection issues
- Abnormal trading patterns

## Development Environment

### Prerequisites
- Rust 1.70+
- Node.js 18+
- Solana CLI 1.16+
- Anchor 0.28+
- Docker & Docker Compose
- PostgreSQL client tools

### Quick Start
```bash
# 1. Clone repository
git clone https://github.com/NakaSato/gridtokenx-platform

# 2. Start infrastructure
docker-compose up -d

# 3. Build Anchor programs
cd anchor && anchor build && anchor deploy --provider.cluster localnet

# 4. Start API Gateway
cd api-gateway && cargo run

# 5. Start Frontend
cd frontend && npm install && npm run dev
```

### Development Workflow
1. Feature branch from `main`
2. Implement changes with tests
3. Run test suite: `make test`
4. Create pull request
5. Code review
6. Merge to `main`

## Deployment Architecture

### Local Development
- All services in Docker containers
- Local Solana test validator
- Hot reload for frontend and backend

### Staging (Planned)
- Solana Devnet
- Separate database instances
- Monitoring enabled

### Production (Future)
- Custom Solana PoA network
- High-availability setup
- Full monitoring and alerting
- Backup and disaster recovery

## API Versioning

### Current Version: v1
- Base path: `/api/v1/`
- Stability: Beta
- Breaking changes: Notified in advance

### Deprecation Policy
- 3-month notice for breaking changes
- 6-month support for deprecated endpoints
- Clear migration guides provided

## Performance Targets

### Response Times
- API: < 200ms (95th percentile)
- Blockchain: < 500ms (transaction confirmation)
- Order Matching: < 1 second (per epoch)

### Availability
- Target: 99.5% uptime
- Maintenance windows: Announced 48h in advance
- Graceful degradation during issues

## Future Enhancements

### Short Term (3 months)
- [ ] Mobile application (React Native)
- [ ] Advanced trading features (limit orders, stop-loss)
- [ ] Enhanced analytics dashboard
- [ ] Email notifications

### Medium Term (6 months)
- [ ] Multi-campus support
- [ ] Peer-to-peer messaging
- [ ] Carbon credit tracking
- [ ] Integration with utility grid

### Long Term (12+ months)
- [ ] Machine learning price predictions
- [ ] Automated demand response
- [ ] Battery storage integration
- [ ] Cross-campus energy trading

## Related Documentation

### Architecture Details
- [Blockchain Architecture](../blockchain/BLOCKCHAIN_GUIDE.md)
- [PoA Governance](../blockchain/POA_GOVERNANCE_SETUP.md)
- [Anchor Programs](../blockchain/ANCHOR_PROGRAMS_ARCHITECTURE.md)

### Diagrams
- [System Context (C4)](../../diagrams/component/C4_LEVEL_1_SYSTEM_CONTEXT.puml)
- [Container Diagram](../../diagrams/component/C4_LEVEL_2_CONTAINERS.puml)
- [Architecture Overview](../../diagrams/sequence/ARCHITECTURE_OVERVIEW_SEQUENCE.puml)

### Guides
- [Setup Guide](../../guides/setup/INITIALIZATION_SETUP.md)
- [Testing Guide](../../guides/testing/RUN_ANCHOR_TESTS.md)

### Reference
- [Data Dictionary](../../reference/data-models/DATA_DICTIONARY.md)
- [Process Specifications](../../specifications/processes/PROCESS_SPECIFICATIONS.md)

## Glossary

- **AMI**: Advanced Metering Infrastructure
- **ATA**: Associated Token Account (Solana)
- **kWh**: Kilowatt-hour (unit of energy)
- **PDA**: Program Derived Address (Solana)
- **PoA**: Proof of Authority (consensus mechanism)
- **Prosumer**: Producer + Consumer (generates and consumes energy)
- **REC**: Renewable Energy Center
- **SPL**: Solana Program Library

## Contact & Support

- **Documentation**: `/docs/technical/`
- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions
- **Team**: GridTokenX Development Team

---

**Document Version**: 1.0  
**Last Updated**: 2025-11-08  
**Status**: Active  
**Maintained By**: System Architecture Team
