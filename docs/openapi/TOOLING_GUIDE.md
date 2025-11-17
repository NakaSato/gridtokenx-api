# OpenAPI Tooling Guide

Complete guide for using the GridTokenX Platform OpenAPI documentation and tooling.

## 📚 Table of Contents

- [Quick Start](#quick-start)
- [Available Scripts](#available-scripts)
- [Documentation](#documentation)
- [Client SDK Generation](#client-sdk-generation)
- [Testing](#testing)
- [Workflows](#workflows)
- [Troubleshooting](#troubleshooting)

## 🚀 Quick Start

### 1. Start the API Gateway

```bash
# Ensure PostgreSQL is running
docker-compose up -d postgres

# Start the API Gateway
cd api-gateway
cargo run
```

### 2. View Interactive API Documentation

Open your browser to: **http://localhost:8080/api/docs**

### 3. Run Complete Workflow

```bash
cd api-gateway
./scripts/openapi-workflow.sh
```

This will:
- ✅ Test the OpenAPI implementation
- ✅ Generate OpenAPI specification files
- ✅ Generate client SDKs (TypeScript & Python)
- ✅ Run integration tests

## 📜 Available Scripts

All scripts are located in `api-gateway/scripts/`:

### Core Scripts

| Script | Purpose | Usage |
|--------|---------|-------|
| `openapi-workflow.sh` | Master script - runs all steps | `./scripts/openapi-workflow.sh` |
| `test-openapi.sh` | Test OpenAPI implementation | `./scripts/test-openapi.sh` |
| `generate-openapi-spec.sh` | Extract OpenAPI spec from server | `./scripts/generate-openapi-spec.sh` |
| `check-openapi-status.sh` | Check documentation coverage | `./scripts/check-openapi-status.sh` |

### Client Generation Scripts

| Script | Purpose | Usage |
|--------|---------|-------|
| `generate-typescript-client.sh` | Generate TypeScript/Axios client | `./scripts/generate-typescript-client.sh` |
| `generate-python-client.sh` | Generate Python client | `./scripts/generate-python-client.sh` |

### Testing Scripts

| Script | Purpose | Usage |
|--------|---------|-------|
| `run-integration-tests.sh` | Run OpenAPI integration tests | `./scripts/run-integration-tests.sh` |

## 📖 Documentation

### Generated Documentation Files

| File | Description |
|------|-------------|
| `docs/openapi/openapi-spec.json` | OpenAPI 3.1 specification (JSON) |
| `docs/openapi/openapi-spec.yaml` | OpenAPI 3.1 specification (YAML) |
| `docs/openapi/API_EXAMPLES.md` | Practical API usage examples |
| `docs/openapi/IMPLEMENTATION_SUMMARY.md` | Implementation details |
| `docs/openapi/STATUS_CURRENT.md` | Documentation status |

### Interactive Documentation

- **Swagger UI**: http://localhost:8080/api/docs
- **OpenAPI JSON**: http://localhost:8080/api/docs/openapi.json

### Documentation Coverage

Check current coverage:

```bash
./scripts/check-openapi-status.sh
```

Output shows:
- ✅ Fully documented handlers
- ⚠️ Partially documented handlers
- ❌ Undocumented handlers
- Overall progress percentage

## 🔧 Client SDK Generation

### Prerequisites

Install openapi-generator:

```bash
# Via npm (recommended)
npm install -g @openapitools/openapi-generator-cli

# Or use npx (no installation needed)
npx @openapitools/openapi-generator-cli version
```

### TypeScript Client

Generate TypeScript/Axios client:

```bash
./scripts/generate-typescript-client.sh
```

**Output**: `clients/typescript/`

**Usage**:

```typescript
import { Configuration, AuthApi, TradingApi } from '@gridtokenx/api-client';

const config = new Configuration({
  basePath: 'http://localhost:8080',
  accessToken: 'your-jwt-token'
});

const authApi = new AuthApi(config);
const tradingApi = new TradingApi(config);

// Use the APIs
const marketData = await tradingApi.getMarketData();
```

See `clients/typescript/README.md` for complete usage guide.

### Python Client

Generate Python client:

```bash
./scripts/generate-python-client.sh
```

**Output**: `clients/python/`

**Usage**:

```python
import gridtokenx_api_client
from gridtokenx_api_client.api import auth_api, trading_api

configuration = gridtokenx_api_client.Configuration(
    host="http://localhost:8080"
)
configuration.access_token = "your-jwt-token"

with gridtokenx_api_client.ApiClient(configuration) as api_client:
    trading_instance = trading_api.TradingApi(api_client)
    market_data = trading_instance.get_market_data()
```

See `clients/python/README.md` for complete usage guide.

### Other Languages

Generate clients for other languages:

```bash
# Go
openapi-generator-cli generate \
  -i docs/openapi/openapi-spec.json \
  -g go \
  -o clients/go

# Java
openapi-generator-cli generate \
  -i docs/openapi/openapi-spec.json \
  -g java \
  -o clients/java

# Ruby
openapi-generator-cli generate \
  -i docs/openapi/openapi-spec.json \
  -g ruby \
  -o clients/ruby
```

See: https://openapi-generator.tech/docs/generators

## 🧪 Testing

### Integration Tests

Run integration tests against the OpenAPI specification:

```bash
./scripts/run-integration-tests.sh
```

Tests verify:
- ✅ OpenAPI spec is valid
- ✅ All documented endpoints are accessible
- ✅ Authentication flow works correctly
- ✅ Error responses match specification
- ✅ Schemas match expectations

**Test Report**: `tests/reports/openapi-test-report.html`

### Test with Coverage

```bash
./scripts/run-integration-tests.sh --coverage
```

### Manual Testing

Use Swagger UI for interactive testing:

1. Open http://localhost:8080/api/docs
2. Click "Authorize" button
3. Enter JWT token: `Bearer <your-token>`
4. Test endpoints interactively

### Testing with curl

Examples in `docs/openapi/API_EXAMPLES.md`:

```bash
# Login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password"}'

# Use token
export JWT_TOKEN="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

curl -X GET http://localhost:8080/api/auth/profile \
  -H "Authorization: Bearer $JWT_TOKEN"
```

## 🔄 Workflows

### Complete Workflow

Run all steps in sequence:

```bash
./scripts/openapi-workflow.sh
```

Interactive prompts guide you through:
1. Server startup (if needed)
2. OpenAPI spec generation
3. Client SDK generation (optional)
4. Integration testing (optional)

### Individual Steps

Run steps independently:

```bash
# Step 1: Test implementation
./scripts/test-openapi.sh

# Step 2: Generate spec
./scripts/generate-openapi-spec.sh

# Step 3: Generate TypeScript client
./scripts/generate-typescript-client.sh

# Step 4: Generate Python client  
./scripts/generate-python-client.sh

# Step 5: Run tests
./scripts/run-integration-tests.sh
```

### CI/CD Integration

Add to your CI pipeline:

```yaml
# .github/workflows/openapi.yml
name: OpenAPI Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Start server
        run: |
          cd api-gateway
          cargo run &
          sleep 10
      
      - name: Validate OpenAPI
        run: |
          cd api-gateway
          ./scripts/test-openapi.sh
      
      - name: Run integration tests
        run: |
          cd api-gateway
          ./scripts/run-integration-tests.sh
```

## 🔍 Troubleshooting

### Server Not Running

**Error**: `❌ API Gateway is not running`

**Solution**:
```bash
cd api-gateway
cargo run
```

### Database Migration Error

**Error**: `migration was previously applied but has been modified`

**Solution**:
```bash
# Reset migrations
cd api-gateway
sqlx database drop -y
sqlx database create
sqlx migrate run
```

### OpenAPI Spec Not Found

**Error**: `❌ OpenAPI spec not found`

**Solution**:
```bash
# Ensure server is running, then generate spec
./scripts/generate-openapi-spec.sh
```

### Client Generation Fails

**Error**: `openapi-generator-cli not found`

**Solution**:
```bash
# Install via npm
npm install -g @openapitools/openapi-generator-cli

# Or use npx (no installation)
npx @openapitools/openapi-generator-cli version
```

### Integration Tests Fail

**Error**: Test failures in `test_openapi.py`

**Debug**:
```bash
# Check server logs
tail -f /tmp/gridtokenx-server.log

# Verify server is responding
curl http://localhost:8080/api/health

# Run tests with verbose output
cd api-gateway
source .venv-test/bin/activate
pytest tests/integration/test_openapi.py -vv
```

### Swagger UI Not Loading

**Error**: Swagger UI shows blank page

**Solution**:
1. Check browser console for errors
2. Verify OpenAPI spec is valid: http://localhost:8080/api/docs/openapi.json
3. Clear browser cache
4. Try different browser

### Port Already in Use

**Error**: `Address already in use (os error 48)`

**Solution**:
```bash
# Find process using port 8080
lsof -ti:8080

# Kill the process
kill $(lsof -ti:8080)
```

## 📊 Metrics

### Documentation Coverage

Current: **62/62 handlers (100%)**

- ✅ Authentication & Health: 18 handlers
- ✅ Core Business Logic: 23 handlers
- ✅ Supporting Services: 14 handlers
- ✅ Testing & Real-time: 7 handlers

### Endpoint Statistics

- **Total Paths**: 62
- **Total Schemas**: 80+
- **Tags**: 12
- **Security Schemes**: JWT Bearer

## 🚀 Best Practices

### 1. Keep Documentation Updated

After adding new handlers:

```bash
# Check coverage
./scripts/check-openapi-status.sh

# Generate updated spec
./scripts/generate-openapi-spec.sh

# Regenerate clients
./scripts/generate-typescript-client.sh
./scripts/generate-python-client.sh
```

### 2. Version Control

Commit these files:
```
docs/openapi/openapi-spec.json
docs/openapi/openapi-spec.yaml
docs/openapi/API_EXAMPLES.md
clients/typescript/
clients/python/
```

### 3. API Versioning

When making breaking changes:
- Update API version in `src/openapi/mod.rs`
- Document changes in CHANGELOG
- Consider deprecation warnings
- Maintain backward compatibility when possible

### 4. Security

- Never commit JWT tokens
- Use environment variables for sensitive data
- Implement rate limiting
- Enable CORS properly
- Use HTTPS in production

## 📞 Support

- **Documentation**: http://localhost:8080/api/docs
- **Issues**: Create issue in project repository
- **Examples**: `docs/openapi/API_EXAMPLES.md`

## 📝 License

MIT License - See LICENSE file for details

---

**Last Updated**: November 10, 2025  
**OpenAPI Version**: 3.1.0  
**API Version**: 0.1.0
