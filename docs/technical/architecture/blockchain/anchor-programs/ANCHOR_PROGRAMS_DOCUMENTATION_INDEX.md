# 📖 Anchor Programs Documentation - Complete Index

**GridTokenX Platform - Smart Contracts Documentation Set**

---

## 📚 Documentation Overview

This comprehensive documentation set covers all 5 Anchor programs in the GridTokenX platform. Choose the document that best fits your needs:

---

## 1. **ANCHOR_PROGRAMS_DETAILED_GUIDE.md** 📘
**For: Complete Technical Understanding**

The most comprehensive guide covering all programs in detail.

### Contents:
- ✅ Complete program descriptions
- ✅ All functions with parameters and return values
- ✅ Data structures explained
- ✅ Event types and triggers
- ✅ Error codes and meanings
- ✅ Security considerations
- ✅ Cross-program interactions
- ✅ Attack prevention patterns

### Best For:
- Developers building integrations
- Auditors reviewing contracts
- New team members learning the system
- Feature development and extensions
- Security analysis

### Reading Time: 45-60 minutes

---

## 2. **ANCHOR_PROGRAMS_QUICK_REF.md** ⚡
**For: Fast Lookups and Development**

Quick reference with tables and checklists for developers.

### Contents:
- ✅ Program IDs and locations
- ✅ Function tables (access level, parameters, purpose)
- ✅ Enum definitions
- ✅ Error code list
- ✅ PDA seed formulas
- ✅ Event types summary
- ✅ Default configuration values
- ✅ Common error patterns
- ✅ Testing patterns
- ✅ Pre-call checklists

### Best For:
- Developers writing code
- Quick function lookups
- Integration developers
- Quick debugging
- Testing checklists

### Reading Time: 5-10 minutes

---

## 3. **ANCHOR_PROGRAMS_ARCHITECTURE.md** 🏗️
**For: System Architecture and Design**

Visual diagrams and flow charts for understanding system design.

### Contents:
- ✅ High-level system architecture
- ✅ Data flow diagrams (6 different flows)
- ✅ Program structure (code organization)
- ✅ Account relationships and dependencies
- ✅ Complete transaction flows with state changes
- ✅ Authorization matrix
- ✅ Call sequence diagrams

### Best For:
- Understanding system design
- Visualizing data flows
- Learning account relationships
- Authorization verification
- Transaction tracing
- Architecture decisions

### Reading Time: 30-45 minutes

---

## Program Descriptions (Quick Reference)

### 🔮 Oracle Program
**Program ID:** `ApwexmUbEZMpez5dJXKza4V7gqSqWvAA9BPbok2psxXg`

**Role:** Data input layer for meter readings and market clearing

**Key Functions:** (5 total)
- `initialize` - Setup oracle
- `submit_meter_reading` - Accept AMI data
- `trigger_market_clearing` - Trigger settlement
- `update_oracle_status` - Enable/disable
- `update_api_gateway` - Change data source

**Access Pattern:** API Gateway for submissions, Authority for admin

---

### 🏛️ Governance Program
**Program ID:** `Dy8JFn95L1E7NoUkXbFQtW1kGR7Ja21CkNcirNgv4ghe`

**Role:** Proof-of-Authority governance for ERC issuance

**Key Functions:** (10 total)
- `initialize_poa` - Setup governance
- `issue_erc` - Create certificate
- `validate_erc_for_trading` - Approve trading
- `emergency_pause/unpause` - System controls
- `update_erc_limits` - Adjust parameters
- `get_governance_stats` - Statistics

**Access Pattern:** Authority only (Engineering Department)

---

### 📋 Registry Program
**Program ID:** `42LoRKPphBBdvaCDx2ZjNuZFqzXuJziiiNXyiV6FhBY5`

**Role:** User and smart meter registration

**Key Functions:** (8 total)
- `initialize` - Setup registry
- `register_user` - New user registration
- `register_meter` - Meter registration
- `update_meter_reading` - Record readings
- `update_user_status` - Manage users
- `is_valid_user/meter` - Verification

**Access Pattern:** Public registration, Authority for status updates

---

### ⚡ Energy-Token Program
**Program ID:** `2CVWTnckn5TXUWXdZoZE6LydiQJGMYHVVPipkoy1LVqr`

**Role:** Native energy token management

**Key Functions:** (5 total)
- `initialize/initialize_token` - Setup token
- `add_rec_validator` - Add validators
- `transfer_tokens` - User transfers
- `burn_tokens` - Consumption

**Access Pattern:** SPL Token Program integration via CPI

---

### 🛒 Trading Program
**Program ID:** `dS3zvp95PFVrNNBfZDXn78QL5MvhUqDCFR4rn8z9Jgh`

**Role:** P2P energy marketplace

**Key Functions:** (7 total)
- `initialize_market` - Setup market
- `create_sell_order` - List energy
- `create_buy_order` - Request energy
- `match_orders` - Execute trade
- `cancel_order` - Withdraw order
- `update_market_params` - Config

**Access Pattern:** Users for orders, Authority for admin functions

---

## Documentation Navigation by Role

### 👨‍💻 Developer / Integrator
1. Start with: **ANCHOR_PROGRAMS_QUICK_REF.md** (5 min)
2. Deep dive: **ANCHOR_PROGRAMS_DETAILED_GUIDE.md** (45 min)
3. Reference: **ANCHOR_PROGRAMS_QUICK_REF.md** while coding

### 🔍 Auditor / Security Reviewer
1. Start with: **ANCHOR_PROGRAMS_ARCHITECTURE.md** (30 min)
2. Review: Security section in **ANCHOR_PROGRAMS_DETAILED_GUIDE.md** (15 min)
3. Verify: Authorization matrix in **ANCHOR_PROGRAMS_ARCHITECTURE.md** (10 min)

### 👥 Project Manager / Stakeholder
1. Start with: **ANCHOR_PROGRAMS_ARCHITECTURE.md** (20 min)
2. Reference: Program descriptions in **ANCHOR_PROGRAMS_QUICK_REF.md** (5 min)

### 🎓 New Team Member
1. Start with: **ANCHOR_PROGRAMS_ARCHITECTURE.md** (30 min) - Understand flows
2. Deep dive: **ANCHOR_PROGRAMS_DETAILED_GUIDE.md** (60 min) - Learn details
3. Practice: Use **ANCHOR_PROGRAMS_QUICK_REF.md** (ongoing)

### 🐛 Debugger / Troubleshooter
1. Start with: **ANCHOR_PROGRAMS_QUICK_REF.md** (5 min) - Find error code
2. Reference: Relevant function in **ANCHOR_PROGRAMS_DETAILED_GUIDE.md** (5 min)
3. Check: Authorization matrix in **ANCHOR_PROGRAMS_ARCHITECTURE.md** (5 min)

---

## Key Concepts Explained

### Program-Derived Accounts (PDAs)
Deterministic account addresses derived from seeds. Used to prevent account spoofing.

Example: User account can't accidentally use another user's account because the PDA includes their pubkey in the seed.

### Cross-Program Invocations (CPIs)
Programs calling other programs to perform actions.

Example: Trading program calls Energy-Token program to transfer tokens.

### Events
Off-chain notifications of state changes. Indexed for quick lookup and audit trails.

Example: When an order is created, `SellOrderCreated` event is emitted.

### Signers
Cryptographic proof that a user authorized an action. Cannot be forged without private key.

Example: User must sign to create a trading order.

### Escrowing
Temporarily holding tokens or energy in a program account until trade completes.

Example: When buyer creates order, tokens are held in escrow until matched.

---

## Common Workflows

### Workflow 1: Register and Trade Energy

```
1. User calls Registry.register_user()
   └─> Creates UserAccount

2. User calls Registry.register_meter()
   └─> Creates MeterAccount

3. Energy is generated (physical)

4. Oracle submits readings
   └─> Via Oracle.submit_meter_reading()

5. Authority issues ERC
   └─> Via Governance.issue_erc()

6. Authority validates ERC
   └─> Via Governance.validate_erc_for_trading()

7. Authority transfers tokens
   └─> Via Energy-Token.transfer_tokens()

8. Seller creates sell order
   └─> Via Trading.create_sell_order()

9. Buyer creates buy order
   └─> Via Trading.create_buy_order()

10. Authority matches orders
    └─> Via Trading.match_orders()

11. Trade settled ✓
    └─> Tokens transferred, fee collected
```

### Workflow 2: System Emergency

```
1. Security issue detected

2. Authority calls Governance.emergency_pause()
   └─> System frozen
   └─> No new transactions accepted
   └─> Existing state preserved

3. Issue resolved

4. Authority calls Governance.emergency_unpause()
   └─> System resumes normal operation
```

### Workflow 3: Market Clearing

```
1. Oracle triggers Governance.trigger_market_clearing()

2. System processes all pending matches

3. Balances updated

4. Trades settled

5. Next trading period begins
```

---

## File Structure in Workspace

```
gridtokenx-platform/
├── anchor/
│   ├── programs/
│   │   ├── oracle/src/lib.rs              # 225 lines
│   │   ├── governance/src/lib.rs          # 599 lines
│   │   ├── registry/src/lib.rs            # 391 lines
│   │   ├── energy-token/src/lib.rs        # 164 lines
│   │   └── trading/src/lib.rs             # 364 lines
│   │
│   └── Cargo.toml                         # Workspace config
│
├── ANCHOR_PROGRAMS_DETAILED_GUIDE.md      # THIS SET
├── ANCHOR_PROGRAMS_QUICK_REF.md           #
├── ANCHOR_PROGRAMS_ARCHITECTURE.md        #
│
└── ... (other documentation files)
```

---

## Statistics

### Code Metrics
```
Total Programs: 5
Total Functions: 37
Total Data Structs: 13
Total Error Types: 45
Total Event Types: 22
Lines of Documentation: 3500+
```

### Access Patterns
```
Authority-only functions: 15
Public functions: 8
User-callable functions: 14
```

### Account Types
```
PDA Accounts: 9
Regular Accounts: Multiple SPL Token accounts
Program-owned: All Anchor program accounts
```

---

## External References

### Useful Links

- **Anchor Framework:** https://www.anchor-lang.com/
- **Solana Documentation:** https://docs.solana.com/
- **SPL Token Program:** https://github.com/solana-labs/solana-program-library
- **Rust Language:** https://www.rust-lang.org/

### Key Concepts

- **PDA (Program Derived Account):** Deterministic account address
- **CPI (Cross-Program Invocation):** One program calling another
- **Signer:** Account that authorized a transaction
- **Lamports:** Smallest unit of SOL (1 SOL = 1,000,000,000 lamports)

---

## Quick Answers

**Q: How do I send a meter reading?**
A: Call `Oracle.submit_meter_reading()` with meter_id, energy_produced, energy_consumed, timestamp. Only API Gateway can call this.

**Q: How do I trade energy?**
A: 1) Register user/meter, 2) Seller creates sell order, 3) Buyer creates buy order, 4) Authority matches orders.

**Q: What if something goes wrong?**
A: Authority can call `Governance.emergency_pause()` to freeze the system.

**Q: How are fees collected?**
A: Trading program keeps 0.25% (25 basis points) of every trade.

**Q: Can I trade if I haven't been registered?**
A: No. You must first call `Registry.register_user()` and `Registry.register_meter()`.

**Q: What's an ERC?**
A: Energy Renewable Certificate - proof that energy was generated from renewable source. Must be validated before trading.

**Q: Who can issue ERCs?**
A: Only the Engineering Department (Authority) can issue and validate ERCs.

---

## Document Versions

| Document | Version | Date | Status |
|----------|---------|------|--------|
| ANCHOR_PROGRAMS_DETAILED_GUIDE.md | 1.0 | Nov 1, 2025 | ✅ Current |
| ANCHOR_PROGRAMS_QUICK_REF.md | 1.0 | Nov 1, 2025 | ✅ Current |
| ANCHOR_PROGRAMS_ARCHITECTURE.md | 1.0 | Nov 1, 2025 | ✅ Current |
| ANCHOR_PROGRAMS_DOCUMENTATION_INDEX.md | 1.0 | Nov 1, 2025 | ✅ Current |

---

## Support & Questions

For questions about:
- **Function behavior:** See ANCHOR_PROGRAMS_DETAILED_GUIDE.md
- **Quick reference:** See ANCHOR_PROGRAMS_QUICK_REF.md
- **System design:** See ANCHOR_PROGRAMS_ARCHITECTURE.md
- **Code location:** Check `anchor/programs/{program}/src/lib.rs`

---

## Maintenance Notes

### When to Update Documentation

1. After new functions are added
2. After error types change
3. After PDA seed structure changes
4. After access control changes
5. After event structure changes

### How to Update Documentation

1. Modify source code in `anchor/programs/{program}/src/lib.rs`
2. Update corresponding documentation section
3. Update quick reference tables
4. Update architecture diagrams if flow changed
5. Rebuild and test

---

**Documentation Set Complete ✅**

**Total Pages:** 3  
**Total Content:** 3500+ lines  
**Code Coverage:** 100% (all 5 programs documented)  
**Last Updated:** November 1, 2025  
**Framework Version:** Anchor 0.32.1

---

### Navigation

- **← Previous:** Build & Test Reports
- **Home:** /Users/chanthawat/Developments/gridtokenx-platform/
- **Next:** Implementation Guide (TBD)

---

**For latest updates, see:** README.md in project root
