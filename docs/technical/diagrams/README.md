# Diagrams Documentation

## Overview

Visual documentation for the GridTokenX platform using industry-standard notation (PlantUML, C4 Model).

## Structure

### 📁 sequence/
Sequence diagrams showing interactions over time
- User registration flows
- Energy generation and token minting
- Trading workflows
- Market clearing processes

### 📁 component/
Component diagrams using C4 Model
- System context (Level 1)
- Container diagrams (Level 2)
- Component details (Level 3)
- Code-level views (Level 4)

### 📁 deployment/
Deployment and infrastructure diagrams
- Docker compose setup
- Network topology
- Service dependencies
- Monitoring stack

### 📁 flow/
Data flow diagrams (DFDs)
- Level 0: Context diagram
- Level 1: Major processes
- Level 2: Detailed flows
- CRUD matrices

## Diagram Standards

### PlantUML Conventions
```plantuml
- Use pastel colors (#C8E6C9, #BBDEFB, etc.)
- Bold headers in notes: <b>Header</b>
- Italic annotations: <i>context</i>
- Consistent participant naming
- Helvetica font, 12pt
```

### File Naming
- Sequence: `STEP_N_process-name.puml`
- Component: `C4_LEVEL_N_component.puml`
- Flow: `DFD_LEVEL_N_process.puml`
- Deployment: `DEPLOY_service-name.puml`

## Key Diagrams

### Sequence Diagrams
- `sequence/STEP_1_REGISTRATION.puml` - User onboarding
- `sequence/STEP_2_ENERGY_GENERATION.puml` - Token minting
- `sequence/STEP_3_ENERGY_TRADING.puml` - Order placement
- `sequence/STEP_4_MARKET_CLEARING.puml` - Settlement
- `sequence/ARCHITECTURE_OVERVIEW.puml` - Complete flow

### Component Diagrams
- `component/C4_LEVEL_1_SYSTEM_CONTEXT.puml` - System boundaries
- `component/C4_LEVEL_2_CONTAINERS.puml` - High-level components
- `component/C4_LEVEL_3_COMPONENTS_*.puml` - Detailed components

### Flow Diagrams
- `flow/DFD_LEVEL_0.puml` - Context diagram
- `flow/DFD_LEVEL_1.puml` - Major processes
- `flow/DFD_LEVEL_2_*.puml` - Detailed subsystems

## Rendering

### VS Code
Install PlantUML extension:
```bash
code --install-extension jebbs.plantuml
```

### Command Line
```bash
plantuml *.puml
```

### Online
- PlantUML Server: http://www.plantuml.com/plantuml
- C4 Model: https://c4model.com

## For LLMs

### Context Loading
```
# Load overview first
cat sequence/ARCHITECTURE_OVERVIEW.puml

# Then specific flows
cat sequence/STEP_1_REGISTRATION.puml
cat sequence/STEP_2_ENERGY_GENERATION.puml
cat sequence/STEP_3_ENERGY_TRADING.puml
cat sequence/STEP_4_MARKET_CLEARING.puml
```

### Understanding Diagrams
- Participants represent system components
- Arrows show interactions and data flow
- Notes provide context and business rules
- Activation bars show when components are active

## Related Documentation
- Architecture: `/technical/architecture/`
- Specifications: `/technical/specifications/processes/`
- Guides: `/technical/guides/development/`

---
**Category**: Diagrams  
**Last Updated**: 2025-11-08
