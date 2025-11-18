# Process Specifications - GridTokenX Platform
ใช้ 3 วิธีในการระบุ Process: Structured English, Decision Tables, และ Decision Trees

---

## Table of Contents
1. [Process Specification แบบ Structured English](#1-process-specification-แบบ-structured-english)
2. [Process Specification แบบ Decision Tables](#2-process-specification-แบบ-decision-tables)
3. [Process Specification แบบ Decision Trees](#3-process-specification-แบบ-decision-trees)

---

# 1. Process Specification แบบ Structured English

## Process 3.2: Validate Trading Order (การตรวจสอบความถูกต้องของคำสั่งซื้อขาย)

### Process ID: P3.2
### Process Name: Validate Trading Order
### Description: ตรวจสอบความถูกต้องและความสมบูรณ์ของคำสั่งซื้อขายก่อนเพิ่มลงในระบบ

### Structured English Specification:

```
BEGIN Validate_Trading_Order

    INPUT: Order_Details (order_type, quantity, price, user_id, wallet_address)
    OUTPUT: Validation_Result (valid/invalid, error_messages[])
    
    DECLARE valid = TRUE
    DECLARE error_messages = []
    
    // Step 1: ตรวจสอบการ Authentication
    IF user_id IS NOT authenticated THEN
        SET valid = FALSE
        ADD "User not authenticated" TO error_messages
        RETURN validation_result
    END IF
    
    // Step 2: ตรวจสอบประเภทคำสั่ง
    IF order_type NOT IN ['BUY', 'SELL'] THEN
        SET valid = FALSE
        ADD "Invalid order type" TO error_messages
    END IF
    
    // Step 3: ตรวจสอบปริมาณ (Quantity)
    IF quantity IS NULL OR quantity <= 0 THEN
        SET valid = FALSE
        ADD "Quantity must be greater than 0" TO error_messages
    ELSE IF quantity < MINIMUM_ORDER_QUANTITY THEN
        SET valid = FALSE
        ADD "Quantity below minimum (0.1 kWh)" TO error_messages
    ELSE IF quantity > MAXIMUM_ORDER_QUANTITY THEN
        SET valid = FALSE
        ADD "Quantity exceeds maximum (1000 kWh)" TO error_messages
    END IF
    
    // Step 4: ตรวจสอบราคา (Price)
    IF price IS NULL OR price <= 0 THEN
        SET valid = FALSE
        ADD "Price must be greater than 0" TO error_messages
    ELSE
        GET current_market_price FROM Oracle
        IF price < (current_market_price * 0.5) THEN
            SET valid = FALSE
            ADD "Price too low (< 50% of market price)" TO error_messages
        ELSE IF price > (current_market_price * 2.0) THEN
            SET valid = FALSE
            ADD "Price too high (> 200% of market price)" TO error_messages
        END IF
    END IF
    
    // Step 5: ตรวจสอบยอดคงเหลือสำหรับคำสั่งขาย (SELL)
    IF order_type = 'SELL' THEN
        GET token_balance FROM Blockchain WHERE wallet = wallet_address
        IF token_balance < quantity THEN
            SET valid = FALSE
            ADD "Insufficient token balance" TO error_messages
        END IF
        
        // ตรวจสอบ token ถูก lock ไว้หรือไม่
        GET locked_tokens FROM Active_Orders WHERE user_id = user_id
        GET available_balance = token_balance - locked_tokens
        IF available_balance < quantity THEN
            SET valid = FALSE
            ADD "Tokens locked in other orders" TO error_messages
        END IF
    END IF
    
    // Step 6: ตรวจสอบยอดเงินสำหรับคำสั่งซื้อ (BUY)
    IF order_type = 'BUY' THEN
        GET usdc_balance FROM Blockchain WHERE wallet = wallet_address
        CALCULATE required_amount = quantity * price
        IF usdc_balance < required_amount THEN
            SET valid = FALSE
            ADD "Insufficient USDC balance" TO error_messages
        END IF
        
        // ตรวจสอบเงินถูก lock ไว้หรือไม่
        GET locked_usdc FROM Active_Orders WHERE user_id = user_id
        GET available_usdc = usdc_balance - locked_usdc
        IF available_usdc < required_amount THEN
            SET valid = FALSE
            ADD "USDC locked in other orders" TO error_messages
        END IF
    END IF
    
    // Step 7: ตรวจสอบขอบเขตการซื้อขายรายวัน
    GET daily_trade_volume FROM Trade_History 
        WHERE user_id = user_id 
        AND date = TODAY
    IF (daily_trade_volume + quantity) > DAILY_LIMIT THEN
        SET valid = FALSE
        ADD "Daily trading limit exceeded" TO error_messages
    END IF
    
    // Step 8: ตรวจสอบ Rate Limiting
    GET order_count FROM Orders 
        WHERE user_id = user_id 
        AND created_at > (CURRENT_TIME - 1 MINUTE)
    IF order_count >= MAX_ORDERS_PER_MINUTE THEN
        SET valid = FALSE
        ADD "Too many orders. Please wait." TO error_messages
    END IF
    
    // Step 9: สร้างผลลัพธ์
    IF valid = TRUE THEN
        GENERATE unique_order_id
        SET order_status = 'VALIDATED'
        LOCK_FUNDS (order_type, quantity, price, wallet_address)
        RETURN (status: "success", order_id: unique_order_id)
    ELSE
        RETURN (status: "error", errors: error_messages)
    END IF

END Validate_Trading_Order
```

### Constants and Parameters:

| Parameter | Value | Description |
|-----------|-------|-------------|
| `MINIMUM_ORDER_QUANTITY` | 0.1 kWh | ปริมาณขั้นต่ำต่อคำสั่ง |
| `MAXIMUM_ORDER_QUANTITY` | 1000 kWh | ปริมาณสูงสุดต่อคำสั่ง |
| `DAILY_LIMIT` | 5000 kWh | ขีดจำกัดการซื้อขายต่อวัน |
| `MAX_ORDERS_PER_MINUTE` | 10 | จำนวนคำสั่งสูงสุดต่อนาที |
| `PRICE_RANGE_MULTIPLIER_MIN` | 0.5 | ราคาต่ำสุด (50% ของราคาตลาด) |
| `PRICE_RANGE_MULTIPLIER_MAX` | 2.0 | ราคาสูงสุด (200% ของราคาตลาด) |

### Example Scenarios:

**Scenario 1: Valid Sell Order**
```
Input:
  order_type: "SELL"
  quantity: 10 kWh
  price: 2.5 USDC/kWh
  user_id: "user-123"
  wallet: "ABC...xyz"
  
Validation:
  ✓ User authenticated
  ✓ Order type valid
  ✓ Quantity valid (0.1 ≤ 10 ≤ 1000)
  ✓ Price reasonable (2.5 within 50-200% of market price)
  ✓ Token balance sufficient (15 kWh available)
  ✓ Daily limit not exceeded
  ✓ Rate limit OK
  
Output:
  status: "success"
  order_id: "ord-789abc"
  locked_tokens: 10 kWh
```

**Scenario 2: Invalid Buy Order - Insufficient Balance**
```
Input:
  order_type: "BUY"
  quantity: 50 kWh
  price: 3.0 USDC/kWh
  user_id: "user-456"
  wallet: "DEF...xyz"
  
Validation:
  ✓ User authenticated
  ✓ Order type valid
  ✓ Quantity valid
  ✓ Price reasonable
  ✗ USDC balance insufficient (100 USDC available, 150 USDC required)
  
Output:
  status: "error"
  errors: ["Insufficient USDC balance"]
```

---

# 2. Process Specification แบบ Decision Tables

## Process 2.2: Verify Energy Production Data (การตรวจสอบข้อมูลการผลิตพลังงาน)

### Process ID: P2.2
### Process Name: Verify Energy Production Data
### Description: ตรวจสอบความถูกต้องของข้อมูลการผลิตพลังงานจาก Smart Meter ก่อนนำไปใช้ในการ Mint Token

### Decision Table 1: Anomaly Detection (การตรวจจับความผิดปกติ)

| Condition # | C1 | C2 | C3 | C4 | C5 | C6 | C7 | C8 |
|-------------|----|----|----|----|----|----|----|----|
| **Conditions** ||||||||
| Production > Max Capacity | Y | Y | Y | Y | N | N | N | N |
| Sudden Spike (>50% increase) | Y | Y | N | N | Y | Y | N | N |
| Weather Supports Production | Y | N | Y | N | Y | N | Y | N |
| **Actions** ||||||||
| Mark as VALID | | | | | | | ✓ | |
| Mark as SUSPICIOUS | ✓ | ✓ | ✓ | | ✓ | | | |
| Mark as INVALID | | | | ✓ | | ✓ | | ✓ |
| Send Alert to Operator | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | ✓ |
| Request Manual Review | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | | |
| Auto-Reject Data | | | | ✓ | | ✓ | | ✓ |
| Allow Minting | | | | | | | ✓ | |

**Legend:**
- Y = Yes (เงื่อนไขเป็นจริง)
- N = No (เงื่อนไขไม่เป็นจริง)
- ✓ = Action should be taken (ดำเนินการ)

### Decision Table 2: Data Quality Assessment

| Rule # | R1 | R2 | R3 | R4 | R5 | R6 | R7 | R8 |
|--------|----|----|----|----|----|----|----|----|
| **Conditions** ||||||||
| Data Timestamp Recent (<10 min) | Y | Y | Y | Y | N | N | N | N |
| Meter Online Status | Y | Y | N | N | Y | Y | N | N |
| Data Format Valid (JSON) | Y | N | Y | N | Y | N | Y | N |
| **Actions** ||||||||
| Accept Data | ✓ | | | | | | | |
| Reject - Stale Data | | | | | ✓ | ✓ | ✓ | ✓ |
| Reject - Meter Offline | | | ✓ | ✓ | | | ✓ | ✓ |
| Reject - Invalid Format | | ✓ | | ✓ | | ✓ | | ✓ |
| Store in Raw Data Log | ✓ | ✓ | ✓ | ✓ | | | | |
| Process for Verification | ✓ | | | | | | | |

### Decision Table 3: Verification Decision Matrix

| Scenario | S1 | S2 | S3 | S4 | S5 | S6 | S7 | S8 |
|----------|----|----|----|----|----|----|----|----|
| **Input Conditions** ||||||||
| Data Quality: High | Y | Y | Y | Y | N | N | N | N |
| Historical Pattern Match | Y | Y | N | N | Y | Y | N | N |
| Solar Irradiance Correlation | Y | N | Y | N | Y | N | Y | N |
| **Verification Status** ||||||||
| Confidence Level | 95% | 85% | 80% | 70% | 75% | 60% | 55% | 40% |
| Verification Result | AUTO_APPROVE | AUTO_APPROVE | REVIEW | REVIEW | REVIEW | MANUAL | MANUAL | REJECT |
| Proceed to Minting | ✓ | ✓ | | | | | | |
| Queue for Review | | | ✓ | ✓ | ✓ | ✓ | ✓ | |
| Notify Operator | | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| Send Alert to User | | | | | ✓ | ✓ | ✓ | ✓ |

### Implementation Code Structure:

```python
def verify_energy_production(meter_data):
    """
    ตรวจสอบข้อมูลการผลิตพลังงานตาม Decision Tables
    """
    # Decision Table 1: Anomaly Detection
    anomaly_score = 0
    
    if meter_data.production > meter_data.max_capacity:
        anomaly_score += 40
    
    if is_sudden_spike(meter_data.production, meter_data.historical_avg):
        anomaly_score += 30
    
    if not weather_supports_production(meter_data.timestamp):
        anomaly_score += 20
    
    if anomaly_score >= 60:
        return {
            'status': 'INVALID',
            'action': 'AUTO_REJECT',
            'alert': True
        }
    elif anomaly_score >= 30:
        return {
            'status': 'SUSPICIOUS',
            'action': 'MANUAL_REVIEW',
            'alert': True
        }
    else:
        return {
            'status': 'VALID',
            'action': 'PROCEED',
            'alert': False
        }
```

### Decision Table Examples:

**Example 1: Rule C7 - Valid Production**
```
Conditions:
  - Production = 4.5 kWh (Max capacity = 5 kWh) ✓
  - Sudden Spike = No (previous reading: 4.2 kWh) ✓
  - Weather = Sunny (supports production) ✓

Decision:
  → Mark as VALID
  → Allow Minting
  → No alert needed
```

**Example 2: Rule C4 - Invalid Data**
```
Conditions:
  - Production = 8.0 kWh (Max capacity = 5 kWh) ✗
  - Sudden Spike = Yes (previous reading: 2.0 kWh) ✗
  - Weather = Cloudy (does not support) ✗

Decision:
  → Mark as INVALID
  → Auto-Reject Data
  → Send Alert to Operator
  → Request Manual Review
```

**Example 3: Rule R1 - Accept Data**
```
Conditions:
  - Timestamp = 2 minutes ago ✓
  - Meter Status = Online ✓
  - Data Format = Valid JSON ✓

Decision:
  → Accept Data
  → Store in Raw Data Log
  → Process for Verification
```

---

# 3. Process Specification แบบ Decision Trees

## Process 3.3: Order Matching Algorithm (อัลกอริทึมการจับคู่คำสั่งซื้อขาย)

### Process ID: P3.3
### Process Name: Market Clearing - Order Matching
### Description: ตัดสินใจว่าจะจับคู่คำสั่งซื้อขายอย่างไร และคำนวณราคา Clearing Price

### Decision Tree Diagram:

```
                    [START: Market Clearing Triggered]
                                |
                    [Check: Are there active orders?]
                         /              \
                       YES               NO
                        |                 |
        [Check: Buy orders exist?]    [End: No trades]
                /         \
              YES          NO
               |            |
    [Check: Sell orders?] [End: No matches]
           /        \
         YES         NO
          |          |
    [Sort Orders]  [End: No matches]
          |
    [Decision 1: Price Match?]
     (Highest Buy ≥ Lowest Sell)
          /              \
        YES               NO
         |                 |
    [Decision 2:]      [End: No clearing]
    [Volume Match?]
    /     |      \
  Full  Partial  None
   |      |       |
[Case A][Case B][Case C]
   |      |       |
   v      v       v
[Execute Trade]
```

### Detailed Decision Tree with Logic:

```
┌─────────────────────────────────────────────────────────────┐
│ ROOT NODE: Market Clearing Process                         │
│ Trigger: Every 15 minutes                                   │
└─────────────────────────────────────────────────────────────┘
                        │
        ┌───────────────┴───────────────┐
        │                               │
    Decision 1                      Decision 1
    [Active Orders?]                [Active Orders?]
        │                               │
       YES                             NO
        │                               │
        v                               v
┌─────────────────┐            ┌──────────────┐
│ Decision 2:     │            │ ACTION:      │
│ Buy Orders > 0? │            │ End Process  │
└─────────────────┘            │ No clearing  │
        │                      └──────────────┘
    ┌───┴───┐
   YES      NO
    │        │
    v        v
┌─────────┐ ┌──────────┐
│Decision3│ │ ACTION:  │
│Sell > 0?│ │ End      │
└─────────┘ └──────────┘
    │
┌───┴───┐
YES      NO
 │        │
 v        v
┌──────────────────┐  ┌──────────┐
│ PROCESSING:      │  │ ACTION:  │
│ 1. Sort Buy DESC │  │ End      │
│ 2. Sort Sell ASC │  └──────────┘
│ 3. Find matches  │
└──────────────────┘
         │
         v
┌─────────────────────────────────────┐
│ Decision 4: Price Comparison        │
│ IF (Highest Buy Price ≥ Lowest Sell)│
└─────────────────────────────────────┘
         │
    ┌────┴────┐
   YES        NO
    │          │
    v          v
┌────────┐  ┌──────────┐
│Continue│  │ ACTION:  │
│Matching│  │ End      │
└────────┘  │ No match │
    │       └──────────┘
    v
┌──────────────────────────────────────┐
│ Decision 5: Volume Comparison        │
│ Compare Buy Qty vs Sell Qty          │
└──────────────────────────────────────┘
    │
    ├───────────┬───────────┬───────────┐
    │           │           │           │
 Buy=Sell    Buy>Sell    Buy<Sell    Error
    │           │           │           │
    v           v           v           v
┌────────┐  ┌────────┐  ┌────────┐  ┌──────┐
│Case A: │  │Case B: │  │Case C: │  │Reject│
│Full    │  │Partial │  │Partial │  │Trade │
│Match   │  │Buy     │  │Sell    │  └──────┘
└────────┘  └────────┘  └────────┘
    │           │           │
    v           v           v
┌───────────────────────────────────┐
│ Decision 6: Calculate Price       │
│ Clearing Price Determination      │
└───────────────────────────────────┘
    │
    ├─────────────┬─────────────┬──────────────┐
    │             │             │              │
 Midpoint    Weighted Avg   Market Price   Auction
    │             │             │              │
    v             v             v              v
  (Buy+Sell)/2  Complex     Oracle Price   Highest Bid
    │             │             │              │
    └─────────────┴─────────────┴──────────────┘
                    │
                    v
        ┌───────────────────────┐
        │ EXECUTE TRADE         │
        │ 1. Transfer tokens    │
        │ 2. Transfer USDC      │
        │ 3. Apply fees         │
        │ 4. Update orders      │
        │ 5. Record trade       │
        └───────────────────────┘
                    │
                    v
        ┌───────────────────────┐
        │ Decision 7:           │
        │ More orders to match? │
        └───────────────────────┘
                    │
            ┌───────┴───────┐
           YES              NO
            │                │
            v                v
    [Loop back to      [END: Process
     Decision 4]        Complete]
```

### Decision Tree Rules and Actions:

#### Node 1: Active Orders Check
```
IF (active_buy_orders.count == 0 AND active_sell_orders.count == 0) THEN
    RETURN "No orders to process"
    EXIT
END IF
```

#### Node 2: Buy Orders Availability
```
IF (active_buy_orders.count == 0) THEN
    RETURN "No buy orders available"
    EXIT
ELSE
    CONTINUE to Node 3
END IF
```

#### Node 3: Sell Orders Availability
```
IF (active_sell_orders.count == 0) THEN
    RETURN "No sell orders available"
    EXIT
ELSE
    CONTINUE to Processing
END IF
```

#### Processing: Sort and Prepare
```
// Sort buy orders by price (descending) and timestamp (FIFO)
buy_orders = SORT(active_buy_orders, 
                  ORDER BY price DESC, created_at ASC)

// Sort sell orders by price (ascending) and timestamp (FIFO)
sell_orders = SORT(active_sell_orders, 
                   ORDER BY price ASC, created_at ASC)

highest_buy = buy_orders[0]
lowest_sell = sell_orders[0]
```

#### Node 4: Price Match Decision
```
IF (highest_buy.price >= lowest_sell.price) THEN
    // Match possible
    CONTINUE to Node 5
ELSE
    // No overlap in prices
    RETURN "No price match possible"
    EXIT
END IF
```

#### Node 5: Volume Comparison
```
buy_qty = highest_buy.quantity
sell_qty = lowest_sell.quantity

CASE:
    WHEN buy_qty == sell_qty:
        match_type = "FULL_MATCH"
        trade_qty = buy_qty
        
    WHEN buy_qty > sell_qty:
        match_type = "PARTIAL_BUY"
        trade_qty = sell_qty
        remaining_buy = buy_qty - sell_qty
        
    WHEN buy_qty < sell_qty:
        match_type = "PARTIAL_SELL"
        trade_qty = buy_qty
        remaining_sell = sell_qty - buy_qty
        
    ELSE:
        REJECT_TRADE()
END CASE
```

#### Node 6: Clearing Price Calculation
```
SWITCH (pricing_method):
    CASE "midpoint":
        clearing_price = (highest_buy.price + lowest_sell.price) / 2
        
    CASE "weighted_average":
        total_volume = buy_qty + sell_qty
        clearing_price = (highest_buy.price * buy_qty + 
                         lowest_sell.price * sell_qty) / total_volume
        
    CASE "market_price":
        clearing_price = GET_ORACLE_PRICE()
        
    CASE "uniform_auction":
        clearing_price = highest_buy.price  // Pay-as-bid
        
    DEFAULT:
        clearing_price = (highest_buy.price + lowest_sell.price) / 2
END SWITCH
```

#### Execute Trade Action
```
EXECUTE_TRADE:
    1. Calculate amounts:
       total_value = trade_qty * clearing_price
       platform_fee = total_value * FEE_RATE (0.5%)
       seller_receives = total_value - platform_fee
       
    2. Transfer REC tokens:
       TRANSFER(from: seller.wallet, 
               to: buyer.wallet, 
               amount: trade_qty)
       
    3. Transfer USDC:
       TRANSFER(from: buyer.wallet, 
               to: seller.wallet, 
               amount: seller_receives)
       
       TRANSFER(from: buyer.wallet, 
               to: platform.wallet, 
               amount: platform_fee)
       
    4. Update order statuses:
       IF match_type == "FULL_MATCH":
           highest_buy.status = "FILLED"
           lowest_sell.status = "FILLED"
       ELSE IF match_type == "PARTIAL_BUY":
           highest_buy.status = "PARTIAL"
           highest_buy.remaining_qty = remaining_buy
           lowest_sell.status = "FILLED"
       ELSE IF match_type == "PARTIAL_SELL":
           highest_buy.status = "FILLED"
           lowest_sell.status = "PARTIAL"
           lowest_sell.remaining_qty = remaining_sell
       END IF
       
    5. Record trade:
       INSERT INTO trade_history (
           trade_id, buy_order_id, sell_order_id,
           quantity, price, total_value, fee,
           executed_at
       )
       
    6. Send notifications:
       NOTIFY(buyer, "Trade executed")
       NOTIFY(seller, "Trade executed")
```

#### Node 7: Continue Matching
```
IF (active_buy_orders.count > 0 AND active_sell_orders.count > 0) THEN
    // More orders exist
    GOTO Node 4  // Loop back to price match decision
ELSE
    RETURN "Market clearing complete"
    EXIT
END IF
```

### Decision Tree Example Scenarios:

#### Scenario 1: Full Match
```
Input:
  Buy Orders:  [10 kWh @ 3.0 USDC]
  Sell Orders: [10 kWh @ 2.5 USDC]

Decision Path:
  Node 1: YES (orders exist)
  Node 2: YES (buy orders exist)
  Node 3: YES (sell orders exist)
  Node 4: YES (3.0 ≥ 2.5) ✓
  Node 5: FULL_MATCH (10 == 10)
  Node 6: Clearing Price = (3.0 + 2.5) / 2 = 2.75 USDC
  
Execute:
  Trade: 10 kWh @ 2.75 USDC
  Total: 27.50 USDC
  Fee: 0.14 USDC (0.5%)
  Seller receives: 27.36 USDC
  
  Both orders marked as FILLED
```

#### Scenario 2: Partial Match (Buy > Sell)
```
Input:
  Buy Orders:  [50 kWh @ 3.2 USDC]
  Sell Orders: [30 kWh @ 2.8 USDC]

Decision Path:
  Node 1-4: All YES
  Node 5: PARTIAL_BUY (50 > 30)
         trade_qty = 30 kWh
         remaining_buy = 20 kWh
  Node 6: Clearing Price = 3.0 USDC
  
Execute:
  Trade: 30 kWh @ 3.0 USDC
  Total: 90.00 USDC
  
  Sell order: FILLED
  Buy order: PARTIAL (20 kWh remaining @ 3.2 USDC)
  
Node 7: Continue matching with remaining buy order
```

#### Scenario 3: No Match (Price Gap)
```
Input:
  Buy Orders:  [25 kWh @ 2.0 USDC]
  Sell Orders: [25 kWh @ 3.5 USDC]

Decision Path:
  Node 1-3: All YES
  Node 4: NO (2.0 < 3.5) ✗
  
Output:
  No trade executed
  Gap = 1.5 USDC (too large)
  Wait for next clearing cycle
```

### Complexity Analysis:

| Metric | Value | Description |
|--------|-------|-------------|
| **Tree Depth** | 7 levels | Maximum decision depth |
| **Decision Nodes** | 7 nodes | Total decision points |
| **Leaf Nodes** | 8 outcomes | Possible end states |
| **Average Path Length** | 5-6 decisions | Typical execution path |
| **Time Complexity** | O(n log n) | Sorting orders |
| **Space Complexity** | O(n) | Storing orders |

---

## Summary Comparison

| Specification Type | Best Used For | Advantages | Example Process |
|--------------------|---------------|------------|----------------|
| **Structured English** | Sequential, complex logic | Easy to read, detailed | Order Validation (P3.2) |
| **Decision Tables** | Multiple conditions, rule-based | Comprehensive coverage | Energy Verification (P2.2) |
| **Decision Trees** | Branching logic, algorithms | Visual clarity, flow | Order Matching (P3.3) |

---

**Document Version**: 1.0  
**Last Updated**: November 3, 2025  
**Status**: ✅ Complete  
**Total Specifications**: 3 processes × 3 methods = 9 detailed specifications
