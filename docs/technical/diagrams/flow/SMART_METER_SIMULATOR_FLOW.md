# Smart Meter Simulator - Data Flow Diagrams

## Overview
Visual representation of data flows in the Smart Meter Simulator system.

---

## Simulation Cycle Flow

```mermaid
graph TB
    Start[Simulation Start] --> Timer[Timer Tick<br/>every N seconds]
    Timer --> Weather[Update Weather<br/>Simulation]
    Weather --> MeterLoop{For Each<br/>Meter}
    
    MeterLoop -->|Meter Config| CalcSolar[Calculate Solar<br/>Generation Factor]
    CalcSolar -->|time + weather| CalcConsume[Calculate<br/>Consumption Pattern]
    CalcConsume -->|time + user type| BatteryMgmt[Battery<br/>Management]
    
    BatteryMgmt --> GenReading[Generate Enhanced<br/>Reading]
    
    GenReading --> ReadingData[EnergyReading Object<br/>with all parameters]
    
    ReadingData --> Validate[Validate<br/>Reading]
    Validate --> Output{Output<br/>Channels}
    
    Output -->|Optional| Kafka[Kafka Producer<br/>3 topics]
    Output -->|Optional| InfluxDB[InfluxDB<br/>Time-series]
    Output -->|Optional| PostgreSQL[PostgreSQL<br/>Relational]
    Output -->|Always| File[JSONL File<br/>Local storage]
    Output -->|If enabled| WebSocket[WebSocket<br/>Broadcast]
    
    Kafka --> Stats[Update<br/>Statistics]
    InfluxDB --> Stats
    PostgreSQL --> Stats
    File --> Stats
    WebSocket --> Stats
    
    Stats --> MeterLoop
    MeterLoop -->|No more| Timer
    
    style Start fill:#90EE90
    style Timer fill:#87CEEB
    style Weather fill:#FFD700
    style GenReading fill:#FF6B6B
    style Output fill:#9370DB
    style Stats fill:#32CD32
```

---

## Weather Simulation State Machine

```mermaid
stateDiagram-v2
    [*] --> Sunny
    
    Sunny --> Sunny: 60%
    Sunny --> PartlyCloudy: 40%
    
    PartlyCloudy --> Sunny: 35%
    PartlyCloudy --> PartlyCloudy: 35%
    PartlyCloudy --> Cloudy: 30%
    
    Cloudy --> PartlyCloudy: 40%
    Cloudy --> Cloudy: 30%
    Cloudy --> Overcast: 30%
    
    Overcast --> Cloudy: 40%
    Overcast --> Overcast: 30%
    Overcast --> Rainy: 30%
    
    Rainy --> Overcast: 50%
    Rainy --> Cloudy: 50%
    
    note right of Sunny
        100% Solar Potential
        Duration: 2-10 cycles
    end note
    
    note right of PartlyCloudy
        70-90% Solar Potential
        Duration: 2-10 cycles
    end note
    
    note right of Cloudy
        40-70% Solar Potential
        Duration: 2-10 cycles
    end note
    
    note right of Overcast
        20-40% Solar Potential
        Duration: 2-10 cycles
    end note
    
    note right of Rainy
        10-30% Solar Potential
        Duration: 2-10 cycles
    end note
```

---

## Solar Generation Calculation Flow

```mermaid
graph LR
    A[Current Time] --> B[Calculate Time Factor<br/>sin²(π×(hour-6)/12)]
    Weather[Weather Condition] --> C[Weather Impact<br/>Sunny: 1.0<br/>Rainy: 0.1-0.3]
    
    B --> D[Calculate Irradiance<br/>time × weather × 1200 W/m²]
    C --> D
    
    D --> E[Panel Temperature<br/>ambient + irradiance/1000 × 25]
    
    E --> F[Temperature Derating<br/>1 + (-0.004)×(temp-25)]
    
    Config[Meter Config] --> G[Base Generation<br/>capacity × efficiency]
    
    G --> H[Final Generation]
    F --> H
    D --> H
    
    H --> I[energy_generated<br/>+ noise]
    
    style A fill:#87CEEB
    style Weather fill:#FFD700
    style Config fill:#90EE90
    style I fill:#FF6B6B
```

---

## Battery Management Flow

```mermaid
graph TB
    Start[Calculate Net Energy<br/>generation - consumption] --> Check{Net Energy?}
    
    Check -->|Positive<br/>Excess| Charge[Charge Battery]
    Check -->|Negative<br/>Deficit| CheckBattery{Battery Level<br/>> 10%?}
    Check -->|Zero| NoAction[No Battery Action]
    
    Charge --> CalcCharge[charge_amount = min(<br/>net_energy × efficiency,<br/>(100-level)/100 × capacity)]
    CalcCharge --> UpdateLevel1[level += charge_amount/capacity × 100]
    UpdateLevel1 --> Surplus[surplus_energy = net_energy - charge_amount]
    
    CheckBattery -->|Yes| Discharge[Discharge Battery]
    CheckBattery -->|No| GridDeficit[deficit_energy = abs(net_energy)]
    
    Discharge --> CalcDischarge[discharge_amount = min(<br/>abs(net_energy),<br/>level/100 × capacity)]
    CalcDischarge --> UpdateLevel2[level -= discharge_amount/capacity × 100]
    UpdateLevel2 --> AddGen[generation += discharge_amount]
    AddGen --> CalcDeficit[deficit = abs(net_energy) - discharge_amount]
    
    Surplus --> Done[Battery Update Complete]
    GridDeficit --> Done
    CalcDeficit --> Done
    NoAction --> Done
    
    style Start fill:#87CEEB
    style Charge fill:#90EE90
    style Discharge fill:#FFB347
    style Done fill:#9370DB
```

---

## Trading Price Calculation Flow

```mermaid
graph TB
    Start[Meter Config<br/>+ Trading Strategy] --> Strategy{Trading<br/>Strategy?}
    
    Strategy -->|Aggressive| Agg1[sell_price = base × 1.1-1.3]
    Strategy -->|Aggressive| Agg2[buy_price = base × 0.8-0.95]
    
    Strategy -->|Moderate| Mod1[sell_price = base × 0.95-1.15]
    Strategy -->|Moderate| Mod2[buy_price = base × 0.95-1.1]
    
    Strategy -->|Conservative| Con1[sell_price = base × 0.9-1.05]
    Strategy -->|Conservative| Con2[buy_price = base × 1.05-1.2]
    
    Agg1 --> Calc[Calculate Trading Parameters]
    Agg2 --> Calc
    Mod1 --> Calc
    Mod2 --> Calc
    Con1 --> Calc
    Con2 --> Calc
    
    Calc --> Sale[energy_for_sale = surplus × 0.8<br/>Reserve 20% buffer]
    Calc --> Need[energy_needed = deficit<br/>- battery_available]
    
    Sale --> Output[Trading Parameters<br/>in EnergyReading]
    Need --> Output
    
    style Start fill:#87CEEB
    style Strategy fill:#FFD700
    style Agg1 fill:#FF6B6B
    style Mod1 fill:#FFB347
    style Con1 fill:#90EE90
    style Output fill:#9370DB
```

---

## Multi-Channel Output Flow

```mermaid
graph TB
    Reading[EnergyReading<br/>Generated] --> Decision{Output<br/>Channels}
    
    Decision -->|Always| File[File Output<br/>JSONL append]
    
    Decision -->|If available| Kafka{Kafka<br/>Topics}
    Kafka --> KafkaMain[energy-readings<br/>Full reading]
    Kafka -->|If surplus/deficit| KafkaTrading[trading-opportunities<br/>Trading subset]
    Kafka -->|If REC eligible| KafkaREC[renewable-certificates<br/>REC subset]
    
    Decision -->|If available| Influx[InfluxDB<br/>Time-series point]
    
    Decision -->|If available| Postgres[PostgreSQL<br/>meter_readings table]
    
    Decision -->|If enabled| WS{WebSocket<br/>Broadcast}
    WS --> WSClients[All Connected<br/>Clients]
    
    File --> Stats[Update Statistics]
    KafkaMain --> Stats
    KafkaTrading --> Stats
    KafkaREC --> Stats
    Influx --> Stats
    Postgres --> Stats
    WSClients --> Stats
    
    Stats --> Complete[Reading<br/>Processed]
    
    style Reading fill:#87CEEB
    style File fill:#90EE90
    style Kafka fill:#FFB347
    style Influx fill:#FF6B6B
    style Postgres fill:#9370DB
    style WS fill:#FFD700
    style Complete fill:#32CD32
```

---

## Consumption Pattern Flow

```mermaid
graph TB
    Start[Current Hour<br/>+ Meter Config] --> UserType{User<br/>Type?}
    
    UserType -->|Consumer| ConsPeak{Time?}
    ConsPeak -->|6-9 AM<br/>5-10 PM| C1[factor = 1.4-2.0]
    ConsPeak -->|10 PM - 6 AM| C2[factor = 0.3-0.7]
    ConsPeak -->|9 AM - 5 PM| C3[factor = 0.7-1.1]
    
    UserType -->|Prosumer| ProsPeak{Time?}
    ProsPeak -->|10 AM - 3 PM<br/>Solar Peak| P1[factor = 0.6-0.9<br/>Shifted usage]
    ProsPeak -->|7-9 AM<br/>6-9 PM| P2[factor = 1.2-1.6]
    ProsPeak -->|Other| P3[factor = 0.8-1.2]
    
    UserType -->|Storage Provider| StorPeak{Time?}
    StorPeak -->|8 AM - 5 PM| S1[factor = 1.1-1.4]
    StorPeak -->|Off-hours| S2[factor = 0.7-1.0]
    
    C1 --> Calc[consumption = base × factor × gaussian(1.0, variability)]
    C2 --> Calc
    C3 --> Calc
    P1 --> Calc
    P2 --> Calc
    P3 --> Calc
    S1 --> Calc
    S2 --> Calc
    
    Calc --> Output[energy_consumed<br/>in reading]
    
    style Start fill:#87CEEB
    style UserType fill:#FFD700
    style Calc fill:#FF6B6B
    style Output fill:#9370DB
```

---

## WebSocket Real-time Broadcasting

```mermaid
sequenceDiagram
    participant S as Simulator
    participant WS as WebSocket Server
    participant C1 as Client 1
    participant C2 as Client 2
    participant CN as Client N
    
    Note over S,CN: Initialization
    S->>WS: Start WebSocket Server
    WS-->>S: Server Ready (port 8765)
    
    C1->>WS: Connect
    WS->>C1: Accept Connection
    WS-->>C1: Send Initial Readings
    
    C2->>WS: Connect
    WS->>C2: Accept Connection
    WS-->>C2: Send Initial Readings
    
    Note over S,CN: Simulation Cycle
    loop Every N seconds
        S->>S: Generate Readings (all meters)
        
        S->>WS: broadcast_reading_sync(reading)
        
        WS->>WS: Serialize to JSON
        
        par Broadcast to All Clients
            WS-->>C1: Send Reading (JSON)
            WS-->>C2: Send Reading (JSON)
            WS-->>CN: Send Reading (JSON)
        end
        
        Note over WS: Remove Disconnected Clients
        
        WS-->>S: Broadcast Complete
    end
    
    Note over S,CN: Client Disconnect
    C1->>WS: Disconnect
    WS->>WS: Remove from clients set
    
    Note over S,CN: Shutdown
    S->>WS: Stop Server
    WS->>C2: Close Connection
    WS->>CN: Close Connection
```

---

## FastAPI REST API Flow

```mermaid
sequenceDiagram
    participant Client
    participant FastAPI
    participant Simulator
    participant DB as Database
    
    Note over Client,DB: Status Request
    Client->>FastAPI: GET /api/status
    FastAPI->>Simulator: Check status
    Simulator-->>FastAPI: Status info
    FastAPI-->>Client: 200 OK + JSON
    
    Note over Client,DB: Statistics Request
    Client->>FastAPI: GET /api/stats
    FastAPI->>Simulator: Get current readings
    Simulator->>Simulator: Generate latest readings
    Simulator-->>FastAPI: Aggregated stats
    FastAPI-->>Client: 200 OK + Stats JSON
    
    Note over Client,DB: Recent Readings
    Client->>FastAPI: GET /api/readings?limit=10
    FastAPI->>Simulator: Get meter configs
    FastAPI->>Simulator: Generate readings
    Simulator-->>FastAPI: Latest 10 readings
    FastAPI-->>Client: 200 OK + Readings Array
    
    Note over Client,DB: Meters List
    Client->>FastAPI: GET /api/meters
    FastAPI->>Simulator: Get meters list
    Simulator->>DB: Query smart_meters (if available)
    DB-->>Simulator: Meter configs
    Simulator-->>FastAPI: Meters info
    FastAPI-->>Client: 200 OK + Meters Array
    
    Note over Client,DB: Health Check
    Client->>FastAPI: GET /health
    FastAPI->>Simulator: Check running
    Simulator-->>FastAPI: Status
    FastAPI-->>Client: 200 OK + Health JSON
```

---

## Service Initialization Flow

```mermaid
graph TB
    Start[Simulator.__init__] --> LoadConfig[Load Configuration<br/>from Environment Variables]
    
    LoadConfig --> InitKafka{Initialize<br/>Kafka?}
    InitKafka -->|Success| K1[✓ Kafka Available<br/>services += 1]
    InitKafka -->|Fail| K2[✗ Kafka Unavailable<br/>Log warning]
    
    K1 --> InitDB{Initialize<br/>PostgreSQL?}
    K2 --> InitDB
    
    InitDB -->|Success| D1[✓ PostgreSQL Available<br/>services += 1]
    InitDB -->|Fail| D2[✗ PostgreSQL Unavailable<br/>Log warning]
    
    D1 --> InitInflux{Initialize<br/>InfluxDB?}
    D2 --> InitInflux
    
    InitInflux -->|Success| I1[✓ InfluxDB Available<br/>services += 1]
    InitInflux -->|Fail| I2[✗ InfluxDB Unavailable<br/>Log warning]
    
    I1 --> CheckMode{Services<br/>Available?}
    I2 --> CheckMode
    
    CheckMode -->|0 services| Standalone[Set Standalone Mode<br/>File output only]
    CheckMode -->|1+ services| Integrated[Set Integrated Mode<br/>Multi-channel output]
    
    Standalone --> InitMeters[Initialize Meters<br/>From DB or Simulated]
    Integrated --> InitMeters
    
    InitMeters --> CreateDir[Create Output<br/>Directories]
    CreateDir --> Ready[Simulator Ready]
    
    style Start fill:#87CEEB
    style K1 fill:#90EE90
    style D1 fill:#90EE90
    style I1 fill:#90EE90
    style K2 fill:#FFB347
    style D2 fill:#FFB347
    style I2 fill:#FFB347
    style Standalone fill:#FFD700
    style Integrated fill:#32CD32
    style Ready fill:#9370DB
```

---

## Data Flow Summary

### Input Sources
1. **Configuration**: Environment variables
2. **Time**: System clock for time-based calculations
3. **Randomness**: Weather transitions, noise factors
4. **Database** (optional): Meter configurations

### Processing Stages
1. **Weather Simulation**: Dynamic weather state machine
2. **Solar Generation**: Physics-based calculation
3. **Consumption Pattern**: Time-of-day + user type
4. **Battery Management**: Charge/discharge logic
5. **Trading Calculation**: Strategy-based pricing
6. **REC Validation**: Eligibility determination

### Output Channels
1. **File**: Always active (JSONL format)
2. **Kafka**: 3 topics (readings, trading, REC)
3. **InfluxDB**: Time-series optimization
4. **PostgreSQL**: Relational storage
5. **WebSocket**: Real-time broadcasting

### Feedback Loops
- Weather state persists across cycles
- Battery level carries over between readings
- Statistics accumulate over time

---

**Document Version**: 1.0.0  
**Last Updated**: 2025-11-09  
**Related**: [Smart Meter Simulator Technical Documentation](../../architecture/system/SMART_METER_SIMULATOR.md)
