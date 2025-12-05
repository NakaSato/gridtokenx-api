#!/bin/bash
echo "Stopping simulator..."
lsof -ti:8000 | xargs kill -9 || echo "No process on port 8000"
sleep 2

echo "Starting simulator..."
cd /Users/chanthawat/Developments/gridtokenx-platform/gridtokenx-smartmeter-simulator
source .venv/bin/activate || echo "No venv found, trying global python"

# Export API Key explicitly to ensure it overrides defaults
export API_KEY="engineering-department-api-key-2025"

# Start the simulator in the background
nohup python3 -m src.smart_meter_simulator.main > simulator.log 2>&1 &
PID=$!
echo "Simulator started with PID $PID"
sleep 5
echo "Simulator logs:"
tail -n 20 simulator.log
