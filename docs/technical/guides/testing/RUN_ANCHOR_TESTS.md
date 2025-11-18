# Running Anchor Tests - Complete Guide

## Overview

The GridTokenX project includes 83 comprehensive tests across 5 Anchor programs. This guide provides step-by-step instructions to run these tests successfully.

## Prerequisites

Before running tests, ensure you have:
- Node.js >= 18.0.0
- Rust and Cargo
- Solana CLI
- Anchor CLI >= 0.29.0
- pnpm package manager

## Quick Start (3 Steps)

### Step 1: Set Up Environment

```bash
cd gridtokenx-app

# Install dependencies
pnpm install

# Build Anchor programs
pnpm run anchor-build

# Generate TypeScript types from IDLs
pnpm run codama:js
```

### Step 2: Start Local Validator (Terminal 1)

```bash
# Start a local Solana test validator
solana-test-validator

# Keep this running in a separate terminal!
# You should see output like:
# Ledger location: /tmp/test-ledger
# Log: /tmp/test-ledger/validator.log
# ⠋ Initializing...
```

### Step 3: Run Tests (Terminal 2)

```bash
cd gridtokenx-app/anchor

# Run all 83 tests
anchor test --skip-local-validator

# Or run specific program tests:
anchor test --skip-local-validator --run-only oracle
anchor test --skip-local-validator --run-only registry
anchor test --skip-local-validator --run-only trading
anchor test --skip-local-validator --run-only energy-token
anchor test --skip-local-validator --run-only governance
```

## Detailed Instructions

### Option A: Run All Tests at Once (Simplest)

```bash
cd gridtokenx-app/anchor
anchor test
```

This will:
1. Automatically build the programs
2. Start a local validator
3. Deploy programs
4. Run all 83 tests
5. Clean up

**Time:** ~2-3 minutes (first run takes longer due to compilation)

### Option B: Run Tests with Existing Validator (Recommended for Development)

**Terminal 1 - Start Validator:**
```bash
solana-test-validator
```

**Terminal 2 - Run Tests:**
```bash
cd gridtokenx-app/anchor
anchor test --skip-local-validator
```

**Advantages:**
- Validator stays running for multiple test runs
- Faster subsequent test runs
- Better for debugging
- Can keep logs visible

### Option C: Run Individual Program Tests

Test a specific program (useful for development):

```bash
cd gridtokenx-app/anchor

# Oracle tests (10 tests)
anchor test --skip-local-validator --run-only oracle

# Registry tests (15 tests)
anchor test --skip-local-validator --run-only registry

# Trading tests (19 tests)
anchor test --skip-local-validator --run-only trading

# Energy Token tests (18 tests)
anchor test --skip-local-validator --run-only energy-token

# Governance tests (21 tests)
anchor test --skip-local-validator --run-only governance
```

### Option D: Run with Verbose Output

See detailed program logs:

```bash
cd gridtokenx-app/anchor
anchor test --skip-local-validator -- --nocapture
```

## Validator Setup Guide

### Starting a Local Validator

**Method 1: Using solana-test-validator (Recommended)**

```bash
# Start validator with default settings
solana-test-validator

# Or with custom port
solana-test-validator --rpc-port 8899 --ws-toggle-rpc-ext

# Or verbose output
solana-test-validator --verbose
```

**Method 2: Kill and Restart**

If validator isn't responding:

```bash
# Kill existing process
pkill solana-test-validator

# Wait a few seconds
sleep 3

# Start fresh
solana-test-validator
```

### Checking Validator Status

In another terminal:

```bash
# Check if validator is running
solana cluster-version -u localhost

# Get current balance
solana balance -u localhost

# Check logs
tail -f /tmp/test-ledger/validator.log
```

## Expected Test Output

### Successful Run

```
Running test suite: "/Users/user/gridtokenx-app/anchor/tests"

 ✓ oracle.test.ts (10 tests)
 ✓ registry.test.ts (15 tests)
 ✓ trading.test.ts (19 tests)
 ✓ energy-token.test.ts (18 tests)
 ✓ governance.test.ts (21 tests)

Tests:  83 passed (83)
Duration: ~65s
```

### Test Statistics

| Program | Tests | Expected Time |
|---------|-------|---------------|
| Oracle | 10 | ~8s |
| Registry | 15 | ~12s |
| Trading | 19 | ~15s |
| Energy Token | 18 | ~12s |
| Governance | 21 | ~18s |
| **TOTAL** | **83** | **~65s** |

## Troubleshooting

### Error: "Connection refused"

**Problem:** Validator isn't running or not accessible

**Solution:**
```bash
# Check if validator is running
solana-test-validator

# Or if already running, check port
netstat -an | grep 8899

# Kill and restart
pkill solana-test-validator
sleep 3
solana-test-validator
```

### Error: "Program not deployed"

**Problem:** Programs haven't been deployed to local validator

**Solution:**
```bash
cd gridtokenx-app/anchor
anchor build
anchor deploy
```

### Error: "Keypair not found"

**Problem:** Solana wallet isn't set up

**Solution:**
```bash
# Create new keypair
solana-keygen new -o ~/.config/solana/id.json

# Airdrop SOL to wallet
solana airdrop 10 -u localhost

# Check balance
solana balance -u localhost
```

### Error: "Test timeout"

**Problem:** Tests are taking too long or hanging

**Solution:**
```bash
# Increase timeout
anchor test --skip-local-validator --timeout 120

# Or run with verbose output to see what's happening
anchor test --skip-local-validator -- --nocapture
```

### Error: "Account not found"

**Problem:** Test data accounts weren't initialized properly

**Solution:**
```bash
# Try rebuilding everything
cd gridtokenx-app
pnpm run anchor-build

# Kill validator and start fresh
pkill solana-test-validator
sleep 3
solana-test-validator

# Then run tests again
cd anchor
anchor test --skip-local-validator
```

## Advanced Testing

### Running Tests Multiple Times

```bash
# Keep validator running and run tests repeatedly
while true; do
  anchor test --skip-local-validator
  echo "Test run complete. Waiting 5 seconds..."
  sleep 5
done
```

### Testing Specific Test Suite

Edit a test file and run:

```bash
# Run with verbose logging
RUST_LOG=debug anchor test --skip-local-validator -- --nocapture

# Or with specific filter
anchor test --skip-local-validator -- --grep "should initialize"
```

### Debugging with Logs

```bash
# Run tests and save output
anchor test --skip-local-validator -- --nocapture 2>&1 | tee test-output.log

# View later
cat test-output.log
```

## Environment Variables

```bash
# Set custom provider URL
export ANCHOR_PROVIDER_URL=http://localhost:8899

# Set custom wallet
export ANCHOR_WALLET=~/.config/solana/id.json

# Enable debug logging
export RUST_LOG=debug

# Use devnet instead of localnet (not recommended for tests)
export SOLANA_NETWORK=devnet
```

## File Locations

```
gridtokenx-app/
├── anchor/
│   ├── tests/
│   │   ├── oracle.test.ts
│   │   ├── registry.test.ts
│   │   ├── trading.test.ts
│   │   ├── energy-token.test.ts
│   │   ├── governance.test.ts
│   │   └── README.md
│   ├── programs/
│   │   ├── oracle/src/lib.rs
│   │   ├── registry/src/lib.rs
│   │   ├── trading/src/lib.rs
│   │   ├── energy-token/src/lib.rs
│   │   └── governance/src/lib.rs
│   ├── Anchor.toml
│   └── Cargo.toml
├── ANCHOR_TESTS_SUMMARY.md
├── RUN_ANCHOR_TESTS.md (this file)
└── QUICK_TEST_GUIDE.md
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Run Anchor Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - uses: pnpm/action-setup@v2
      
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'pnpm'
      
      - name: Install dependencies
        run: |
          pnpm install
          pnpm run anchor-build
      
      - name: Run Anchor tests
        run: |
          cd anchor
          anchor test --skip-local-validator
```

## Performance Optimization

### First Run
- Compilation takes 1-2 minutes
- Program deployment takes ~30 seconds
- Tests run in ~65 seconds
- **Total: ~2-3 minutes**

### Subsequent Runs (with validator running)
- No compilation needed
- Programs already deployed
- Tests run in ~65 seconds
- **Total: ~1-2 minutes**

### Tips for Faster Testing
1. Keep validator running between test runs
2. Use `--run-only` to test specific programs
3. Use `--skip-local-validator` flag
4. Comment out tests you're not currently working on

## Next Steps

1. **Read detailed documentation:**
   - `anchor/tests/README.md` - Full test documentation
   - `ANCHOR_TESTS_SUMMARY.md` - Test statistics
   - `QUICK_TEST_GUIDE.md` - Quick reference

2. **Explore test files:**
   - `anchor/tests/oracle.test.ts` - Oracle program tests
   - `anchor/tests/registry.test.ts` - Registry program tests
   - `anchor/tests/trading.test.ts` - Trading program tests
   - `anchor/tests/energy-token.test.ts` - Energy Token tests
   - `anchor/tests/governance.test.ts` - Governance tests

3. **Add your own tests:**
   - Follow the test structure in existing files
   - Run individual test suites as you develop
   - Check `anchor/tests/README.md` for patterns

## Quick Reference Commands

```bash
# Setup
cd gridtokenx-app
pnpm install
pnpm run anchor-build

# Terminal 1: Start validator
solana-test-validator

# Terminal 2: Run tests
cd anchor
anchor test --skip-local-validator

# Individual program tests
anchor test --skip-local-validator --run-only oracle
anchor test --skip-local-validator --run-only registry
anchor test --skip-local-validator --run-only trading
anchor test --skip-local-validator --run-only energy-token
anchor test --skip-local-validator --run-only governance

# Verbose output
anchor test --skip-local-validator -- --nocapture

# Clean and rebuild
anchor clean
anchor build

# Deploy programs
anchor deploy

# Check validator status
solana cluster-version -u localhost
solana balance -u localhost
```

## Support & Documentation

- **Anchor Docs:** https://docs.rs/anchor-lang/
- **Solana Docs:** https://docs.solana.com/
- **Test Files:** `anchor/tests/`
- **Project Docs:** `MASTER_DOCUMENTATION.md`

## Summary

To run the GridTokenX Anchor tests:

1. Build programs: `pnpm run anchor-build`
2. Start validator: `solana-test-validator` (Terminal 1)
3. Run tests: `cd anchor && anchor test --skip-local-validator` (Terminal 2)

Expected result: **83 tests pass in ~65 seconds** ✅