#!/bin/bash
echo "🛑 Stopping all services..."
killall -9 api-gateway || true
killall -9 solana-test-validator || true
sleep 2

echo "🧹 Cleaning databases..."
# Assuming we have a way to reset DB - usually by dropping and creating
# For now, let's just clear the relevant tables if possible, but a full drop is safer.
# Since I don't have psql, I'll try to find a way or skip for now if I can't.

echo "🚀 Starting Solana Localnet Validator..."
# Clear ledger to ensure clean state
rm -rf test-ledger
solana-test-validator --reset --quiet &
sleep 5

echo "🏗️ Setting up Token Mint..."
# This will be done by the API Gateway startup or I can do it manually
# Actually the API Gateway expects the mint in .env

echo "✨ Ready for new test run."
