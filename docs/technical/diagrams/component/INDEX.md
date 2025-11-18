# Component Diagrams Index

## Overview

C4 Model component diagrams showing the structure of the GridTokenX system at different levels of detail.

## C4 Model Levels

### Level 1: System Context

#### C4_LEVEL_1_SYSTEM_CONTEXT.puml
**Description**: Highest level view showing the system and its external actors  
**Shows**:
- Campus Users (Prosumers/Consumers)
- Grid Operator (REC)
- GridTokenX Platform
- External Systems

**Use Case**: Understanding system boundaries and external interactions  
**Audience**: Stakeholders, architects, LLMs

---

### Level 2: Container Diagrams

#### C4_LEVEL_2_CONTAINERS.puml
**Description**: High-level technology choices and containers  
**Shows**:
- Frontend (React + Vite)
- API Gateway (Rust/Axum)
- Solana Blockchain (PoA)
- PostgreSQL Database
- TimescaleDB
- Redis Cache
- Smart Meter Simulator

**Use Case**: Understanding major system components and their relationships  
**Audience**: Developers, architects, DevOps

---

### Level 3: Component Details

#### C4_LEVEL_3_COMPONENTS_FRONTEND.puml
**Description**: React frontend internal components  
**Shows**:
- React Components
- React Query hooks
- Wallet integration (Gill SDK)
- State management
- Routing

**Use Case**: Frontend development and architecture understanding  
**Audience**: Frontend developers

#### C4_LEVEL_3_COMPONENTS_BACKEND.puml
**Description**: Rust API Gateway internal components  
**Shows**:
- REST endpoints
- JWT authentication
- Business logic layer
- RPC client
- Database connections
- Cache management

**Use Case**: Backend development and API design  
**Audience**: Backend developers

#### C4_LEVEL_3_COMPONENTS_ANCHOR.puml
**Description**: Solana blockchain programs and structure  
**Shows**:
- Registry Program
- Energy Token Program
- Trading Program
- Oracle Program
- Governance Program
- Account structures

**Use Case**: Smart contract development and blockchain integration  
**Audience**: Blockchain developers, Anchor programmers

## C4 Model Conventions

### Diagram Hierarchy
```
Level 1: System Context    (Who uses the system?)
    ↓
Level 2: Containers        (What are the major parts?)
    ↓
Level 3: Components        (How is each part built?)
    ↓
Level 4: Code             (Implementation details - not in repo)
```

### Reading Order
1. Start with Level 1 for system overview
2. Move to Level 2 for technical architecture
3. Dive into Level 3 for specific subsystem details

## Usage

### For LLMs
```bash
# Load in hierarchical order
cat C4_LEVEL_1_SYSTEM_CONTEXT.puml
cat C4_LEVEL_2_CONTAINERS.puml

# Then load specific subsystems
cat C4_LEVEL_3_COMPONENTS_FRONTEND.puml
cat C4_LEVEL_3_COMPONENTS_BACKEND.puml
cat C4_LEVEL_3_COMPONENTS_ANCHOR.puml
```

### For Developers

#### Understanding the System
```bash
# New to project
1. Read C4_LEVEL_1 - See the big picture
2. Read C4_LEVEL_2 - Understand tech stack
3. Read relevant Level 3 - Your area of work
```

#### Frontend Development
```bash
# Focus on
1. C4_LEVEL_2_CONTAINERS.puml - How frontend fits
2. C4_LEVEL_3_COMPONENTS_FRONTEND.puml - Internal structure
```

#### Backend Development
```bash
# Focus on
1. C4_LEVEL_2_CONTAINERS.puml - API Gateway role
2. C4_LEVEL_3_COMPONENTS_BACKEND.puml - Service layers
```

#### Blockchain Development
```bash
# Focus on
1. C4_LEVEL_2_CONTAINERS.puml - Blockchain integration
2. C4_LEVEL_3_COMPONENTS_ANCHOR.puml - Programs structure
```

## Diagram Files

| File | Level | Focus | Status |
|------|-------|-------|--------|
| C4_LEVEL_1_SYSTEM_CONTEXT.puml | 1 | System boundaries | ✅ Migrated |
| C4_LEVEL_2_CONTAINERS.puml | 2 | Tech stack | ✅ Migrated |
| C4_LEVEL_3_COMPONENTS_FRONTEND.puml | 3 | React app | ✅ Migrated |
| C4_LEVEL_3_COMPONENTS_BACKEND.puml | 3 | API Gateway | ✅ Migrated |
| C4_LEVEL_3_COMPONENTS_ANCHOR.puml | 3 | Blockchain | ✅ Migrated |

## Related Documentation

- **Architecture Docs**: `/technical/architecture/`
- **Sequence Diagrams**: `/technical/diagrams/sequence/`
- **System Architecture**: `/technical/architecture/system/SYSTEM_ARCHITECTURE.md`

## C4 Model Resources

- **Official Site**: https://c4model.com
- **PlantUML for C4**: https://github.com/plantuml-stdlib/C4-PlantUML
- **Best Practices**: https://c4model.com/#coreDiagrams

---
**Category**: Diagrams > Component  
**Count**: 5 diagrams  
**Status**: ✅ Migration Complete
