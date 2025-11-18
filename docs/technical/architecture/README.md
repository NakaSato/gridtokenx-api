# Architecture Documentation

## Overview

This directory contains comprehensive architectural documentation for the GridTokenX platform, organized by system layer.

## Structure

### 📁 system/
Overall system architecture and design patterns
- System context and boundaries
- High-level component interactions
- Technology stack decisions
- Architecture decision records (ADRs)

### 📁 blockchain/
Solana blockchain architecture
- Anchor program designs
- Account structures
- Instruction flows
- Security model
- PoA governance

### 📁 frontend/
React frontend architecture
- Component hierarchy
- State management (React Query)
- Routing structure
- UI/UX patterns
- Wallet integration

### 📁 backend/
Rust API Gateway architecture
- Service layers
- Authentication/Authorization
- Database design
- Caching strategy
- RPC client integration

## Key Documents

### System Level
- `system/OVERVIEW.md` - Complete system overview
- `system/TECH_STACK.md` - Technology choices and rationale
- `system/DEPLOYMENT.md` - Deployment architecture

### Blockchain Level
- `blockchain/ANCHOR_PROGRAMS.md` - All Anchor programs
- `blockchain/ACCOUNT_MODEL.md` - Account structure
- `blockchain/POA_GOVERNANCE.md` - PoA setup

### Application Level
- `frontend/COMPONENT_ARCHITECTURE.md` - React structure
- `backend/SERVICE_ARCHITECTURE.md` - API Gateway design

## For LLMs

### Context Loading Priority
```
1. system/OVERVIEW.md          # Start here
2. blockchain/ANCHOR_PROGRAMS.md
3. backend/SERVICE_ARCHITECTURE.md
4. frontend/COMPONENT_ARCHITECTURE.md
```

### Related Directories
- Diagrams: `/technical/diagrams/component/`
- Specifications: `/technical/specifications/requirements/`
- Reference: `/technical/reference/`

---
**Category**: Architecture  
**Last Updated**: 2025-11-08
