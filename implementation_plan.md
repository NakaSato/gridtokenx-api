# Blockchain Integration Testing Plan

## Goal
Validate the fully implemented Blockchain Core by executing real on-chain transactions in a localnet environment. This phase ensures that the Key Management, ERC Service, and Settlement logic work correctly when interacting with deployed Solana programs.

## User Review Required
> [!IMPORTANT]
> **Program Deployment**: This plan assumes the Anchor programs (Registry, Oracle, Governance, Energy Token, Trading) are located in a sibling directory (`../gridtokenx-anchor`) or are already deployed to the localnet.
> 
> **Action Required**: Please ensure `solana-test-validator` is running and programs are deployed. If they are not deployed, please deploy them using `anchor deploy` in the anchor project.

## Proposed Changes

### 1. Environment Setup Scripts
Create scripts to automate the preparation of the blockchain environment.

#### [NEW] `examples/setup_blockchain_env.rs`
- **Fund Authority Wallet**: Checks balance and requests airdrop if low (< 5 SOL).
- **Verify Program Deployment**: Checks if program accounts exist on-chain.
- **Initialize Token Mint**: If the Energy Token program is deployed but the mint doesn't exist, initializes the mint (requires calling the program's initialize instruction).

### 2. Integration Tests
Create comprehensive integration tests in `tests/integration/` to verify core workflows.

#### [NEW] `tests/integration/erc_lifecycle_test.rs`
- **Issue Certificate**: Issue a new ERC certificate on-chain.
- **Verify State**: Fetch account info to verify on-chain data matches.
- **Transfer Certificate**: Transfer ownership to another wallet.
- **Retire Certificate**: Retire the certificate and verify status.

#### [NEW] `tests/integration/token_minting_test.rs`
- **Mint Tokens**: Use `BlockchainService` to mint Energy Tokens to a user wallet.
- **Verify Balance**: Check SPL token balance of the recipient.

#### [NEW] `tests/integration/settlement_test.rs`
- **Setup**: Create buyer and seller token accounts.
- **Execute Transfer**: Run `SettlementService::execute_blockchain_transfer`.
- **Verify Balances**: Ensure tokens moved correctly.

#### [NEW] `tests/integration/full_trading_cycle_test.rs`
- **End-to-End Flow**: Test the complete path from Order -> Match -> Settlement -> Blockchain.
- **Simulate Market**: Create conflicting buy/sell orders in the database.
- **Trigger Matching**: Call `MarketClearingEngine::execute_matching_cycle`.
- **Verify Settlement**: Ensure `Settlement` record is created and `SettlementService` picks it up.
- **Check Blockchain**: Verify the final transaction signature in the database.

## Verification Plan

### Automated Verification
Run the setup script followed by the integration tests:

```bash
# 1. Setup Environment (Fund wallet, check programs)
cargo run --example setup_blockchain_env

# 2. Run Integration Tests
cargo test --test erc_lifecycle_test
cargo test --test token_minting_test
cargo test --test settlement_test
cargo test --test full_trading_cycle_test
```

### Manual Verification
- Use `solana explorer` (custom RPC to localhost) to visually inspect transactions.
- Check logs for transaction signatures and confirm they exist on-chain.
