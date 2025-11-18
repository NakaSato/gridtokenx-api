# Quick Test Guide - GridTokenX Anchor Programs

## TL;DR - Run Tests Now

### One-Command Setup & Test
```bash
cd gridtokenx-app/anchor
anchor test
```

### Run Tests with Existing Validator
```bash
# Terminal 1
solana-test-validator

# Terminal 2
cd gridtokenx-app/anchor
anchor test --skip-local-validator
```

## Running Specific Tests

### Run Single Program Tests
```bash
anchor test --run-only oracle          # Oracle program tests
anchor test --run-only registry        # Registry program tests
anchor test --run-only trading         # Trading program tests
anchor test --run-only energy-token    # Energy Token program tests
anchor test --run-only governance      # Governance program tests
```

### Run with Verbose Output
```bash
anchor test -- --nocapture
```

### Run with Custom Timeout
```bash
anchor test --timeout 120
```

## Test Files Overview

| File | Tests | Purpose |
|------|-------|---------|
| **oracle.test.ts** | 10 | Meter readings & market clearing |
| **registry.test.ts** | 15 | User & meter management |
| **trading.test.ts** | 19 | P2P energy trading orders |
| **energy-token.test.ts** | 18 | Token transfers & burning |
| **governance.test.ts** | 21 | PoA governance & ERCs |

**Total: 83 tests**

## What Gets Tested

### ✅ Oracle Program (10 tests)
- Meter reading submission
- Market clearing triggers
- API Gateway authorization
- Status management
- Error handling

### ✅ Registry Program (15 tests)
- User registration (Prosumer/Consumer)
- Meter registration (Solar/Wind/Battery/Grid)
- Status updates
- Meter reading recording
- User/Meter validation

### ✅ Trading Program (19 tests)
- Sell order creation
- Buy order creation
- Order matching
- Order cancellation
- Market parameters
- Complete trading workflows

### ✅ Energy Token Program (18 tests)
- Token initialization
- REC validator management
- Token transfers
- Token burning
- Supply tracking
- Error scenarios

### ✅ Governance Program (21 tests)
- PoA initialization
- Emergency pause/unpause
- ERC certificate issuance
- ERC validation for trading
- Maintenance mode
- Configuration updates
- Statistics tracking

## Pre-Flight Checklist

```bash
# 1. Install dependencies
pnpm install

# 2. Build programs
pnpm run anchor-build

# 3. Generate TypeScript types
pnpm run codama:js

# 4. Check Anchor version (should be >= 0.29.0)
anchor --version

# 5. Check Solana CLI is installed
solana --version

# 6. Create wallet if needed
solana-keygen new -o ~/.config/solana/id.json
```

## Troubleshooting Quick Fixes

### "Program not deployed"
```bash
anchor build
anchor deploy
```

### "Keypair not found"
```bash
solana-keygen new -o ~/.config/solana/id.json
solana airdrop 10 -u localhost
```

### Tests timeout
```bash
anchor test --timeout 120 --skip-local-validator
```

### Local validator issues
```bash
# Kill existing validator
pkill solana-test-validator

# Start fresh
solana-test-validator
```

### See detailed logs
```bash
anchor test -- --nocapture 2>&1 | tee test-output.log
```

## Development Workflow

### Watch Mode (Keep validator running)
```bash
# Terminal 1: Start validator
solana-test-validator

# Terminal 2: Run tests repeatedly
while true; do
  anchor test --skip-local-validator
  sleep 5
done
```

### Debug Single Test
```bash
# Edit test file, add console.log statements
# Run with:
anchor test --run-only program-name -- --nocapture
```

### Add New Tests
1. Edit `/anchor/tests/program-name.test.ts`
2. Add new `describe` or `it` block
3. Run: `anchor test --run-only program-name`

## Expected Output

```
 PASS  oracle.test.ts
 PASS  registry.test.ts
 PASS  trading.test.ts
 PASS  energy-token.test.ts
 PASS  governance.test.ts

Tests:  83 passed (83)
Duration: ~65s
```

## Environment Variables

```bash
# Optional - Set custom provider URL
export ANCHOR_PROVIDER_URL=http://localhost:8899

# Optional - Set custom wallet
export ANCHOR_WALLET=~/.config/solana/id.json

# Optional - Set network (localnet, devnet, testnet, mainnet)
export SOLANA_NETWORK=localnet
```

## Performance Tips

- **First run**: Slower due to compilation (~1-2 minutes)
- **Subsequent runs**: Faster (~1 minute with validator running)
- **Use `--skip-local-validator`** if validator is already running
- **Parallel tests**: Run with `--jobs` flag if supported

## Test Structure Example

```typescript
describe("Program Name", () => {
  // Setup once
  before(async () => {
    [pda, bump] = await PublicKey.findProgramAddressSync(
      [Buffer.from("seed")],
      program.programId
    );
  });

  describe("Feature Group", () => {
    it("should do something", async () => {
      // Arrange
      const data = { /* test data */ };
      
      // Act
      const tx = await program.methods
        .method(data)
        .accounts({ /* accounts */ })
        .rpc();
      
      // Assert
      const account = await program.account.Type.fetch(pda);
      expect(account.field).to.equal(data.value);
    });
  });
});
```

## Common Commands Cheat Sheet

```bash
# Build only
anchor build

# Build and deploy
anchor deploy

# Build and test
anchor test

# Test specific program
anchor test --run-only trading

# Test without rebuilding
anchor test --skip-build

# Test without starting validator
anchor test --skip-local-validator

# See program logs
anchor test -- --nocapture

# Generate types from IDL
anchor idl fetch <PROGRAM_ID> -o <FILENAME>

# Check program ID matches
anchor keys list
```

## File Locations

```
gridtokenx-app/
├── anchor/
│   ├── tests/
│   │   ├── oracle.test.ts           (10 tests)
│   │   ├── registry.test.ts         (15 tests)
│   │   ├── trading.test.ts          (19 tests)
│   │   ├── energy-token.test.ts     (18 tests)
│   │   ├── governance.test.ts       (21 tests)
│   │   └── README.md                (Full documentation)
│   ├── programs/
│   │   ├── oracle/
│   │   ├── registry/
│   │   ├── trading/
│   │   ├── energy-token/
│   │   └── governance/
│   └── Anchor.toml
└── ANCHOR_TESTS_SUMMARY.md          (Test overview)
```

## Next Steps

1. **Read detailed docs**: `anchor/tests/README.md`
2. **View test summary**: `ANCHOR_TESTS_SUMMARY.md`
3. **Run tests**: `anchor test`
4. **Add your tests**: Edit test files and run `anchor test`
5. **Debug failures**: Use `-- --nocapture` flag for logs

## Support & Documentation

- **Anchor Docs**: https://docs.rs/anchor-lang/
- **Solana Docs**: https://docs.solana.com/
- **This Project**: See `MASTER_DOCUMENTATION.md`
- **Test Details**: See `anchor/tests/README.md`

---

**Happy Testing! 🧪**