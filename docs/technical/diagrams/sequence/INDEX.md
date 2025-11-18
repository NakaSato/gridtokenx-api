# Sequence Diagrams Index

## Overview

Sequence diagrams showing step-by-step interactions and data flows in the GridTokenX platform.

## Complete Flow Diagrams

### ARCHITECTURE_OVERVIEW_SEQUENCE.puml
**Description**: Comprehensive system architecture showing all major data flows  
**Components**: All system components from frontend to blockchain  
**Use Case**: Understanding complete system interactions  
**Status**: ✅ Updated and validated

## Step-by-Step Process Flows

### STEP_1_REGISTRATION.puml
**Title**: REC Registration Flow  
**Description**: User onboarding and smart meter assignment process  
**Actors**: REC, Prosumer/Consumer  
**Key Components**: API Gateway, Registry Program, PostgreSQL, Blockchain  
**Process**:
1. User registration with wallet
2. Meter assignment by REC

**Status**: ✅ Updated with pastel colors and LLM-optimized notes

### STEP_2_ENERGY_GENERATION.puml
**Title**: Energy Generation and SPL Token Minting Flow  
**Description**: Smart meter data collection and token issuance  
**Actors**: REC, Prosumer/Consumer  
**Key Components**: Smart Meter Simulator, Oracle Program, Energy Token Program, TimescaleDB  
**Process**:
1. AMI data collection (15-min intervals)
2. Oracle validation
3. Token minting (kWh × 10^9)

**Status**: ✅ Updated with improved styling

### STEP_3_ENERGY_TRADING.puml
**Title**: Energy Trading (15-min Epochs) Flow  
**Description**: Peer-to-peer energy trading and order matching  
**Actors**: Seller (Prosumer), Buyer (Consumer)  
**Key Components**: API Gateway, Trading Program, PostgreSQL Orders Table  
**Process**:
1. Seller creates sell order
2. Buyer creates buy order
3. Orders collected in order book

**Status**: ✅ Updated with pastel colors

### STEP_4_MARKET_CLEARING.puml
**Title**: Automated Market Clearing (15-min) Flow  
**Description**: Automated order matching and settlement  
**Actors**: Seller, Buyer  
**Key Components**: Epoch Timer, Oracle Program, Trading Program, Token Program  
**Process**:
1. Epoch timer triggers clearing
2. Order matching algorithm
3. Token transfers
4. Settlement confirmation

**Status**: ✅ Updated with consistent styling

## Diagram Standards

### Visual Style
- **Colors**: Pastel shades for better readability
- **Font**: Helvetica, 12pt
- **Notes**: Bold headers, italic context
- **Actors**: Color-coded by role

### Color Scheme
```
Sellers/REC:        #C8E6C9 (light green)
Buyers/Consumers:   #BBDEFB (light blue)
API Gateway:        #E1F5FE (light cyan)
Trading Programs:   #FFCCBC (light coral)
Token Programs:     #E1BEE7 (light purple)
Blockchain:         #F8BBD0 (light pink)
Databases:          #FFF9C4 (light yellow)
```

## Usage

### For LLMs
```bash
# Load complete flow first
cat ARCHITECTURE_OVERVIEW_SEQUENCE.puml

# Then load step-by-step
cat STEP_1_REGISTRATION.puml
cat STEP_2_ENERGY_GENERATION.puml
cat STEP_3_ENERGY_TRADING.puml
cat STEP_4_MARKET_CLEARING.puml
```

### For Developers
1. Start with ARCHITECTURE_OVERVIEW for big picture
2. Follow STEP_1 through STEP_4 for detailed flows
3. Reference specific steps during implementation

### Rendering
```bash
# Using PlantUML CLI
plantuml *.puml

# Using VS Code extension
# Install: jebbs.plantuml
# Right-click diagram > Preview
```

## Related Documentation

- **Architecture**: `/technical/architecture/blockchain/`
- **Specifications**: `/technical/specifications/processes/`
- **Guides**: `/technical/guides/development/`
- **Flow Diagrams**: `/technical/diagrams/flow/`

## Notes

- All diagrams use PlantUML format
- Special characters replaced with text equivalents
- All diagrams validated and rendering correctly
- Last updated: 2025-11-08

---
**Category**: Diagrams > Sequence  
**Count**: 5 diagrams  
**Status**: ✅ Migration Complete
