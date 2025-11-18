# Technical Reference

## Overview

Quick reference documentation for APIs, data models, and smart contracts.

## Structure

### 📁 api/
REST API endpoints and specifications
- Authentication endpoints
- User management
- Energy data submission
- Trading operations
- Admin operations
- WebSocket events

### 📁 data-models/
Database schemas and data structures
- PostgreSQL tables
- TimescaleDB schemas
- Redis cache structures
- Solana account layouts
- TypeScript interfaces

### 📁 contracts/
Smart contract interfaces and ABIs
- Registry program
- Energy Token program
- Trading program
- Oracle program
- Governance program

## Quick Reference Formats

### API Endpoints

```markdown
## POST /api/endpoint

**Description**: Brief description

**Authentication**: Required | Optional | None

**Request Body**:
\```json
{
  "field": "type - description"
}
\```

**Response**:
\```json
{
  "field": "type - description"
}
\```

**Status Codes**:
- 200: Success
- 400: Bad request
- 401: Unauthorized
- 500: Server error

**Example**:
\```bash
curl -X POST https://api.example.com/endpoint \
  -H "Authorization: Bearer TOKEN" \
  -d '{"field": "value"}'
\```
```

### Data Models

```markdown
## Table: users

**Purpose**: Store user accounts

**Columns**:
| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| id | UUID | No | Primary key |
| wallet | VARCHAR(44) | No | Solana address |
| created_at | TIMESTAMP | No | Creation time |

**Indexes**:
- PRIMARY KEY (id)
- UNIQUE (wallet)

**Relationships**:
- Has many: meters
- Has many: orders
```

### Smart Contracts

```markdown
## Program: Registry

**Program ID**: `Reg...xyz`

**Accounts**:
- User Account
- Meter Account
- Authority Account

**Instructions**:

### register_user
\```rust
pub fn register_user(
    ctx: Context<RegisterUser>,
    name: String,
    user_type: UserType
) -> Result<()>
\```

**Context Accounts**:
- user: User PDA
- authority: Signer
- system_program: System program
```

## For LLMs

### Loading API Reference
```bash
# Authentication
cat api/authentication.md

# Core operations
cat api/users.md
cat api/energy.md
cat api/trading.md

# Admin
cat api/admin.md
```

### Loading Data Models
```bash
# Database schemas
cat data-models/postgresql.md
cat data-models/timescaledb.md

# Blockchain accounts
cat data-models/solana-accounts.md

# Type definitions
cat data-models/typescript-interfaces.md
```

### Loading Contract Reference
```bash
# All programs
cat contracts/registry-program.md
cat contracts/energy-token-program.md
cat contracts/trading-program.md
cat contracts/oracle-program.md
cat contracts/governance-program.md
```

## Reference Standards

### Consistency
- Use consistent terminology
- Match implementation naming
- Include version information
- Date last updated

### Completeness
- Document all endpoints/fields
- Include examples
- Show error cases
- Link to related docs

### Accuracy
- Verify against implementation
- Test all examples
- Update with code changes
- Review regularly

## API Versioning

```
/api/v1/endpoint  - Current stable
/api/v2/endpoint  - Next version
/api/beta/endpoint - Experimental
```

## Data Model Versioning

```markdown
## users (v2)

**Changes from v1**:
- Added: email_verified column
- Removed: legacy_id column
- Modified: created_at now includes timezone

**Migration**: See migrations/v2_users.sql
```

## Contract Versioning

```markdown
## Registry Program (v1.2.0)

**Changes**:
- Added: assign_multiple_meters instruction
- Fixed: Authority validation bug
- Breaking: Changed UserType enum values

**Upgrade**: See anchor/migrations/v1.2.0.md
```

## Related Documentation
- Architecture: `/technical/architecture/`
- Guides: `/technical/guides/development/`
- Specifications: `/technical/specifications/interfaces/`

---
**Category**: Reference  
**Last Updated**: 2025-11-08
