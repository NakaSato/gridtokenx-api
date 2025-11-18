# 🔄 Transaction Flows

**GridTokenX Platform - Complete Business Process Workflows**

> **📘 For detailed sequence diagrams with complete workflows, see:**
> - [Token Minting Flow](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml#ANCHOR_SEQUENCE_TOKEN_MINTING)
> - [P2P Trading Flow](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml#ANCHOR_SEQUENCE_P2P_TRADING)
> - [ERC Certification Flow](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml#ANCHOR_SEQUENCE_ERC_CERTIFICATION)
> - [User Registration Flow](../anchor/ANCHOR_ARCHITECTURE_DIAGRAMS.puml#ANCHOR_SEQUENCE_USER_REGISTRATION)
> 
> **Note**: This document provides high-level workflows. For technical implementation details, refer to the Anchor documentation.

---

## Table of Contents

1. [End-to-End Business Flows](#end-to-end-business-flows)
2. [Multi-Program Transaction Sequences](#multi-program-transaction-sequences)
3. [Error Handling and Recovery Flows](#error-handling-and-recovery-flows)
4. [Cross-Program Communication Patterns](#cross-program-communication-patterns)
5. [State Transition Workflows](#state-transition-workflows)

---

## End-to-End Business Flows

### Complete Energy Trading Workflow

**From Energy Generation to Token Sale**

```
Step 1-5: Energy Generation and Oracle Processing
┌─────────────────────────────────────────────┐
│ Physical World:                             │
│ ├─ Solar panel generates 100 kWh            │
│ ├─ AMI meter detects generation             │
│ └─ Data transmitted to API Gateway          │
│                                             │
│ Oracle.submit_meter_reading(                │
│   meter_id: "METER_001",                    │
│   reading: 100,                             │
│   timestamp: current_time                   │
│ )                                           │
│                                             │
│ Results:                                    │
│ ├─ OracleData.total_readings += 1           │
│ ├─ Reading validated and processed          │
│ └─ Event: MeterReadingSubmitted             │
└─────────────────────────────────────────────┘
         │
         ▼

Step 6: Registry Update
┌─────────────────────────────────────────────┐
│ Registry.update_meter_reading(              │
│   meter_id: "METER_001",                    │
│   production: 100,                          │
│   timestamp: current_time                   │
│ )                                           │
│                                             │
│ Updates:                                    │
│ ├─ MeterAccount.total_energy_produced += 100│
│ ├─ UserAccount.total_energy_produced += 100 │
│ └─ Event: MeterReadingUpdated               │
│                                             │
│ Cross-Program Call:                         │
│ └─ Triggers Governance.issue_erc()          │
└─────────────────────────────────────────────┘
         │
         ▼

Step 7: ERC Issuance
┌─────────────────────────────────────────────┐
│ Governance.issue_erc(                       │
│   certificate_id: "ERC_2024_001",           │
│   energy_amount: 100,                       │
│   renewable_source: "Solar",                │
│   authority: governance_authority            │
│ )                                           │
│                                             │
│ Creates:                                    │
│ ├─ ErcCertificate PDA                       │
│ │  ├─ certificate_id: "ERC_2024_001"        │
│ │  ├─ energy_amount: 100                    │
│ │  ├─ status: Valid                         │
│ │  └─ expires_at: current_time + 1_year     │
│ └─ Event: ErcIssued                         │
└─────────────────────────────────────────────┘
         │
         ▼

Step 8: ERC Validation for Trading
┌─────────────────────────────────────────────┐
│ Governance.validate_erc_for_trading(        │
│   certificate_id: "ERC_2024_001"            │
│ )                                           │
│                                             │
│ Updates:                                    │
│ ├─ ErcCertificate.validated_for_trading = true│
│ ├─ ErcCertificate.trading_validated_at = now │
│ └─ Event: ErcValidatedForTrading            │
│                                             │
│ Cross-Program Call:                         │
│ └─ Triggers EnergyToken.transfer_tokens()   │
└─────────────────────────────────────────────┘
         │
         ▼

Step 9: Token Issuance
┌─────────────────────────────────────────────┐
│ EnergyToken.transfer_tokens(                │
│   amount: 100,                              │
│   recipient: user_pubkey                    │
│ )                                           │
│                                             │
│ SPL Token Operations:                       │
│ ├─ Create user TokenAccount if needed       │
│ ├─ Mint 100 energy tokens to user           │
│ └─ TokenInfo.total_supply += 100            │
│                                             │
│ Result:                                     │
│ ├─ User now has 100 tradeable tokens        │
│ └─ Event: TokensIssued                      │
└─────────────────────────────────────────────┘
         │
         ▼

Step 10: Create Sell Order
┌─────────────────────────────────────────────┐
│ UserA (Producer):                           │
│ - Has 100 energy tokens                     │
│ - Wants to sell 50 kWh @ 10 tokens/kWh     │
│                                             │
│ Trading.create_sell_order(                  │
│   order_id: "SELL_001",                     │
│   amount: 50,                               │
│   price_per_kwh: 10                         │
│ )                                           │
│                                             │
│ Process:                                    │
│ ├─ Validate user has 50 tokens              │
│ ├─ Transfer 50 tokens to escrow             │
│ ├─ Create Order PDA                         │
│ │  ├─ seller: UserA                         │
│ │  ├─ amount: 50                            │
│ │  ├─ status: Active                        │
│ │  └─ escrow_amount: 50                     │
│ └─ Event: SellOrderCreated                  │
│                                             │
│ UserA Balance: 50 tokens (100 - 50 escrow) │
└─────────────────────────────────────────────┘
         │
         ▼

Step 11: Create Buy Order
┌─────────────────────────────────────────────┐
│ UserB (Consumer):                           │
│ - Has 1000 tokens                           │
│ - Wants 50 kWh @ max 10 tokens/kWh         │
│                                             │
│ Trading.create_buy_order(                   │
│   order_id: "BUY_001",                      │
│   amount: 50,                               │
│   max_price_per_kwh: 10                     │
│ )                                           │
│                                             │
│ Process:                                    │
│ ├─ Calculate max cost: 50 * 10 = 500       │
│ ├─ Validate user has 500 tokens            │
│ ├─ Transfer 500 tokens to escrow            │
│ ├─ Create Order PDA                         │
│ │  ├─ buyer: UserB                          │
│ │  ├─ amount: 50                            │
│ │  ├─ status: Active                        │
│ │  └─ escrow_amount: 500                    │
│ └─ Event: BuyOrderCreated                   │
│                                             │
│ UserB Balance: 500 tokens (1000 - 500 escrow)│
└─────────────────────────────────────────────┘
         │
         ▼

Step 12: Order Matching
┌─────────────────────────────────────────────┐
│ Trading.match_orders()                      │
│                                             │
│ Order Compatibility Check:                  │
│ ├─ OrderA (sell) active ✓                   │
│ ├─ OrderB (buy) active ✓                    │
│ ├─ Price compatible ✓ (10 == 10)            │
│ └─ Sufficient escrow ✓                      │
│                                             │
│ Trade Calculations:                         │
│ ├─ trade_amount = 50 kWh                    │
│ ├─ trade_price = 10 tokens/kWh              │
│ ├─ total_value = 50 * 10 = 500 tokens       │
│ ├─ fee = 500 * 0.0025 = 1.25 tokens         │
│ └─ seller_gets = 500 - 1.25 = 498.75 tokens │
│                                             │
│ Token Transfers:                            │
│ ├─ 50 energy tokens → UserB                 │
│ ├─ 498.75 tokens → UserA                    │
│ ├─ 1.25 tokens → Market (fee)               │
│ └─ Return unused escrow to UserB            │
│                                             │
│ State Updates:                              │
│ ├─ OrderA.status = Completed                │
│ ├─ OrderB.status = Completed                │
│ ├─ Market.total_trades += 1                 │
│ ├─ Market.total_volume += 50                │
│ └─ Create TradeRecord                       │
│                                             │
│ Events:                                     │
│ ├─ OrdersMatched                            │
│ └─ TradeExecuted                            │
└─────────────────────────────────────────────┘
         │
         ▼

Final State:
┌─────────────────────────────────────────────┐
│ UserA (Producer):                           │
│ ├─ Started: 100 tokens (50 energy + 50 cash)│
│ ├─ Sold: 50 kWh energy @ 10 tokens/kWh     │
│ └─ Final: 548.75 tokens (50 + 498.75)       │
│   (Gained 498.75 from sale)                │
│                                             │
│ UserB (Consumer):                           │
│ ├─ Started: 1000 tokens                     │
│ ├─ Bought: 50 kWh energy @ 10 tokens/kWh   │
│ └─ Final: 550 tokens + 50 kWh energy        │
│   (500 spent on energy + 50 energy tokens) │
│                                             │
│ Market:                                     │
│ ├─ Fee collected: 1.25 tokens               │
│ ├─ Total volume: 50 kWh                     │
│ └─ Total trades: 1                          │
└─────────────────────────────────────────────┘
```

---

## Multi-Program Transaction Sequences

### Energy Meter Registration Flow

```
Step 1: User Registration
┌─────────────────────────────────────────────┐
│ Registry.register_user(                     │
│   user_type: Prosumer,                      │
│   location: "123 Solar Street"              │
│ )                                           │
│                                             │
│ Creates:                                    │
│ ├─ UserAccount PDA                          │
│ │  ├─ owner: user_pubkey                    │
│ │  ├─ user_type: Prosumer                   │
│ │  ├─ location: "123 Solar Street"          │
│ │  └─ status: Active                        │
│ ├─ Registry.user_count += 1                 │
│ └─ Event: UserRegistered                    │
└─────────────────────────────────────────────┘
         │
         ▼

Step 2: Meter Registration
┌─────────────────────────────────────────────┐
│ Registry.register_meter(                    │
│   meter_id: "METER_SOLAR_001",              │
│   meter_type: Solar                         │
│ )                                           │
│                                             │
│ Creates:                                    │
│ ├─ MeterAccount PDA                         │
│ │  ├─ meter_id: "METER_SOLAR_001"           │
│ │  ├─ owner: user_pubkey                    │
│ │  ├─ meter_type: Solar                     │
│ │  └─ status: Active                        │
│ ├─ UserAccount.meter_count += 1             │
│ ├─ Registry.meter_count += 1                │
│ └─ Event: MeterRegistered                   │
└─────────────────────────────────────────────┘
         │
         ▼

Step 3: Oracle Configuration
┌─────────────────────────────────────────────┐
│ Oracle.initialize(                          │
│   api_gateway: gateway_pubkey               │
│ )                                           │
│                                             │
│ Creates:                                    │
│ ├─ OracleData PDA                           │
│ │  ├─ authority: oracle_authority           │
│ │  ├─ api_gateway: gateway_pubkey           │
│ │  └─ active: true                          │
│ └─ Event: OracleInitialized                 │
│                                             │
│ System Ready for Meter Readings             │
└─────────────────────────────────────────────┘
```

### Emergency Pause Flow

```
Step 1: Emergency Detection
┌─────────────────────────────────────────────┐
│ External Event:                             │
│ ├─ Security breach detected                 │
│ ├─ System anomaly identified                │
│ └─ Regulatory requirement triggered         │
└─────────────────────────────────────────────┘
         │
         ▼

Step 2: Emergency Pause
┌─────────────────────────────────────────────┐
│ Governance.emergency_pause()                │
│                                             │
│ Immediate Actions:                          │
│ ├─ PoAConfig.emergency_paused = true        │
│ ├─ PoAConfig.emergency_timestamp = now      │
│ └─ Event: EmergencyPauseActivated           │
│                                             │
│ System-wide Impact:                         │
│ ├─ All trading functions blocked            │
│ ├─ Token transfers halted                   │
│ ├─ New order creation disabled              │
│ ├─ ERC issuance paused                      │
│ └─ Only emergency functions available       │
└─────────────────────────────────────────────┘
         │
         ▼

Step 3: Emergency Resolution
┌─────────────────────────────────────────────┐
│ Manual Intervention:                        │
│ ├─ Authority investigates issue             │
│ ├─ Problem identified and resolved          │
│ └─ System integrity verified                │
│                                             │
│ Governance.emergency_unpause()              │
│                                             │
│ Recovery Actions:                           │
│ ├─ PoAConfig.emergency_paused = false       │
│ ├─ All functions re-enabled                 │
│ ├─ Pending transactions processed           │
│ └─ Event: EmergencyPauseDeactivated         │
│                                             │
│ System Restored to Normal Operation         │
└─────────────────────────────────────────────┘
```

---

## Error Handling and Recovery Flows

### Failed Order Creation Recovery

```
Error Scenario: Insufficient Token Balance
┌─────────────────────────────────────────────┐
│ User attempts: create_sell_order(100)       │
│ User balance: 50 tokens                     │
│                                             │
│ Validation Failure:                         │
│ ├─ Check: balance >= amount                 │
│ ├─ Result: 50 < 100 ❌                      │
│ └─ Error: InsufficientBalance               │
│                                             │
│ Recovery Actions:                           │
│ ├─ Transaction reverted                     │
│ ├─ No state changes made                    │
│ ├─ User tokens remain unchanged             │
│ └─ Error event emitted                      │
│                                             │
│ User Options:                               │
│ ├─ Reduce order amount to ≤ 50              │
│ ├─ Acquire more tokens                      │
│ └─ Wait for incoming token transfers        │
└─────────────────────────────────────────────┘
```

### Failed Order Matching Recovery

```
Error Scenario: Price Mismatch During Matching
┌─────────────────────────────────────────────┐
│ SellOrder: 50 kWh @ 12 tokens/kWh           │
│ BuyOrder: 50 kWh @ max 10 tokens/kWh        │
│                                             │
│ Matching Attempt:                           │
│ ├─ Check: sell_price <= buy_max_price       │
│ ├─ Result: 12 > 10 ❌                       │
│ └─ Error: PriceMismatch                     │
│                                             │
│ Recovery Actions:                           │
│ ├─ Orders remain Active                     │
│ ├─ Escrow funds remain locked               │
│ ├─ No token transfers executed              │
│ └─ Event: MatchingFailed                    │
│                                             │
│ Resolution Options:                         │
│ ├─ Seller reduces price to ≤ 10             │
│ ├─ Buyer increases max price to ≥ 12        │
│ ├─ Orders expire naturally                  │
│ └─ Users cancel and recreate orders         │
└─────────────────────────────────────────────┘
```

### Oracle Connection Failure Recovery

```
Error Scenario: API Gateway Unreachable
┌─────────────────────────────────────────────┐
│ Oracle.submit_meter_reading() called        │
│ API Gateway not responding                  │
│                                             │
│ Validation Failure:                         │
│ ├─ Check: caller == api_gateway             │
│ ├─ Result: unauthorized caller ❌           │
│ └─ Error: UnauthorizedOracle                │
│                                             │
│ Recovery Actions:                           │
│ ├─ Reading submission rejected              │
│ ├─ Oracle state unchanged                   │
│ ├─ Error logged with timestamp              │
│ └─ Event: OracleSubmissionFailed            │
│                                             │
│ System Response:                            │
│ ├─ Retry mechanism activated                │
│ ├─ Fallback procedures initiated            │
│ ├─ Authority notified of issues             │
│ └─ Manual intervention may be required      │
└─────────────────────────────────────────────┘
```

---

## Cross-Program Communication Patterns

### Oracle → Registry Communication

```
Call Pattern: submit_meter_reading → update_meter_reading
┌─────────────────────────────────────────────┐
│ Oracle Program (Caller)                     │
│ ├─ Validates API Gateway signature          │
│ ├─ Processes meter reading data             │
│ ├─ Updates internal Oracle state            │
│ └─ Issues CPI call to Registry              │
│                                             │
│ Cross-Program Invocation (CPI):             │
│ ├─ Target: Registry Program                 │
│ ├─ Function: update_meter_reading           │
│ ├─ Accounts: MeterAccount PDA               │
│ └─ Data: meter_id, production, timestamp    │
│                                             │
│ Registry Program (Callee)                   │
│ ├─ Validates caller is Oracle Program       │
│ ├─ Updates MeterAccount data                │
│ ├─ Updates UserAccount aggregates           │
│ └─ Returns success/failure to Oracle        │
│                                             │
│ Response Handling:                          │
│ ├─ Oracle receives Registry response        │
│ ├─ Updates Oracle reading counters          │
│ ├─ Emits combined success event             │
│ └─ Transaction completes atomically         │
└─────────────────────────────────────────────┘
```

### Registry → Governance Communication

```
Call Pattern: update_meter_reading → issue_erc
┌─────────────────────────────────────────────┐
│ Registry Program (Caller)                   │
│ ├─ Meter reading updated successfully       │
│ ├─ Detects renewable energy production      │
│ ├─ Qualifies for ERC issuance               │
│ └─ Issues CPI call to Governance            │
│                                             │
│ Cross-Program Invocation (CPI):             │
│ ├─ Target: Governance Program               │
│ ├─ Function: issue_erc                      │
│ ├─ Accounts: PoAConfig, ErcCertificate PDA  │
│ └─ Data: certificate_id, amount, source     │
│                                             │
│ Governance Program (Callee)                 │
│ ├─ Validates caller is Registry Program     │
│ ├─ Checks emergency pause status            │
│ ├─ Validates ERC limits and quotas          │
│ ├─ Creates new ErcCertificate               │
│ └─ Returns certificate details              │
│                                             │
│ Response Handling:                          │
│ ├─ Registry receives ERC details            │
│ ├─ Links ERC to meter production            │
│ ├─ Updates user ERC tracking                │
│ └─ Emits ERC issuance event                 │
└─────────────────────────────────────────────┘
```

### Governance → Energy-Token Communication

```
Call Pattern: validate_erc_for_trading → transfer_tokens
┌─────────────────────────────────────────────┐
│ Governance Program (Caller)                 │
│ ├─ ERC validated for trading use            │
│ ├─ Certificate marked as trading-approved   │
│ ├─ Calculates token amount to issue         │
│ └─ Issues CPI call to Energy-Token          │
│                                             │
│ Cross-Program Invocation (CPI):             │
│ ├─ Target: Energy-Token Program             │
│ ├─ Function: transfer_tokens                │
│ ├─ Accounts: TokenInfo, User TokenAccount   │
│ └─ Data: amount, recipient, ERC reference   │
│                                             │
│ Energy-Token Program (Callee)               │
│ ├─ Validates caller is Governance Program   │
│ ├─ Checks token supply limits               │
│ ├─ Creates user token account if needed     │
│ ├─ Mints tokens via SPL Token Program       │
│ └─ Returns minting confirmation             │
│                                             │
│ Response Handling:                          │
│ ├─ Governance receives minting confirmation │
│ ├─ Updates ERC with token issuance details  │
│ ├─ Links ERC to token supply                │
│ └─ Emits token issuance event               │
└─────────────────────────────────────────────┘
```

---

## State Transition Workflows

### Order Lifecycle State Machine

```
Order States and Transitions:
┌─────────────────────────────────────────────┐
│                                             │
│ [Initial] ──create──► [Active]              │
│                          │                  │
│                          │                  │
│                    ┌─────┼─────┐            │
│                    ▼     ▼     ▼            │
│               [Cancelled]  │  [Expired]      │
│                          │                  │
│                          ▼                  │
│                  [PartiallyFilled]          │
│                          │                  │
│                          ▼                  │
│                    [Completed]              │
│                                             │
└─────────────────────────────────────────────┘

State Transition Details:

Active → PartiallyFilled:
├─ Trigger: match_orders() with partial match
├─ Conditions: order.filled_amount < order.amount
├─ Actions: Update filled_amount, adjust escrow
└─ Events: OrderPartiallyFilled

PartiallyFilled → Completed:
├─ Trigger: match_orders() completes remaining amount
├─ Conditions: order.filled_amount == order.amount
├─ Actions: Set status to Completed, release escrow
└─ Events: OrderCompleted

Active → Cancelled:
├─ Trigger: cancel_order() by order owner
├─ Conditions: order.status == Active, valid owner
├─ Actions: Return escrow, set cancelled_at
└─ Events: OrderCancelled

Active → Expired:
├─ Trigger: Current time > order.expires_at
├─ Conditions: Automatic system check
├─ Actions: Return escrow, cleanup order
└─ Events: OrderExpired
```

### ERC Certificate Lifecycle

```
ERC States and Transitions:
┌─────────────────────────────────────────────┐
│                                             │
│ [Initial] ──issue──► [Valid]                │
│                         │                   │
│                         │                   │
│                    ┌────┼────┐              │
│                    ▼    ▼    ▼              │
│              [Expired] │ [Revoked]           │
│                        │                    │
│                        ▼                    │
│               [TradingValidated]            │
│                                             │
└─────────────────────────────────────────────┘

State Transition Details:

Valid → TradingValidated:
├─ Trigger: validate_erc_for_trading()
├─ Conditions: ERC is Valid, within validity period
├─ Actions: Set trading_validated_at, trigger tokens
└─ Events: ErcValidatedForTrading

Valid → Expired:
├─ Trigger: Current time > erc.expires_at
├─ Conditions: Automatic expiration check
├─ Actions: Set status to Expired
└─ Events: ErcExpired

Valid → Revoked:
├─ Trigger: Authority intervention
├─ Conditions: Manual governance decision
├─ Actions: Set status to Revoked, block trading
└─ Events: ErcRevoked
```

### User Account Status Workflow

```
User States and Transitions:
┌─────────────────────────────────────────────┐
│                                             │
│ [Initial] ──register──► [Active]            │
│                            │                │
│                            │                │
│                       ┌────┼────┐           │
│                       ▼    ▼    ▼           │
│                [Suspended] │ [Inactive]      │
│                       │    │                │
│                       │    │                │
│                       └────┼────┘           │
│                            │                │
│                            ▼                │
│                     [Reactivated]           │
│                            │                │
│                            ▼                │
│                       [Active]              │
│                                             │
└─────────────────────────────────────────────┘

State Transition Details:

Active → Suspended:
├─ Trigger: update_user_status() by authority
├─ Conditions: Policy violation, investigation needed
├─ Actions: Block trading, maintain meter data
└─ Events: UserSuspended

Suspended → Reactivated:
├─ Trigger: update_user_status() by authority
├─ Conditions: Investigation resolved, compliance restored
├─ Actions: Restore trading rights, clear restrictions
└─ Events: UserReactivated

Active → Inactive:
├─ Trigger: User request or prolonged inactivity
├─ Conditions: No trading activity, user initiated
├─ Actions: Soft deactivation, preserve data
└─ Events: UserDeactivated
```

---

**[← Back to Architecture Overview](./README.md)**