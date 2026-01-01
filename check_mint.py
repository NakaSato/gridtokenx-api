import subprocess
import json
from solana.rpc.api import Client
from solders.pubkey import Pubkey

ENERGY_TOKEN_PROGRAM_ID = Pubkey.from_string("HaT3koMseafcCB9aUQUCrSLMDfN1km7Xik9UhZSG9UV6")
SEEDS = [b"mint"]

mint_pda, bump = Pubkey.find_program_address(SEEDS, ENERGY_TOKEN_PROGRAM_ID)
print(f"Mint Address: {mint_pda}")

# Use solana CLI to get account info
try:
    result = subprocess.run(
        ["solana", "account", str(mint_pda), "--output", "json"],
        capture_output=True,
        text=True
    )
    if result.returncode == 0:
        data = json.loads(result.stdout)
        print(f"Owner: {data.get('owner')}")
        print(f"Data: {data}")
    else:
        print(f"Error getting account: {result.stderr}")
except Exception as e:
    print(f"Exception: {e}")
