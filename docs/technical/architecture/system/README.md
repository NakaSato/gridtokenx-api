# 🏗️ GridTokenX Architecture Documentation

**Comprehensive Technical Architecture Guide**

---

## 📋 Documentation Overview

This documentation provides a complete technical architecture guide for the GridTokenX platform's Anchor smart contracts. The documentation is organized into focused sections for better navigation and maintenance.

---

## 📚 Documentation Structure

### 🎯 Core Architecture
- **[System Architecture](./01-system-architecture.md)** - High-level system design, program dependencies, and architectural overview
- **[Code Structure](./02-code-structure.md)** - Detailed breakdown of each program's internal structure and components
- **[Smart Meter Simulator](./SMART_METER_SIMULATOR.md)** - Advanced AMI simulation system for P2P energy trading with real-time data generation

### 🔄 Process Flows
- **[Data Flow Diagrams](./03-data-flow-diagrams.md)** - Step-by-step data flow for all program functions
- **[Transaction Flows](./04-transaction-flows.md)** - Complete transaction sequences and business processes
- **[Call Sequence Diagrams](./06-call-sequence-diagrams.md)** - Inter-program communication patterns and function calls

### 🔐 Security & Access
- **[Authorization Matrix](./05-authorization-matrix.md)** - Complete access control and permission matrix
- **[Account Relationships](./07-account-relationships.md)** - Account dependencies and PDA relationships

### 📘 Anchor Smart Contracts (Comprehensive)
- **[Anchor Documentation Hub](../anchor/README.md)** - Complete Anchor blockchain documentation
  - Architecture Overview - 85KB comprehensive technical reference
  - PlantUML Diagrams - 9 professional diagrams
  - Mermaid Diagrams - GitHub-compatible visuals
  - Quick Reference - Developer cheat sheet
  - Complete Index - Role-based navigation

---

## 🚀 Quick Start

### For Developers
1. Start with [System Architecture](./01-system-architecture.md) for the big picture
2. Review [Code Structure](./02-code-structure.md) to understand program organization
3. Deep dive into [Anchor Documentation](../anchor/README.md) for complete smart contract details
4. Use [Data Flow Diagrams](./03-data-flow-diagrams.md) to understand individual functions
5. Explore [Smart Meter Simulator](./SMART_METER_SIMULATOR.md) for IoT device simulation

### For Integration
1. Check [Authorization Matrix](./05-authorization-matrix.md) for access requirements
2. Follow [Call Sequence Diagrams](./06-call-sequence-diagrams.md) for integration patterns
3. Reference [Transaction Flows](./04-transaction-flows.md) for complete workflows
4. Review [Anchor Quick Reference](../anchor/ANCHOR_QUICK_REFERENCE.md) for instruction signatures

### For Security Review
1. Review [Authorization Matrix](./05-authorization-matrix.md) for access controls
2. Analyze [Account Relationships](./07-account-relationships.md) for security implications
3. Examine [Call Sequence Diagrams](./06-call-sequence-diagrams.md) for potential attack vectors
4. Study [Anchor Security Model](../anchor/ANCHOR_ARCHITECTURE_OVERVIEW.md#security-model) for comprehensive security analysis

### For Visual Learners
1. View [Anchor PlantUML Diagrams](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml) - 9 professional diagrams
2. View [Anchor Mermaid Diagrams](../anchor/ANCHOR_ARCHITECTURE_MERMAID.md) - GitHub-compatible visuals
3. Explore [System Architecture](./01-system-architecture.md) ASCII diagrams

---

## 🏛️ System Overview

### Programs
- **Registry Program** (`Bxvy5Y...`) - User and meter registration and validation
- **Energy Token Program** (`6LgvcJ...`) - GRID token minting and SPL token integration
- **Oracle Program** (`2Jqh9J...`) - AMI data ingestion and market clearing triggers
- **Trading Program** (`Hzmt59...`) - Peer-to-peer energy trading marketplace
- **Governance Program** (`83V1DX...`) - ERC issuance and emergency controls

> **📘 For detailed program IDs and complete technical specs, see [Anchor Documentation](../anchor/README.md)**

### Key Features
- ✅ **Proof of Authority (PoA)** governance model
- ✅ **Emergency pause/unpause** system-wide controls
- ✅ **Energy Renewable Certificates (ERC)** issuance and validation
- ✅ **SPL Token integration** for energy tokenization (GRID tokens)
- ✅ **Automated market matching** with fee collection
- ✅ **AMI meter integration** via API Gateway
- ✅ **Comprehensive access controls** and validation
- ✅ **Double-spend prevention** via settled_net_generation tracking
- ✅ **Double-claim prevention** via claimed_erc_generation tracking
- ✅ **Cross-Program Invocations (CPI)** for secure inter-program communication

---

## 🔧 Technical Specifications

### Blockchain
- **Platform**: Solana
- **Framework**: Anchor 0.32.1
- **Consensus**: Proof of Authority (PoA)
- **Token Standard**: SPL Token (GRID)

### Programs (Localnet)
- **Registry**: `Bxvy5YBKoADe1BSTnj4cd117RLzfjUKG2WEk2iqcmVJE`
- **Energy Token**: `6LgvcJ8pxzSbzWCdaTWB2gUg4WazJv46eSjzj6LCNjNd`
- **Oracle**: `2Jqh9JkxpJuWyqdzSDv3gskgMN9fT4K73P88a6xYAy4i`
- **Trading**: `Hzmt59DnHUKa8h8MJADgAf4zjREhvwZXW5ew5gnTnFPH`
- **Governance**: `83V1DXgURKYkPURCJbBKU3VzkqVjYcPKDuL6DRLKAGvw`

> **📘 For mainnet/testnet deployment addresses and configuration, see [Anchor Documentation](../anchor/ANCHOR_QUICK_REFERENCE.md#program-ids)**

---

## 📊 Documentation Statistics

### Architecture Documentation
- **Total Documents**: 8 core documents + comprehensive Anchor docs
- **Total Coverage**: System architecture, code structure, flows, security, IoT simulation, and complete blockchain layer
- **Last Updated**: November 9, 2025

### Anchor Documentation
- **Total Size**: ~155 KB of documentation
- **Total Diagrams**: 18 diagrams (9 PlantUML + 9 Mermaid)
- **Programs Documented**: 5 programs (Registry, Energy Token, Oracle, Trading, Governance)
- **Instructions Covered**: 27+ smart contract functions
- **Error Codes**: 34+ documented error codes
- **Account Structures**: 10+ PDA schemas

---

## 📖 Document Maintenance

### Last Updated
- **Version**: 2.0
- **Date**: November 7, 2025
- **Anchor Framework**: 0.32.1
- **Major Update**: Added comprehensive Anchor blockchain documentation

### Contributing
When updating this documentation:
1. Keep each section focused and self-contained
2. Update cross-references when adding new functions
3. Maintain consistent diagram formatting
4. Update the main README when adding new sections
5. Cross-reference with [Anchor Documentation](../anchor/) for blockchain details

### Related Documentation
- **[Anchor Smart Contracts](../anchor/README.md)** - Complete blockchain layer documentation
- **[Master Documentation Guide](../MASTER_DOCUMENTATION_GUIDE.md)** - Platform-wide documentation index
- **[System Analysis](../README_SYSTEM_ANALYSIS.md)** - System analysis and planning

---

## 📞 Support

For technical questions about this architecture:
- **Engineering Department**: engineering_erc@utcc.ac.th
- **Repository**: [GridTokenX Platform](https://github.com/NakaSato/gridtokenx-platform)

---

**GridTokenX Platform - Sustainable Energy Trading Ecosystem**