# Anchor Blockchain Documentation - Complete Index

**GridTokenX Platform - Solana Anchor Programs**  
**Version:** 1.0  
**Last Updated:** November 7, 2025

---

## 📚 Documentation Overview

This directory contains complete documentation for the GridTokenX Anchor blockchain layer, consisting of five interconnected smart contracts (programs) on Solana.

### Documentation Files

| File | Type | Size | Purpose |
|------|------|------|---------|
| **[README.md](README.md)** | Guide | ~8 KB | Start here - Overview and navigation |
| **[ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md)** | Technical Doc | ~85 KB | Comprehensive technical documentation |
| **[ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml)** | PlantUML | ~20 KB | Professional diagrams (9 diagrams) |
| **[ANCHOR_ARCHITECTURE_MERMAID.md](ANCHOR_ARCHITECTURE_MERMAID.md)** | Mermaid Diagrams | ~12 KB | GitHub-compatible diagrams |
| **[ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md)** | Quick Ref | ~25 KB | Developer cheat sheet |
| **[INDEX.md](INDEX.md)** | Index | ~5 KB | This file - Complete navigation |

---

## 🎯 Quick Navigation

### By Role

#### 👨‍💼 Project Manager / Business Analyst
**Goal:** Understand what the blockchain layer does

1. Start: [README.md](README.md) - Quick overview
2. Read: [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md) - Introduction and Architecture Overview sections
3. View: [ANCHOR_ARCHITECTURE_MERMAID.md](ANCHOR_ARCHITECTURE_MERMAID.md) - Visual diagrams

**Time Required:** 30 minutes

#### 👨‍💻 Blockchain Developer
**Goal:** Understand architecture and start coding

1. Start: [README.md](README.md) - Setup instructions
2. Study: [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md) - Full document
3. Reference: [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md) - Keep open while coding
4. View: [ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml) - Deep dive diagrams

**Time Required:** 2-3 hours

#### 👨‍🔬 Backend Developer (API Gateway)
**Goal:** Understand how to integrate with blockchain

1. Read: [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md) - Integration Patterns section
2. View: [ANCHOR_ARCHITECTURE_MERMAID.md](ANCHOR_ARCHITECTURE_MERMAID.md) - Sequence diagrams
3. Reference: [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md) - Instruction reference

**Time Required:** 1 hour

#### 🔒 Security Auditor
**Goal:** Understand security model and identify vulnerabilities

1. Read: [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md) - Security Model section
2. View: [ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml) - Security diagrams
3. Study: [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md) - Common Patterns section

**Time Required:** 3-4 hours

#### 📖 Technical Writer
**Goal:** Update or maintain documentation

1. Review: All files in order
2. Update: Follow contributing guidelines in [README.md](README.md)

**Time Required:** Full day

---

## 📖 By Topic

### Architecture & Design

#### System Overview
- **File:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md)
- **Sections:**
  - Introduction
  - Architecture Overview
  - System Context diagram

#### Program Architecture
- **File:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md)
- **Sections:**
  - 1. Registry Program (User & meter management)
  - 2. Energy Token Program (GRID token lifecycle)
  - 3. Oracle Program (External data bridge)
  - 4. Trading Program (P2P marketplace)
  - 5. Governance Program (PoA & ERC certification)

#### Visual Architecture
- **PlantUML:** [ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml)
  - ANCHOR_ARCHITECTURE_OVERVIEW
  - ANCHOR_PROGRAM_RELATIONSHIPS
  - ANCHOR_ACCOUNT_STRUCTURE
- **Mermaid:** [ANCHOR_ARCHITECTURE_MERMAID.md](ANCHOR_ARCHITECTURE_MERMAID.md)
  - High-Level Architecture
  - Program Relationships
  - Account Structure

---

### Data Flows

#### User Registration
- **File:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#user-registration-flow)
- **Diagram:** [ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml) - ANCHOR_SEQUENCE_USER_REGISTRATION
- **Mermaid:** N/A

#### Energy Generation & Token Minting
- **File:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#energy-generation--tokenization-flow)
- **Diagram:** [ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml) - ANCHOR_SEQUENCE_TOKEN_MINTING
- **Mermaid:** [ANCHOR_ARCHITECTURE_MERMAID.md](ANCHOR_ARCHITECTURE_MERMAID.md#token-minting-flow)

#### P2P Trading
- **File:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#p2p-trading-flow)
- **Diagram:** [ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml) - ANCHOR_SEQUENCE_P2P_TRADING
- **Mermaid:** [ANCHOR_ARCHITECTURE_MERMAID.md](ANCHOR_ARCHITECTURE_MERMAID.md#trading-flow)

#### ERC Certification
- **File:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#erc-certification-flow)
- **Diagram:** [ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml) - ANCHOR_SEQUENCE_ERC_CERTIFICATION
- **Mermaid:** [ANCHOR_ARCHITECTURE_MERMAID.md](ANCHOR_ARCHITECTURE_MERMAID.md#erc-certification-flow)

---

### Security

#### Security Model Overview
- **File:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#security-model)
- **Diagram:** [ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml) - ANCHOR_SECURITY_MODEL
- **Mermaid:** [ANCHOR_ARCHITECTURE_MERMAID.md](ANCHOR_ARCHITECTURE_MERMAID.md#security-layers)

#### Double-Spend Prevention
- **File:** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#1-double-spend-prevention)
- **Details:**
  - Token Minting: `settled_net_generation` tracker
  - ERC Certification: `claimed_erc_generation` tracker
  - Trading Escrow: Atomic settlement

#### Authority Model
- **File:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#authority-hierarchy)
- **Diagram:** [ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml) - Authority section in ANCHOR_SECURITY_MODEL

#### Cross-Program Invocation Security
- **File:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#cross-program-invocation-cpi-security)
- **Diagram:** [ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml) - ANCHOR_CPI_PATTERNS
- **Mermaid:** [ANCHOR_ARCHITECTURE_MERMAID.md](ANCHOR_ARCHITECTURE_MERMAID.md#cpi-pattern-energy-token--registry)
- **Quick Ref:** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#2-cross-program-invocation-cpi)

---

### Development

#### Setup & Installation
- **File:** [README.md](README.md#quick-start)
- **Quick Ref:** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#quick-start-commands)

#### Build & Deploy
- **File:** [README.md](README.md#build--deploy)
- **Quick Ref:** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#build--deploy)
- **Deployment:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#deployment)

#### Testing
- **File:** [README.md](README.md#testing)
- **Quick Ref:** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#testing-commands)

#### Common Patterns
- **File:** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#common-patterns)
- **Topics:**
  1. Double-Spend Prevention
  2. Cross-Program Invocation (CPI)
  3. PDA Signing
  4. Authority Validation
  5. Event Emission
  6. State Validation

---

### Reference

#### Program IDs
- **Quick Ref:** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#program-ids)
- **Overview:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#program-ids-localnet)

#### Account Structures
- **Quick Ref:** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#account-structures)
- **Overview:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md) - Each program section
- **Diagram:** [ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml) - ANCHOR_ACCOUNT_STRUCTURE
- **Mermaid:** [ANCHOR_ARCHITECTURE_MERMAID.md](ANCHOR_ARCHITECTURE_MERMAID.md#account-structure)

#### Instructions Reference
- **Quick Ref:** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#instruction-reference)
- **Overview:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md) - Each program section's "Key Instructions"

#### Error Codes
- **Quick Ref:** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#error-codes)
- **Overview:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#error-codes)

---

## 🔗 Cross-References

### Related Documentation

#### System-Wide Documentation
- **Main Docs:** [../README.md](../README.md)
- **System Overview:** [../SYSTEM_OVERVIEW_SIMPLIFIED.md](../SYSTEM_OVERVIEW_SIMPLIFIED.md)
- **Master Guide:** [../MASTER_DOCUMENTATION.md](../MASTER_DOCUMENTATION.md)

#### Architecture Documentation
- **C4 Model:** [../01-c4-model/README_C4_MODEL.md](../01-c4-model/README_C4_MODEL.md)
- **C4 Level 3 (Anchor):** [../01-c4-model/C4_LEVEL_3_COMPONENTS_ANCHOR.puml](../01-c4-model/C4_LEVEL_3_COMPONENTS_ANCHOR.puml)
- **Data Flow Diagrams:** [../02-data-flow-diagrams/README_DIAGRAMS.md](../02-data-flow-diagrams/README_DIAGRAMS.md)

#### API Gateway Integration
- **API Gateway Doc:** [../API_GATEWAY_BLOCKCHAIN_INTERACTION.md](../API_GATEWAY_BLOCKCHAIN_INTERACTION.md)
- **Integration Diagram:** [../API_GATEWAY_BLOCKCHAIN_INTERACTION.puml](../API_GATEWAY_BLOCKCHAIN_INTERACTION.puml)

#### Testing Documentation
- **Test Suite:** [../../anchor/tests/README.md](../../anchor/tests/README.md)
- **Test Documentation:** [../../anchor/TEST_SUITE_DOCUMENTATION.md](../../anchor/TEST_SUITE_DOCUMENTATION.md)
- **Integration Tests:** [../../anchor/RUN_INTEGRATION_TEST.md](../../anchor/RUN_INTEGRATION_TEST.md)

#### Implementation Guides
- **Token Minting:** [../../anchor/TOKEN_MINTING_IMPLEMENTATION.md](../../anchor/TOKEN_MINTING_IMPLEMENTATION.md)
- **Tokenization Status:** [../../anchor/TOKENIZATION_IMPLEMENTATION_STATUS.md](../../anchor/TOKENIZATION_IMPLEMENTATION_STATUS.md)
- **Quick Start Minting:** [../../anchor/QUICK_START_MINTING.md](../../anchor/QUICK_START_MINTING.md)

---

## 📊 Diagram Matrix

| Diagram Name | PlantUML | Mermaid | Topic |
|--------------|----------|---------|-------|
| High-Level Architecture | ✅ ANCHOR_ARCHITECTURE_OVERVIEW | ✅ Yes | System overview |
| Program Relationships | ✅ ANCHOR_PROGRAM_RELATIONSHIPS | ✅ Yes | Program interactions |
| Account Structure | ✅ ANCHOR_ACCOUNT_STRUCTURE | ✅ Yes | Data models |
| User Registration | ✅ ANCHOR_SEQUENCE_USER_REGISTRATION | ❌ No | Registration flow |
| Token Minting | ✅ ANCHOR_SEQUENCE_TOKEN_MINTING | ✅ Yes | Minting flow |
| P2P Trading | ✅ ANCHOR_SEQUENCE_P2P_TRADING | ✅ Yes | Trading flow |
| ERC Certification | ✅ ANCHOR_SEQUENCE_ERC_CERTIFICATION | ✅ Yes | ERC flow |
| CPI Patterns | ✅ ANCHOR_CPI_PATTERNS | ✅ Partial | Cross-program calls |
| Security Model | ✅ ANCHOR_SECURITY_MODEL | ✅ Yes | Security layers |
| Emergency Controls | ❌ No | ✅ Yes | State machine |

**Legend:**
- ✅ = Full diagram available
- ✅ Partial = Simplified version available
- ❌ No = Not available in this format

---

## 🎓 Learning Paths

### Path 1: Quick Overview (30 minutes)
1. Read: [README.md](README.md)
2. View: [ANCHOR_ARCHITECTURE_MERMAID.md](ANCHOR_ARCHITECTURE_MERMAID.md) - High-Level Architecture
3. Skim: [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md) - Program IDs and Account Structures

### Path 2: Developer Onboarding (4 hours)
1. Read: [README.md](README.md) - Full read
2. Study: [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md) - Introduction through Program Architecture
3. View: All diagrams in [ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml)
4. Practice: Follow [README.md](README.md#quick-start) setup
5. Reference: Keep [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md) open

### Path 3: Security Audit (1 day)
1. Study: [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md) - Full document
2. Deep dive: Security Model section
3. Analyze: [ANCHOR_ARCHITECTURE_DIAGRAMS.puml](ANCHOR_ARCHITECTURE_DIAGRAMS.puml) - ANCHOR_SECURITY_MODEL
4. Review: [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md) - Common Patterns
5. Test: Follow error code scenarios
6. Verify: Cross-program invocation security

### Path 4: Integration Development (2 hours)
1. Read: [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md) - Integration Patterns section
2. Study: [ANCHOR_ARCHITECTURE_MERMAID.md](ANCHOR_ARCHITECTURE_MERMAID.md) - All sequence diagrams
3. Reference: [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md) - Instruction Reference
4. Review: [../API_GATEWAY_BLOCKCHAIN_INTERACTION.md](../API_GATEWAY_BLOCKCHAIN_INTERACTION.md)

### Path 5: Complete Understanding (2-3 days)
1. Day 1: Read all documentation files sequentially
2. Day 2: Study all diagrams, cross-reference with code
3. Day 3: Build, deploy, and test locally

---

## 🔍 Search by Keyword

### Program Names
- **Registry:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#1-registry-program)
- **Energy Token:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#2-energy-token-program)
- **Oracle:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#3-oracle-program)
- **Trading:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#4-trading-program)
- **Governance:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#5-governance-program)

### Key Concepts
- **PDA (Program Derived Address):** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#pda-security)
- **CPI (Cross-Program Invocation):** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#2-cross-program-invocation-cpi)
- **Double-Spend Prevention:** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#1-double-spend-prevention)
- **Authority:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#authority-hierarchy)
- **ERC (Energy Renewable Certificate):** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#erc-certification-flow)
- **GRID Token:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#2-energy-token-program)
- **Escrow:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#4-trading-program)

### Technical Terms
- **SPL Token:** [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#2-energy-token-program)
- **Solana:** [README.md](README.md)
- **Anchor Framework:** [README.md](README.md)
- **Event Emission:** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#5-event-emission)
- **Signer Seeds:** [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#3-pda-signing)

---

## 📝 Document Maintenance

### Last Updated
- **All Files:** November 7, 2025
- **Version:** 1.0

### Update Frequency
- **Code changes:** Update documentation immediately
- **New features:** Add to all relevant sections
- **Bug fixes:** Update affected sections
- **Quarterly:** Review all documentation for accuracy

### Contributing
See [README.md](README.md#contributing) for contribution guidelines.

---

## 🆘 Help & Support

### Common Questions

**Q: Where do I start?**  
A: Read [README.md](README.md) first, then follow the learning path that matches your role.

**Q: How do I view PlantUML diagrams?**  
A: See [README.md](README.md#viewing-plantuml-diagrams) for multiple viewing options.

**Q: What's the difference between PlantUML and Mermaid diagrams?**  
A: PlantUML diagrams are more detailed and professional. Mermaid diagrams render directly on GitHub and are simpler.

**Q: How do I know which program to call?**  
A: See the [Program Relationships](#program-relationships) section and review sequence diagrams.

**Q: Where are the program IDs?**  
A: [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#program-ids)

**Q: How do I prevent double-spending?**  
A: Read [ANCHOR_QUICK_REFERENCE.md](ANCHOR_QUICK_REFERENCE.md#1-double-spend-prevention)

**Q: What are PDAs?**  
A: Read [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#pda-security)

**Q: How do I integrate with the API Gateway?**  
A: Read [ANCHOR_ARCHITECTURE_OVERVIEW.md](ANCHOR_ARCHITECTURE_OVERVIEW.md#integration-patterns)

### Getting Help
- Check this index first
- Review the [Quick Reference](ANCHOR_QUICK_REFERENCE.md)
- Search the [Architecture Overview](ANCHOR_ARCHITECTURE_OVERVIEW.md)
- Review relevant diagrams

---

**End of Index**
