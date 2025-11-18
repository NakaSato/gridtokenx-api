# 📚 GridTokenX Master Documentation

**Version**: 1.0  
**Project**: GridTokenX P2P Energy Trading System  
**Last Updated**: January 1, 2025  
**Status**: ✅ Complete

---

## 📖 Table of Contents

1. [Project Overview](#project-overview)
2. [System Architecture](#system-architecture)
3. [Technology Stack](#technology-stack)
4. [Project Structure](#project-structure)
5. [Key Components](#key-components)
6. [4-Step Trading Cycle](#4-step-trading-cycle)
7. [Development Setup](#development-setup)
8. [API Reference](#api-reference)
9. [Database Schema](#database-schema)
10. [Deployment](#deployment)
11. [Quick Reference](#quick-reference)

---

## Project Overview

### What is GridTokenX?

GridTokenX is a **peer-to-peer energy trading platform** that enables students, faculty, and staff within an engineering complex to trade renewable energy on a blockchain-based platform.

**Key Features**:
- ✅ Decentralized energy trading between prosumers and consumers
- ✅ Automated market clearing every 15 minutes
- ✅ REC (Renewable Energy Certificate) tokens on Solana blockchain
- ✅ Real-time energy data from smart meters
- ✅ Proof-of-Authority (PoA) governance model
- ✅ Role-based access control (Student, Faculty, Grid Operator, Admin)

### Use Cases

| User Type | Goal | Benefit |
|-----------|------|---------|
| **Prosumer (Student/Faculty)** | Generate excess energy, sell to others | Earn GRID tokens, reduce energy costs |
| **Consumer (Student/Faculty)** | Buy renewable energy | Support sustainable energy, lower costs |
| **Grid Operator** | Manage meters, system operations | Monitor energy flow, maintain grid |
| **Engineering Authority** | System administration, governance | Control parameters, emergency shutdown |

### Business Model

```
Energy Generation (Solar Panels, etc.)
        ↓
        ├─→ Mint REC Tokens (Energy Token Program)
        │
Users Place Orders (Trading Program)
        ↓
        ├─→ Automatic Market Clearing (Oracle Program)
        │
Settlement (Buyer/Seller Exchange)
        ↓
        └─→ Immutable Record on Blockchain
```

---

## System Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    GridTokenX Platform                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────┐                                   │
│  │   React Frontend     │                                   │
│  │  (User Dashboard)    │                                   │
│  └──────────┬───────────┘                                   │
│             │ HTTPS                                         │
│  ┌──────────▼───────────┐                                   │
│  │   API Gateway        │  23 REST Endpoints               │
│  │  (Rust/Actix-Web)    │  JWT Authentication              │
│  └──────────┬───────────┘                                   │
│             │ JSON-RPC                                      │
│  ┌──────────▼────────────────────────────────────────┐      │
│  │    Solana Blockchain (PoA Validator)              │      │
│  │  ┌──────────────┬─────────────┬─────────────┐    │      │
│  │  │ Registry Prg │ Token Prg   │ Trading Prg │    │      │
│  │  ├──────────────┼─────────────┼─────────────┤    │      │
│  │  │ Governance   │ Oracle Prg  │             │    │      │
│  │  └──────────────┴─────────────┴─────────────┘    │      │
│  └──────────┬───────────────────────────────────────┘      │
│             │                                               │
│  ┌──────────▼───────────────────┐                          │
│  │    Data Layer                │                          │
│  │  ├─ PostgreSQL (Relational)  │                          │
│  │  ├─ TimescaleDB (Time-Series)│                          │
│  │  └─ Redis (Cache)            │                          │
│  └──────────────────────────────┘                          │
│                                                              │
│  ┌──────────────────────┐                                   │
│  │ Smart Meter Simulator│  AMI Data Source                 │
│  │   (Python)           │  (Development)                   │
│  └──────────────────────┘                                   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### C4 Architecture Model

**Level 1: System Context**
- Identifies all users and external systems
- 4 user types: Consumer, Prosumer, Grid Operator, Admin
- 3 external systems: Solana, AMI Backend, Oracle Data

**Level 2: Containers**
- Frontend (React 18 + Vite)
- Backend (Rust + Actix-Web)
- Blockchain (5 Anchor Programs on Solana)
- Data Stores (PostgreSQL, TimescaleDB, Redis)

**Level 3: Components**
- **Frontend**: Feature modules (Registry, Trading, Governance, Meter, Account)
- **Blockchain**: 5 Anchor programs with specific responsibilities
- **Backend**: Services layer with business logic, repositories, external APIs

---

## Technology Stack

### Frontend
| Technology | Purpose |
|------------|---------|
| **React 18** | UI Framework |
| **TypeScript** | Type-safe JavaScript |
| **Vite** | Build tool & dev server |
| **React Query** | Data fetching & caching |
| **Tailwind CSS** | Styling |
| **Solana Web3.js** | Blockchain interaction |

### Backend
| Technology | Purpose |
|------------|---------|
| **Rust** | Systems programming language |
| **Actix-Web** | HTTP server framework |
| **Tokio** | Async runtime |
| **Sqlx** | Database access |
| **JWT** | Authentication tokens |
| **Solana RPC** | Blockchain communication |

### Blockchain
| Technology | Purpose |
|------------|---------|
| **Solana** | Layer 1 blockchain (PoA) |
| **Anchor** | Smart contract framework |
| **SPL Token** | Token standard |
| **Metaplex** | NFT/metadata standard |

### Data Storage
| Technology | Purpose |
|------------|---------|
| **PostgreSQL** | Relational data (users, orders, config) |
| **TimescaleDB** | Time-series data (meter readings) |
| **Redis** | In-memory cache |

---

## Project Structure

```
gridtokenx-app/
├── frontend/                        # React Web Application
│   ├── src/
│   │   ├── features/               # Feature-based modules
│   │   │   ├── registry/           # User registration
│   │   │   ├── trading/            # P2P trading UI
│   │   │   ├── governance/         # Voting interface
│   │   │   ├── meter/              # Energy readings
│   │   │   └── account/            # User profile
│   │   ├── components/             # Shared components
│   │   ├── hooks/                  # Custom React hooks
│   │   ├── services/               # API calls
│   │   ├── store/                  # State management
│   │   └── App.jsx                 # Root component
│   ├── package.json
│   └── vite.config.js
│
├── api-gateway/                     # Backend API Server (Rust)
│   ├── src/
│   │   ├── main.rs                 # Application entry point
│   │   ├── handlers/               # HTTP request handlers
│   │   ├── services/               # Business logic
│   │   ├── models/                 # Data structures
│   │   ├── database/               # DB queries & migrations
│   │   ├── auth/                   # Authentication logic
│   │   ├── middleware/             # HTTP middleware
│   │   └── utils/                  # Utility functions
│   ├── migrations/                 # Database migrations (SQL)
│   ├── Cargo.toml                  # Rust dependencies
│   └── tests/                      # Integration tests
│
├── anchor/                          # Solana Smart Contracts
│   ├── programs/
│   │   ├── energy-token/           # REC token management
│   │   ├── registry/               # User & meter registration
│   │   ├── trading/                # Order book & matching
│   │   ├── governance/             # PoA voting system
│   │   └── oracle/                 # Data validation
│   ├── src/
│   │   ├── index.ts                # Exports
│   │   ├── client/                 # Generated clients
│   │   └── helpers/                # Utility functions
│   ├── tests/                      # Anchor tests
│   ├── Cargo.toml
│   ├── Anchor.toml
│   └── codama.js                   # Client generation config
│
├── docker/                          # Docker Configuration
│   ├── frontend/                   # Frontend Docker build
│   ├── api-gateway/                # Backend Docker build
│   ├── solana-validator/           # Local Solana node
│   ├── postgres/                   # PostgreSQL container
│   ├── timescaledb/                # TimescaleDB container
│   ├── redis/                      # Redis container
│   ├── smart-meter-simulator/      # Meter simulator
│   └── docker-compose.yml          # Multi-container setup
│
├── docs/                            # Comprehensive Documentation
│   ├── 01-c4-model/                # Architecture diagrams
│   ├── 02-data-flow-diagrams/      # Data flow visualization
│   ├── 03-architecture-guides/     # Implementation details
│   ├── 04-planning-reference/      # Context & references
│   └── 05-index-navigation/        # Navigation helpers
│
├── scripts/                         # Utility scripts
├── package.json                     # Monorepo config
├── tsconfig.json                    # TypeScript config
├── Makefile                         # Build automation
├── docker-compose.yml               # Production setup
└── README.md                        # Project readme
```

---

## Key Components

### 1. Frontend (React Application)

**Location**: `frontend/`

**Features**:
- User authentication via Solana wallet
- Dashboard showing energy balance and trading activity
- Order placement interface (Buy/Sell)
- Real-time price updates
- Market clearing results display
- User profile management

**Feature Modules**:
```
Registry:  Manage user registration, roles
Trading:   Create & view orders, trade history
Governance: View proposals, voting interface
Meter:     Display energy generation/consumption
Account:   User profile, settings, wallet info
```

### 2. API Gateway (Backend)

**Location**: `api-gateway/`

**Endpoints**: 23 REST endpoints
```
Authentication:
  POST /auth/login          - Wallet-based login
  POST /auth/logout         - User logout
  GET  /auth/verify         - Verify JWT token

Users:
  POST   /users/register    - Register new user
  GET    /users/:id         - Get user profile
  PUT    /users/:id         - Update profile
  DELETE /users/:id         - Deactivate account

Trading:
  POST   /orders/create     - Place new order
  GET    /orders            - List user orders
  GET    /orders/:id        - Order details
  DELETE /orders/:id        - Cancel order
  GET    /market/price      - Current market price

Meters:
  GET  /meters             - List assigned meters
  POST /meters/:id/reading - Record meter reading
  GET  /meters/:id/history - Historical readings

Governance:
  GET  /governance/proposals    - List proposals
  POST /governance/vote         - Submit vote
  GET  /governance/parameters   - System parameters

Admin:
  POST /admin/assign-meter     - Assign meter to user
  PUT  /admin/update-parameter - Update system settings
```

### 3. Blockchain (Solana Anchor Programs)

**Location**: `anchor/programs/`

**5 Anchor Programs**:

| Program | Purpose | Key Functions |
|---------|---------|---------------|
| **registry** | User & meter management | `register_user()`, `assign_meter()`, `verify_user()` |
| **energy-token** | SPL token for REC | `mint_to()`, `transfer()`, `burn()` |
| **trading** | Order book & matching | `create_order()`, `match_orders()`, `settle_trade()` |
| **oracle** | Market clearing & data validation | `trigger_market_clearing()`, `submit_meter_data()` |
| **governance** | PoA voting system | `create_proposal()`, `vote()`, `execute_proposal()` |

### 4. Database

**PostgreSQL Tables**:
```
users              - User accounts and profiles
user_meters        - Meter assignments
api_keys           - API key management
trading_orders     - Order history
blockchain_transactions - TX history
user_activities    - Activity log
```

**TimescaleDB Hypertables**:
```
energy_readings    - Smart meter data (15-min intervals)
market_prices      - Historical prices
system_metrics     - Performance metrics
```

---

## 4-Step Trading Cycle

### Step 1: Registration & Meter Assignment ⏱️ One-time

1. User registers with Solana wallet
2. API validates wallet signature
3. Registry Program creates user account PDA on-chain
4. Engineering Department assigns smart meters
5. User ready for energy trading

### Step 2: Energy Generation ⏱️ Continuous

1. Smart meter collects energy generation data (AMI readings)
2. Every 15 minutes: Oracle processes readings
3. Tokens minted: 1 kWh = 1,000,000,000 ENG_GRID tokens
4. Tokens deposited to user's Associated Token Account (ATA)
5. Readings stored in TimescaleDB for analytics

### Step 3: Order Placement ⏱️ 0:00-0:15 (15-min window)

1. **Sellers**: Create sell orders ("Sell 2.5 kWh @ $0.05/kWh")
2. **Buyers**: Create buy orders ("Buy 2.5 kWh @ $0.05/kWh")
3. Orders stored in PostgreSQL (order book)
4. Orders stored on-chain (immutable record)
5. Order book accumulates during 15-min window

### Step 4: Automated Market Clearing ⏱️ At :15, :30, :45, :00

1. Epoch timer triggers market close
2. Order matching algorithm runs:
   - Sort sell orders ascending by price
   - Sort buy orders descending by price
   - Match at equilibrium price
3. Settlement execution:
   - SPL token transfer (seller → buyer)
   - Update balances on blockchain
   - Record settlement in PostgreSQL
4. New epoch begins

---

## Development Setup

### Prerequisites

```bash
# Required versions
Node.js: 18+
Rust: 1.70+
Solana CLI: 1.18+
Docker: 20.10+
```

### Quick Start

```bash
# 1. Clone repository
git clone https://github.com/NakaSato/gridtokenx-platform.git
cd gridtokenx-app

# 2. Install dependencies
pnpm install
cd frontend && pnpm install
cd ../api-gateway && cargo build

# 3. Setup Anchor
pnpm run setup

# 4. Generate clients
pnpm run codama:js

# 5. Start development environment
docker-compose -f docker-compose.yml up

# 6. Run frontend
cd frontend && pnpm run dev

# 7. Run backend (in new terminal)
cd api-gateway && cargo run

# 8. Run tests
cd anchor && anchor test
```

### Environment Variables

**Frontend** (`frontend/.env.local`):
```
VITE_RPC_ENDPOINT=http://localhost:8899
VITE_API_GATEWAY=http://localhost:8080
VITE_CLUSTER=localnet
```

**Backend** (`api-gateway/.env`):
```
DATABASE_URL=postgresql://user:password@localhost:5432/gridtokenx
SOLANA_RPC_URL=http://localhost:8899
JWT_SECRET=your-secret-key
```

---

## API Reference

### Authentication

**POST /auth/login**
```json
Request:
{
  "wallet": "8sDXFsq8RDxSv5jBWwD5TaHEQgK1L8Dqzw1R2L4M3N5",
  "signature": "base64-encoded-signature",
  "message": "Sign this message"
}

Response: 200 OK
{
  "token": "eyJhbGciOiJIUzI1NiIs...",
  "user_id": "user-123",
  "user_type": "student"
}
```

### Trading API

**POST /orders/create**
```json
Request:
{
  "order_type": "sell",
  "energy_amount": 2.5,
  "price_per_unit": 0.05,
  "expiry": "2025-01-01T00:15:00Z"
}

Response: 201 Created
{
  "order_id": "SELL_001",
  "status": "pending",
  "created_at": "2025-01-01T00:05:00Z"
}
```

**GET /market/price**
```json
Response: 200 OK
{
  "current_price": 0.0523,
  "24h_high": 0.0650,
  "24h_low": 0.0420,
  "volume_24h": 1250.5,
  "last_updated": "2025-01-01T14:32:00Z"
}
```

---

## Database Schema

### Users Table
```sql
CREATE TABLE users (
  id UUID PRIMARY KEY,
  wallet_address VARCHAR(88) UNIQUE NOT NULL,
  email VARCHAR(255),
  user_type ENUM('student', 'faculty', 'operator', 'admin'),
  registered_at TIMESTAMP DEFAULT NOW(),
  active BOOLEAN DEFAULT TRUE
);
```

### Trading Orders Table
```sql
CREATE TABLE trading_orders (
  id UUID PRIMARY KEY,
  user_id UUID REFERENCES users(id),
  order_type ENUM('buy', 'sell'),
  energy_amount DECIMAL(18, 9),
  price_per_unit DECIMAL(18, 9),
  status ENUM('pending', 'matched', 'settled', 'cancelled'),
  created_at TIMESTAMP DEFAULT NOW(),
  settled_at TIMESTAMP
);
```

### Energy Readings (TimescaleDB)
```sql
CREATE TABLE energy_readings (
  time TIMESTAMP NOT NULL,
  meter_id VARCHAR(20),
  user_id UUID REFERENCES users(id),
  kwh_generated DECIMAL(18, 9),
  kwh_consumed DECIMAL(18, 9),
  PRIMARY KEY (time, meter_id)
);

SELECT create_hypertable('energy_readings', 'time', 
  if_not_exists => TRUE);
```

---

## Deployment

### Docker Deployment

```bash
# Build and run all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# Clean volumes
docker-compose down -v
```

### Production Deployment

```bash
# 1. Set environment variables
export SOLANA_CLUSTER=mainnet-beta
export SOLANA_RPC_URL=https://api.mainnet-beta.solana.com

# 2. Deploy smart contracts
cd anchor
anchor deploy --provider.cluster mainnet

# 3. Deploy frontend
cd ../frontend
pnpm run build
# Upload dist/ to web hosting

# 4. Deploy backend
cd ../api-gateway
cargo build --release
# Deploy binary to server
```

---

## Quick Reference

### Common Commands

```bash
# Development
pnpm run dev              # Start dev server
pnpm run build            # Production build
pnpm run lint             # Check code quality
pnpm run format           # Format code

# Anchor
pnpm run anchor-build     # Build smart contracts
pnpm run anchor-test      # Run contract tests
pnpm run anchor-localnet  # Run local validator

# Code Generation
pnpm run codama:js        # Generate TypeScript clients

# Database
cargo sqlx migrate run    # Run migrations
```

### Key Files to Know

| File | Purpose |
|------|---------|
| `frontend/src/App.jsx` | React root component |
| `api-gateway/src/main.rs` | Backend entry point |
| `anchor/Anchor.toml` | Anchor project config |
| `docker-compose.yml` | Container orchestration |
| `package.json` | Monorepo configuration |

### Important URLs

| Service | URL |
|---------|-----|
| Frontend | http://localhost:5173 |
| API Gateway | http://localhost:8080 |
| Solana RPC | http://localhost:8899 |
| PostgreSQL | localhost:5432 |
| TimescaleDB | localhost:5433 |
| Redis | localhost:6379 |

---

## Documentation Structure

**For complete documentation**, see:

- **Architecture Diagrams**: `docs/01-c4-model/`
- **Data Flow Diagrams**: `docs/02-data-flow-diagrams/`
  - Including STEP_1-4 for complete trading cycle
- **Implementation Guides**: `docs/03-architecture-guides/`
- **Planning Reference**: `docs/04-planning-reference/`
- **Navigation Hub**: `docs/05-index-navigation/`

---

## Support & Resources

### Documentation
- C4 Architecture: `docs/01-c4-model/C4_MODEL_INDEX.md`
- Trading Cycle: `docs/02-data-flow-diagrams/STEPS_1_2_3_4_COMPLETE_FLOW.md`
- API Details: `docs/03-architecture-guides/API_GATEWAY_ARCHITECTURE.md`

### Development
- Smart Contracts: `anchor/programs/*/src/lib.rs`
- Backend Services: `api-gateway/src/services/`
- React Features: `frontend/src/features/`

### Community
- GitHub: https://github.com/NakaSato/gridtokenx-platform
- Issues: Report bugs and request features
- Discussions: Ask questions and share ideas

---

**End of Master Documentation**

📄 **Print/Export**: This file can be saved as PDF (A4 format fits ~8-10 pages)  
📋 **Version Control**: Kept under version control with code  
🔄 **Status**: Updated as project evolves

