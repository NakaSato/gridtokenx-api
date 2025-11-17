# OpenAPI Quick Reference

Quick reference for GridTokenX Platform OpenAPI implementation.

## 🚀 Quick Commands

```bash
# Complete Workflow (All Steps)
./scripts/openapi-workflow.sh

# Check Documentation Coverage
./scripts/check-openapi-status.sh

# Test OpenAPI Implementation
./scripts/test-openapi.sh

# Generate OpenAPI Spec Files
./scripts/generate-openapi-spec.sh

# Generate TypeScript Client
./scripts/generate-typescript-client.sh

# Generate Python Client
./scripts/generate-python-client.sh

# Run Integration Tests
./scripts/run-integration-tests.sh
```

## 📚 Key URLs

| Service | URL |
|---------|-----|
| **Swagger UI** | http://localhost:8080/api/docs |
| **OpenAPI JSON** | http://localhost:8080/api/docs/openapi.json |
| **Health Check** | http://localhost:8080/api/health |
| **API Base** | http://localhost:8080/api |

## 📁 Important Files

```
api-gateway/
├── docs/openapi/
│   ├── openapi-spec.json          # Generated OpenAPI spec (JSON)
│   ├── openapi-spec.yaml          # Generated OpenAPI spec (YAML)
│   ├── API_EXAMPLES.md            # Usage examples
│   ├── TOOLING_GUIDE.md           # Complete guide
│   ├── IMPLEMENTATION_SUMMARY.md  # Technical details
│   └── COMPLETE_SUMMARY.md        # This implementation summary
├── clients/
│   ├── typescript/                # TypeScript client SDK
│   └── python/                    # Python client SDK
├── scripts/
│   ├── openapi-workflow.sh        # Master workflow
│   ├── test-openapi.sh            # OpenAPI tests
│   ├── generate-openapi-spec.sh   # Spec generation
│   ├── generate-typescript-client.sh
│   ├── generate-python-client.sh
│   └── run-integration-tests.sh
└── tests/integration/
    └── test_openapi.py            # Integration tests
```

## 🔑 Authentication

```bash
# 1. Login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password"}'

# 2. Extract token from response
export JWT_TOKEN="eyJhbGci..."

# 3. Use token in requests
curl -X GET http://localhost:8080/api/auth/profile \
  -H "Authorization: Bearer $JWT_TOKEN"
```

## 📊 Coverage Status

- **Total Handlers**: 62/62 (100%)
- **Total Schemas**: 80+
- **Total Tags**: 12
- **OpenAPI Version**: 3.1.0

## 🎯 API Tags

1. `health` - Health checks
2. `auth` - Authentication
3. `users` - User management
4. `blockchain` - Blockchain ops
5. `blockchain-test` - Testing utils
6. `trading` - Trading operations
7. `meters` - Smart meters
8. `erc` - ERC certificates
9. `tokens` - Token operations
10. `oracle` - Price oracle
11. `governance` - Governance
12. `websocket` - Real-time data

## 🛠️ TypeScript Client

```typescript
import { Configuration, TradingApi } from '@gridtokenx/api-client';

const config = new Configuration({
  basePath: 'http://localhost:8080',
  accessToken: 'your-jwt-token'
});

const api = new TradingApi(config);
const data = await api.getMarketData();
```

## 🐍 Python Client

```python
import gridtokenx_api_client
from gridtokenx_api_client.api import trading_api

config = gridtokenx_api_client.Configuration(
    host="http://localhost:8080"
)
config.access_token = "your-jwt-token"

with gridtokenx_api_client.ApiClient(config) as client:
    api = trading_api.TradingApi(client)
    data = api.get_market_data()
```

## 🧪 Testing

```bash
# Run all integration tests
./scripts/run-integration-tests.sh

# Run with coverage
./scripts/run-integration-tests.sh --coverage

# View test report
open tests/reports/openapi-test-report.html
```

## 🔧 Troubleshooting

| Issue | Solution |
|-------|----------|
| Server not running | `cargo run` |
| Migrations error | `sqlx migrate run` |
| Port in use | `kill $(lsof -ti:8080)` |
| Spec not found | `./scripts/generate-openapi-spec.sh` |
| Client gen fails | `npm install -g @openapitools/openapi-generator-cli` |

## 📞 Support

- **Docs**: See `docs/openapi/TOOLING_GUIDE.md`
- **Examples**: See `docs/openapi/API_EXAMPLES.md`
- **Issues**: Check compilation errors with `cargo check`

## ✅ Checklist

- [ ] Server running at http://localhost:8080
- [ ] PostgreSQL running
- [ ] OpenAPI spec generated
- [ ] Clients generated (if needed)
- [ ] Tests passing
- [ ] Documentation reviewed

---

**Last Updated**: November 10, 2025  
**Status**: Production Ready ✅
