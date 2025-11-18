# Anchor Tests Documentation Index

Welcome to the GridTokenX Anchor Tests documentation. This folder contains comprehensive guides for running, understanding, and extending the test suite.

## Quick Navigation

### 📖 Start Here
- **[TESTS_CREATED.md](TESTS_CREATED.md)** - Overview of what was created (start with this!)
- **[QUICK_TEST_GUIDE.md](QUICK_TEST_GUIDE.md)** - Quick reference commands and cheat sheet

### 🚀 Running Tests
- **[RUN_ANCHOR_TESTS.md](RUN_ANCHOR_TESTS.md)** - Complete setup and running instructions
  - Prerequisites and installation
  - Step-by-step setup guide
  - Multiple running options
  - Troubleshooting and debugging
  - CI/CD integration examples

### 📊 Test Details
- **[ANCHOR_TESTS_SUMMARY.md](ANCHOR_TESTS_SUMMARY.md)** - Detailed test statistics and breakdown
  - Test files and statistics
  - Coverage by program
  - Test execution flow
  - Quality metrics
  - Performance notes

### 📝 Test Files Documentation
- **[../anchor/tests/README.md](../anchor/tests/README.md)** - Comprehensive test file documentation
  - Detailed description of each test suite
  - Test structure and patterns
  - Best practices
  - Adding new tests

## Test Suite Overview

### 5 Anchor Programs with 83 Total Tests

| Program | Tests | Location |
|---------|-------|----------|
| Oracle | 10 | `anchor/tests/oracle.test.ts` |
| Registry | 15 | `anchor/tests/registry.test.ts` |
| Trading | 19 | `anchor/tests/trading.test.ts` |
| Energy Token | 18 | `anchor/tests/energy-token.test.ts` |
| Governance | 21 | `anchor/tests/governance.test.ts` |

## Quick Start

### Using the Test Runner Script
```bash
# Run all 83 tests
./run-tests.sh all

# Run specific program
./run-tests.sh oracle          # 10 tests
./run-tests.sh registry        # 15 tests
./run-tests.sh trading         # 19 tests
./run-tests.sh energy-token    # 18 tests
./run-tests.sh governance      # 21 tests

# Other commands
./run-tests.sh setup           # Build only
./run-tests.sh validator       # Start validator
./run-tests.sh help            # Show options
```

### Manual Setup
```bash
# Terminal 1: Start validator
solana-test-validator

# Terminal 2: Run tests
cd anchor
anchor test --skip-local-validator
```

## Documentation Organization

```
docs/
├── ANCHOR_TESTS_INDEX.md           ← You are here
├── TESTS_CREATED.md                (What was created)
├── QUICK_TEST_GUIDE.md             (Quick reference)
├── RUN_ANCHOR_TESTS.md             (Complete guide)
├── ANCHOR_TESTS_SUMMARY.md         (Test statistics)
└── ../anchor/tests/README.md       (Test details)
```

## Key Features

✅ **Comprehensive**
- 83 tests covering all major functionality
- Authorization, error handling, edge cases
- ~2,312 lines of test code

✅ **Well Documented**
- Multiple guides for different use cases
- Step-by-step instructions
- Troubleshooting section
- CI/CD examples

✅ **Easy to Use**
- Automated test runner script
- Quick start commands
- Clear error messages

✅ **Production Ready**
- Can be integrated into CI/CD
- Follows Anchor best practices
- Performance optimized

## Expected Performance

| Program | Duration |
|---------|----------|
| Oracle | ~8s |
| Registry | ~12s |
| Trading | ~15s |
| Energy Token | ~12s |
| Governance | ~18s |
| **TOTAL** | **~65s** |

## Choosing Your Documentation

### I want to...

**...quickly run tests**
→ Read [QUICK_TEST_GUIDE.md](QUICK_TEST_GUIDE.md)

**...understand what was created**
→ Read [TESTS_CREATED.md](TESTS_CREATED.md)

**...set up tests from scratch**
→ Read [RUN_ANCHOR_TESTS.md](RUN_ANCHOR_TESTS.md)

**...see detailed test coverage**
→ Read [ANCHOR_TESTS_SUMMARY.md](ANCHOR_TESTS_SUMMARY.md)

**...understand test structure and add new tests**
→ Read [../anchor/tests/README.md](../anchor/tests/README.md)

**...integrate tests into CI/CD**
→ See [RUN_ANCHOR_TESTS.md](RUN_ANCHOR_TESTS.md) CI/CD Integration section

**...debug failing tests**
→ See [RUN_ANCHOR_TESTS.md](RUN_ANCHOR_TESTS.md) Troubleshooting section

## Troubleshooting Quick Links

| Issue | Solution |
|-------|----------|
| vitest command not found | Removed from Anchor.toml (already fixed!) |
| Connection refused | Start validator: `solana-test-validator` |
| Keypair not found | Create wallet: `solana-keygen new` |
| Program not deployed | Run: `anchor build && anchor deploy` |
| Test timeout | Use: `--timeout 120` flag |

See [RUN_ANCHOR_TESTS.md](RUN_ANCHOR_TESTS.md) for more troubleshooting.

## File Sizes

```
TESTS_CREATED.md          6.0 KB  (Quick overview)
QUICK_TEST_GUIDE.md       6.5 KB  (Quick reference)
RUN_ANCHOR_TESTS.md       10 KB   (Complete guide)
ANCHOR_TESTS_SUMMARY.md   13 KB   (Test statistics)
../anchor/tests/README.md 378 lines (Test documentation)
```

## Test File Locations

```
gridtokenx-app/
├── docs/
│   ├── ANCHOR_TESTS_INDEX.md       ← Start here!
│   ├── TESTS_CREATED.md
│   ├── QUICK_TEST_GUIDE.md
│   ├── RUN_ANCHOR_TESTS.md
│   └── ANCHOR_TESTS_SUMMARY.md
├── anchor/
│   ├── tests/
│   │   ├── oracle.test.ts          (10 tests)
│   │   ├── registry.test.ts        (15 tests)
│   │   ├── trading.test.ts         (19 tests)
│   │   ├── energy-token.test.ts    (18 tests)
│   │   ├── governance.test.ts      (21 tests)
│   │   └── README.md
│   ├── programs/                   (Program source code)
│   └── Anchor.toml                 (Fixed - vitest removed)
├── run-tests.sh                    (Test runner script)
└── MASTER_DOCUMENTATION.md         (Main project docs)
```

## Getting Help

1. **Check the troubleshooting section** in [RUN_ANCHOR_TESTS.md](RUN_ANCHOR_TESTS.md)
2. **Review test examples** in [../anchor/tests/README.md](../anchor/tests/README.md)
3. **See test patterns** in the individual test files
4. **Check Anchor documentation** at https://docs.rs/anchor-lang/

## Summary

This folder contains everything you need to run and understand the GridTokenX Anchor test suite:

- **4 documentation files** (~35 KB total)
- **83 comprehensive tests** across 5 programs
- **Easy-to-use test runner** script
- **Multiple setup options**
- **Complete troubleshooting guide**

**Start with:** [TESTS_CREATED.md](TESTS_CREATED.md) or [QUICK_TEST_GUIDE.md](QUICK_TEST_GUIDE.md)

**Run tests with:** `./run-tests.sh all`

Happy Testing! 🧪