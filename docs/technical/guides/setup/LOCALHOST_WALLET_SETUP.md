# Localhost Wallet Setup Guide

This guide explains how to connect your frontend application to a local Solana validator for development and testing.

## Prerequisites

1. **Local Solana Validator**: Make sure you have a local Solana validator running on `http://127.0.0.1:8899`
   ```bash
   # Start a local validator
   solana-test-validator
   ```

2. **Wallet with SOL**: Fund your wallet with test SOL for deployment and transactions
   ```bash
   # Airdrop SOL to your wallet
   solana airdrop 10 -u http://127.0.0.1:8899 --keypair /path/to/your/dev-wallet.json
   ```

## How to Use Localhost Network

### 1. Switch to Localhost Network
- Look for the **Network Selector** in the top navigation bar (right side, near the wallet button)
- Click on it to see available networks:
  - **Mainnet** (green) - Production Solana network
  - **Devnet** (blue) - Solana development network
  - **Localhost** (orange) - Your local validator at `127.0.0.1:8899`

### 2. Connect Your Wallet
- Select "Localhost" from the network selector
- Click "Connect Wallet"
- The wallet modal will show "Network: Localhost (127.0.0.1:8899)" to confirm
- Choose your preferred wallet (Phantom, Solflare, etc.)
- Approve the connection in your wallet extension

### 3. Verify Connection
- Your wallet should now be connected to the local network
- You can check the network in your wallet extension
- All transactions will now go to your local validator

## Network Persistence

- Your network selection is automatically saved in browser localStorage
- The app will remember your choice between sessions
- Switching networks will reload the page to ensure a clean state

## Troubleshooting

### Wallet Not Connecting
- Ensure your local validator is running: `solana-test-validator`
- Check that your wallet extension supports custom RPC endpoints
- Try refreshing the page after switching networks

### Transactions Failing
- Make sure your wallet has sufficient SOL on localhost
- Verify your local validator is healthy: `solana cluster-version -u localhost`

### Network Not Showing
- Clear browser localStorage if the network selector isn't working
- Check browser console for any JavaScript errors

## Development Workflow

1. Start your local validator
2. Fund your wallet with airdrop
3. Switch to localhost network in the app
4. Connect your wallet
5. Deploy/test your programs
6. Use the app with local data

This setup allows you to develop and test your Solana applications locally before deploying to devnet or mainnet.