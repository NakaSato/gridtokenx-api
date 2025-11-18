# Anchor Blockchain Architecture Documentation

This directory contains comprehensive documentation for the GridTokenX Anchor programs (Solana blockchain smart contracts).

## 📚 Documentation Files

### 1. **ANCHOR_ARCHITECTURE_OVERVIEW.md** - Main Documentation
Complete technical documentation covering:
- System architecture overview
- All five Anchor programs in detail
- Account structures and PDAs
- Data flow diagrams (text-based)
- Security model
- Integration patterns
- Deployment instructions

**Start here** for a comprehensive understanding of the blockchain layer.

### 2. **ANCHOR_ARCHITECTURE_DIAGRAMS.puml** - Visual Diagrams
PlantUML diagrams including:
- High-level architecture overview
- Program relationships and data flow
- Account structure diagrams
- Sequence diagrams:
  - User & meter registration
  - Energy generation & token minting
  - P2P trading workflow
  - ERC certification flow
- CPI (Cross-Program Invocation) patterns
- Security model visualization

**Use PlantUML** to render these diagrams in VS Code, IntelliJ, or online at http://www.plantuml.com/plantuml/

### 3. **ANCHOR_QUICK_REFERENCE.md** - Developer Quick Reference
Quick reference guide with:
- Program IDs
- Build and deploy commands
- Account structures (sizes and fields)
- Instruction reference (all functions)
- Error codes
- Common patterns (double-spend prevention, CPI, PDA signing)
- Testing commands

**Use this** for day-to-day development work.

---

## 🚀 Quick Start

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana
sh -c "$(curl -sSfL https://release.solana.com/v1.18.0/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.32.1
avm use 0.32.1
```

### Build & Deploy
```bash
# Navigate to anchor directory
cd anchor

# Build all programs
anchor build

# Start local validator
solana-test-validator

# Deploy programs (in another terminal)
anchor deploy

# Run tests
anchor test
```

---

## 📋 Five Core Programs

| Program | ID | Purpose | Documentation Section |
|---------|----|---------|-----------------------|
| **Registry** | `Bxvy5Y...` | User & meter registration | [Overview](ANCHOR_ARCHITECTURE_OVERVIEW.md#1-registry-program) |
| **Energy Token** | `6LgvcJ...` | GRID token management | [Overview](ANCHOR_ARCHITECTURE_OVERVIEW.md#2-energy-token-program) |
| **Oracle** | `2Jqh9J...` | External data ingestion | [Overview](ANCHOR_ARCHITECTURE_OVERVIEW.md#3-oracle-program) |
| **Trading** | `Hzmt59...` | P2P marketplace | [Overview](ANCHOR_ARCHITECTURE_OVERVIEW.md#4-trading-program) |
| **Governance** | `83V1DX...` | PoA & ERC certification | [Overview](ANCHOR_ARCHITECTURE_OVERVIEW.md#5-governance-program) |

---

## 🔐 Security Features

### Double-Spend Prevention
- **Token Minting:** `settled_net_generation` field prevents double-minting
- **ERC Certification:** `claimed_erc_generation` field prevents double-claiming
- **Trading Escrow:** Atomic settlement prevents double-spending in trades

### Authority Model
- **System Admin:** Deploy, initialize programs
- **REC Authority:** Issue ERCs, emergency controls
- **API Gateway:** Submit oracle data (only authorized entity)
- **Users:** Self-register, manage own assets

### Cross-Program Security
- PDA-based account creation
- Signer validation on all CPIs
- Account ownership verification
- Event emission for audit trails

---

## 📊 Key Data Flows

### 1. Energy Generation → Token Minting
```
Smart Meter → AMI → API Gateway → Oracle Program → Registry Program
                                                          ↓
User Request → API Gateway → Energy Token Program → Registry (CPI)
                                    ↓
                            SPL Token Program (Mint)
```

### 2. P2P Trading
```
Prosumer → Create Sell Order → Lock tokens in escrow
Consumer → Create Buy Order → Lock tokens in escrow
                    ↓
        Matching Engine → Match Orders
                    ↓
        Atomic Settlement (transfer tokens)
                    ↓
        Create TradeRecord (immutable)
```

### 3. ERC Certification
```
Prosumer → Request ERC → REC Authority
                              ↓
                    Governance Program
                              ↓
                    Verify energy availability
                              ↓
                    Create ErcCertificate
                              ↓
                    Update claimed_erc_generation
```

---

## 🧪 Testing

### Run All Tests
```bash
anchor test
```

### Run Specific Tests
```bash
# Registry tests
anchor test tests/registry.ts

# Energy token tests
anchor test tests/energy-token.ts

# Oracle tests
anchor test tests/oracle.ts

# Trading tests
anchor test tests/trading.ts

# Governance tests
anchor test tests/governance.ts
```

### Integration Tests
```bash
# Run full integration test suite
cd anchor
npm run test:integration

# Or use the provided script
./scripts/run-tests.sh
```

---

## 📖 Architecture Diagrams

### Viewing PlantUML Diagrams

#### Option 1: VS Code (Recommended)
1. Install "PlantUML" extension
2. Open `ANCHOR_ARCHITECTURE_DIAGRAMS.puml`
3. Press `Alt + D` to preview

#### Option 2: IntelliJ IDEA
1. Install "PlantUML Integration" plugin
2. Open `.puml` file
3. Diagrams render automatically

#### Option 3: Online
1. Copy diagram code from `.puml` file
2. Visit http://www.plantuml.com/plantuml/
3. Paste and render

#### Option 4: Command Line
```bash
# Install PlantUML
brew install plantuml  # macOS
apt install plantuml   # Ubuntu

# Generate PNG
plantuml ANCHOR_ARCHITECTURE_DIAGRAMS.puml

# Generate SVG
plantuml -tsvg ANCHOR_ARCHITECTURE_DIAGRAMS.puml
```

### Available Diagrams

1. **ANCHOR_ARCHITECTURE_OVERVIEW** - High-level system context
2. **ANCHOR_PROGRAM_RELATIONSHIPS** - Program interactions
3. **ANCHOR_ACCOUNT_STRUCTURE** - All account types with relationships
4. **ANCHOR_SEQUENCE_USER_REGISTRATION** - Registration flow
5. **ANCHOR_SEQUENCE_TOKEN_MINTING** - Minting flow
6. **ANCHOR_SEQUENCE_P2P_TRADING** - Trading flow
7. **ANCHOR_SEQUENCE_ERC_CERTIFICATION** - ERC issuance flow
8. **ANCHOR_CPI_PATTERNS** - Cross-program invocation patterns
9. **ANCHOR_SECURITY_MODEL** - Multi-layer security architecture

---

## 🔧 Development Workflow

### 1. Make Changes
```bash
# Edit program code
vim anchor/programs/registry/src/lib.rs

# Build
anchor build
```

### 2. Test Changes
```bash
# Run relevant tests
anchor test tests/registry.ts

# Or run all tests
anchor test
```

### 3. Deploy to Localnet
```bash
# Start validator (if not running)
solana-test-validator

# Deploy
anchor deploy

# Or deploy specific program
anchor deploy --program-name registry
```

### 4. Verify Deployment
```bash
# Check program account
solana program show <PROGRAM_ID>

# Check recent transactions
solana transaction-history <PROGRAM_ID>
```

---

## 🐛 Common Issues & Solutions

### Issue: "Program not found"
```bash
# Solution: Re-deploy the program
anchor deploy --program-name <program-name>
```

### Issue: "Account already exists"
```bash
# Solution: Close existing accounts or use different PDAs
anchor test --skip-deploy
```

### Issue: "Insufficient SOL for transaction"
```bash
# Solution: Airdrop SOL to wallet
solana airdrop 10 <WALLET_ADDRESS>
```

### Issue: "Anchor build fails"
```bash
# Solution: Clean and rebuild
anchor clean
anchor build
```

### Issue: "CPI error: incorrect program ID"
```bash
# Solution: Verify program IDs in lib.rs match deployed programs
anchor keys list
```

---

## 📚 Additional Resources

### Official Documentation
- [Anchor Framework](https://www.anchor-lang.com/)
- [Solana Docs](https://docs.solana.com/)
- [SPL Token Program](https://spl.solana.com/token)

### GridTokenX Documentation
- [System Overview](../SYSTEM_OVERVIEW_SIMPLIFIED.md)
- [API Gateway Integration](../API_GATEWAY_BLOCKCHAIN_INTERACTION.md)
- [Data Flow Diagrams](../02-data-flow-diagrams/)
- [C4 Model](../01-c4-model/)

### Testing Resources
- [Anchor Tests Documentation](../../anchor/tests/README.md)
- [Test Suite Documentation](../../anchor/TEST_SUITE_DOCUMENTATION.md)

---

## 🤝 Contributing

When adding new features to Anchor programs:

1. **Update Program Code:**
   - Add instructions to `lib.rs`
   - Add state structs if needed
   - Add events for state changes

2. **Update Tests:**
   - Add unit tests
   - Add integration tests
   - Verify error conditions

3. **Update Documentation:**
   - Update `ANCHOR_ARCHITECTURE_OVERVIEW.md`
   - Add diagrams to `ANCHOR_ARCHITECTURE_DIAGRAMS.puml`
   - Update `ANCHOR_QUICK_REFERENCE.md`

4. **Security Review:**
   - Check authority validation
   - Verify PDA seeds
   - Review CPI security
   - Test double-spend prevention

---

## 📞 Support

For questions or issues:
- Check the [Quick Reference](ANCHOR_QUICK_REFERENCE.md)
- Review the [Architecture Overview](ANCHOR_ARCHITECTURE_OVERVIEW.md)
- Refer to [Test Documentation](../../anchor/tests/README.md)

---

**Last Updated:** November 7, 2025  
**Version:** 1.0  
**Maintained by:** GridTokenX Team
