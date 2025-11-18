# Phase 3: User Authentication, Wallet Signing & Transaction Flow

## 📖 Overview

Phase 3 implements the complete user journey from registration to executing trades on the blockchain. This includes:

1. **User Registration/Login** with integrated wallet creation
2. **Wallet Management** and blockchain signing
3. **Complete Transaction Flow** for trading orders and settlements
4. **Real-time Updates** via WebSocket

## 🚀 Quick Start

```bash
# 1. Start the environment
make solana-start        # Terminal 1
cd api-gateway && cargo run  # Terminal 2

# 2. Test the flow
./scripts/test-wallet-auth.sh

# 3. View API documentation
open http://localhost:8080/api/docs
```

## 📚 Documentation

### Main Documents

| Document | Purpose | Audience |
|----------|---------|----------|
| **[PHASE3_USER_WALLET_TRANSACTION_FLOW.md](PHASE3_USER_WALLET_TRANSACTION_FLOW.md)** | Complete technical specification | All developers |
| **[PHASE3_FRONTEND_SPECIFICATIONS.md](PHASE3_FRONTEND_SPECIFICATIONS.md)** | Frontend requirements & UI/UX specs | Frontend developers |
| **[PHASE3_API_QUICK_REFERENCE.md](PHASE3_API_QUICK_REFERENCE.md)** | API endpoints quick reference | Frontend developers |
| **[PHASE3_IMPLEMENTATION_GUIDE.md](PHASE3_IMPLEMENTATION_GUIDE.md)** | Quick reference for implementation | Frontend/Backend devs |

### Architecture Diagrams

- **[Sequence Diagram](../../diagrams/sequence/phase3_complete_user_flow.puml)** - Complete user flow visualization

### Related Documentation

- [API Gateway Authentication Guide](../../api-gateway/docs/AUTHENTICATION_GUIDE.md)
- [Phase 5: Trading Implementation](PHASE5_TRADING_IMPLEMENTATION_COMPLETE.md)
- [Testing Guide](../../tests/README.md)

## 🎯 Key Features

### ✅ Completed (Backend)

- User registration with automatic wallet creation
- Email verification system
- JWT authentication
- Wallet-integrated login
- Meter reading submission
- Energy token minting
- Trading order creation
- Order matching engine
- Settlement processing
- WebSocket service for real-time updates

### ⏳ In Progress (Frontend)

- Registration UI component
- Login form
- Wallet display component
- Trading dashboard
- Order book visualization
- Transaction history view
- Real-time notifications

## 🔐 Authentication Flow

```
Registration
  → Email verification
  → Wallet creation (optional)
  → JWT token issued

Login
  → Password verification
  → Wallet info retrieved
  → JWT token issued
  → Redirect to dashboard
```

## 💱 Trading Flow

```
Meter Reading
  → Submit energy production data
  → Mint energy tokens (1 token = 1 kWh)

Create Order
  → Validate balance
  → Submit to blockchain
  → Add to order book

Matching
  → Market clearing engine (15 min epochs)
  → Match buy/sell orders
  → Execute settlements

Confirmation
  → Monitor blockchain
  → Update database
  → Send WebSocket notification
```

## 📊 API Endpoints

### Authentication
- `POST /api/auth/register` - Standard registration
- `POST /api/auth/wallet/register` - Registration with wallet
- `POST /api/auth/login` - Standard login
- `POST /api/auth/wallet/login` - Login with wallet info
- `GET /api/auth/verify-email` - Email verification

### Wallet Management
- `POST /api/user/wallet` - Link wallet address
- `DELETE /api/user/wallet` - Remove wallet address

### Trading
- `POST /api/trading/orders` - Create order
- `GET /api/trading/orders` - Get user orders
- `GET /api/trading/order-book` - Current order book
- `GET /api/trading/history` - Trading history

### Tokens
- `POST /api/tokens/mint-from-reading` - Mint from meter reading
- `GET /api/tokens/balance/{wallet}` - Get token balance

### Meters
- `POST /api/meters/submit-reading` - Submit reading
- `GET /api/meters/my-readings` - Get user readings

## 🧪 Testing

### Run Tests
```bash
# Backend tests
cd api-gateway
cargo test

# Integration tests
cd tests
npm run test:integration

# Manual testing
./scripts/test-wallet-auth.sh
```

### Test Checklist
- [ ] Register with wallet creation
- [ ] Login and see wallet info
- [ ] Submit meter reading
- [ ] Mint tokens
- [ ] Create buy order
- [ ] Create sell order
- [ ] Orders match automatically
- [ ] Settlement executes
- [ ] WebSocket updates received
- [ ] Transaction history displays

## 📈 Implementation Status

### Backend: 90% Complete
- ✅ Authentication system
- ✅ Wallet service
- ✅ Trading engine
- ✅ Order matching
- ✅ Settlement processing
- ✅ WebSocket service
- ⏳ Production optimizations

### Frontend: 30% Complete
- ⏳ Registration component
- ⏳ Login component
- ⏳ Wallet display
- ⏳ Trading dashboard
- ⏳ Order book UI
- ⏳ Transaction history
- ⏳ Real-time notifications

## 🎨 Frontend Examples

### Registration Component
```tsx
const handleRegister = async (formData) => {
  const response = await fetch('/api/auth/wallet/register', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      ...formData,
      create_wallet: true,
      airdrop_amount: 2.0
    })
  });
  
  const { access_token, wallet_info } = await response.json();
  localStorage.setItem('token', access_token);
  localStorage.setItem('wallet', wallet_info.address);
};
```

### Trading Component
```tsx
const createOrder = async (orderData) => {
  const token = localStorage.getItem('token');
  const wallet = localStorage.getItem('wallet');
  
  const response = await fetch('/api/trading/orders', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      ...orderData,
      wallet_address: wallet
    })
  });
  
  return await response.json();
};
```

### WebSocket Hook
```tsx
const useWebSocket = (url: string) => {
  const [messages, setMessages] = useState([]);
  
  useEffect(() => {
    const token = localStorage.getItem('token');
    const ws = new WebSocket(`${url}?token=${token}`);
    
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      setMessages(prev => [...prev, data]);
      
      if (data.type === 'order_matched') {
        toast.success('Your order has been matched!');
      }
    };
    
    return () => ws.close();
  }, [url]);
  
  return { messages };
};
```

## 🔧 Common Commands

```bash
# Start development
make dev-full

# Run API gateway
cd api-gateway && cargo run

# View logs
RUST_LOG=debug cargo run

# Test endpoints
curl http://localhost:8080/health
curl http://localhost:8080/api/docs

# Database
psql postgresql://user:pass@localhost/gridtokenx
```

## 🐛 Troubleshooting

### Solana RPC not available
```bash
make solana-stop
make solana-start
```

### JWT token expired
```bash
# Login again to get new token
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "alice", "password": "SecurePass123!"}'
```

### Database issues
```bash
cd api-gateway
sqlx database reset
```

## 📊 Success Metrics

### Technical KPIs
- ✅ Registration success: > 99%
- ✅ Login response time: < 200ms
- ✅ Wallet creation: < 2s
- ⏳ Transaction confirmation: < 10s
- ⏳ Order matching: < 100ms
- ⏳ WebSocket latency: < 50ms

### User Experience
- ⏳ Registration completion: > 90%
- ⏳ Wallet connection success: > 95%
- ⏳ Order placement success: > 99%
- ⏳ User satisfaction: > 4.5/5

## 🚀 Next Steps

### This Week
1. [ ] Create frontend registration component
2. [ ] Implement browser wallet adapter (Phantom)
3. [ ] Add transaction monitoring dashboard
4. [ ] Set up real-time WebSocket notifications

### Next 2 Weeks
1. [ ] Complete frontend trading UI
2. [ ] Implement order book visualization
3. [ ] Add transaction history view
4. [ ] Deploy to staging environment

### Next Month
1. [ ] Production deployment
2. [ ] User acceptance testing
3. [ ] Performance optimization
4. [ ] Security audit

## 📞 Support

- **Documentation**: See links above
- **API Docs**: http://localhost:8080/api/docs
- **Issues**: GitHub Issues
- **Chat**: Team Slack/Discord

---

**Status**: 🔄 IN PROGRESS (Backend: 90%, Frontend: 30%)  
**Target Completion**: November 30, 2025  
**Priority**: HIGH

---

*Last Updated: November 13, 2025*  
*Phase Owner: GridTokenX Development Team*
