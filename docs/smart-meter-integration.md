# Smart Meter Integration Guide

## Overview

The GridTokenX platform provides automated token minting based on smart meter readings. This guide explains how to integrate smart meters with the platform and how the automated processing works.

## Architecture

The smart meter integration consists of the following components:

1. **Meter Reading Submission** - API endpoint for submitting meter readings
2. **Meter Polling Service** - Background service that processes unminted readings
3. **Tokenization Engine** - Converts kWh to tokens and mints them on blockchain
4. **WebSocket Events** - Real-time notifications for meter events
5. **Retry Queue** - Handles failed transactions with exponential backoff

## Flow Diagram

```
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐    ┌───────────────┐    ┌─────────────┐
│ Smart Meter │───▶│  API Gateway │───▶│  PostgreSQL DB │───▶│Polling Service│───▶│ Blockchain  │
└─────────────┘    └──────────────┘    └─────────────────┘    └───────────────┘    └─────────────┘
                           │                      │                        │                   │
                           ▼                      ▼                        ▼                   ▼
                   ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐   ┌────────────────┐
                   │   WebSocket │    │   Admin UI   │    │ Retry Queue DB  │   │ User Wallet   │
                   │  Events    │    │  Dashboard   │    │                 │   │   Balance     │
                   └─────────────┘    └──────────────┘    └─────────────────┘   └────────────────┘
```

## API Integration

### Submitting Meter Readings

Use the following endpoint to submit meter readings:

```http
POST /api/meters/submit-reading
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "kwh_amount": "10.5",
  "reading_timestamp": "2023-11-20T12:00:00Z",
  "meter_signature": "optional_signature_data",
  "meter_id": "optional_meter_uuid"
}
```

**Required Fields:**
- `kwh_amount`: Energy reading in kilowatt-hours (up to 100 kWh)
- `reading_timestamp`: ISO 8601 timestamp of when the reading was taken

**Optional Fields:**
- `meter_signature`: Cryptographic signature for verification
- `meter_id`: UUID of a verified meter (replaces legacy workflow)

### Response

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "user_id": "550e8400-e29b-41d4-a716-446655440001",
  "wallet_address": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
  "kwh_amount": "10.5",
  "reading_timestamp": "2023-11-20T12:00:00Z",
  "submitted_at": "2023-11-20T12:05:00Z",
  "minted": false,
  "mint_tx_signature": null
}
```

## Automated Processing

The automated polling service processes unminted readings based on the configuration:

- **Default interval**: Every 60 seconds
- **Batch size**: Up to 50 readings per cycle
- **Retry mechanism**: Exponential backoff up to 3 attempts

### Processing Steps

1. **Validation**: Check reading age, amount limits, and verification status
2. **Token Calculation**: Convert kWh to tokens using configured ratio
3. **Batch Optimization**: Group readings by token account to minimize fees
4. **Blockchain Minting**: Submit transactions to Solana
5. **Database Update**: Mark readings as minted with transaction signatures
6. **WebSocket Broadcast**: Send real-time notifications

## WebSocket Events

The system broadcasts the following WebSocket events for real-time monitoring:

### MeterReadingReceived
```json
{
  "type": "MeterReadingReceived",
  "user_id": "550e8400-e29b-41d4-a716-446655440001",
  "wallet_address": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
  "meter_serial": "METER-001",
  "kwh_amount": 10.5,
  "timestamp": "2023-11-20T12:05:00Z"
}
```

### TokensMinted
```json
{
  "type": "TokensMinted",
  "user_id": "550e8400-e29b-41d4-a716-446655440001",
  "wallet_address": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
  "meter_serial": "METER-001",
  "kwh_amount": 10.5,
  "tokens_minted": 10500000000,
  "transaction_signature": "5j7s8qB8L3Q9U2tR3H8Y9hP6KcE8gZgQeNpAeK1zCrWt2aV4CjNzDk4Xc",
  "timestamp": "2023-11-20T12:07:15Z"
}
```

### MeterReadingValidationFailed
```json
{
  "type": "MeterReadingValidationFailed",
  "user_id": "550e8400-e29b-41d4-a716-446655440001",
  "wallet_address": "9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM",
  "meter_serial": "METER-001",
  "kwh_amount": 150.0,
  "error_reason": "AmountTooHigh",
  "timestamp": "2023-11-20T12:05:30Z"
}
```

### BatchMintingCompleted
```json
{
  "type": "BatchMintingCompleted",
  "batch_id": "550e8400-e29b-41d4-a716-446655440000",
  "total_readings": 25,
  "successful_mints": 23,
  "failed_mints": 2,
  "timestamp": "2023-11-20T12:07:45Z"
}
```

## Configuration

The system can be configured via environment variables:

### Tokenization Configuration
```bash
# Conversion rate from kWh to tokens
TOKENIZATION_KWH_TO_TOKEN_RATIO=1.0

# Token decimal places (for precision)
TOKENIZATION_DECIMALS=9

# Maximum kWh per reading
TOKENIZATION_MAX_READING_KWH=100.0

# Maximum age of readings to process (days)
TOKENIZATION_READING_MAX_AGE_DAYS=7
```

### Polling Configuration
```bash
# Enable/disable automatic minting
TOKENIZATION_AUTO_MINT_ENABLED=true

# Polling interval in seconds
TOKENIZATION_POLLING_INTERVAL_SECS=60

# Number of readings to process per cycle
TOKENIZATION_BATCH_SIZE=50

# Maximum transactions per batch
TOKENIZATION_MAX_TRANSACTIONS_PER_BATCH=20
```

### Retry Configuration
```bash
# Maximum retry attempts
TOKENIZATION_MAX_RETRY_ATTEMPTS=3

# Initial retry delay in seconds
TOKENIZATION_INITIAL_RETRY_DELAY_SECS=300

# Exponential backoff multiplier
TOKENIZATION_RETRY_BACKOFF_MULTIPLIER=2.0

# Maximum retry delay in seconds
TOKENIZATION_MAX_RETRY_DELAY_SECS=3600
```

## Best Practices

### Security
1. **Meter Verification**: Always use verified meters when possible
2. **Signature Validation**: Include cryptographic signatures with readings
3. **Secure Transmission**: Use HTTPS for all API communications
4. **Rate Limiting**: Implement client-side rate limiting for submissions

### Performance
1. **Batch Submissions**: Submit multiple readings in a single request when possible
2. **Off-Peak Processing**: Configure higher polling intervals during peak market hours
3. **Error Handling**: Implement exponential backoff for API retries
4. **Event Monitoring**: Subscribe to WebSocket events for real-time status

### Integration Examples

### Python Example
```python
import requests
import json
import time
from datetime import datetime

# Configuration
API_URL = "https://api.gridtokenx.com"
JWT_TOKEN = "your_jwt_token"

def submit_reading(kwh_amount):
    """Submit a meter reading to the GridTokenX API"""
    
    headers = {
        "Authorization": f"Bearer {JWT_TOKEN}",
        "Content-Type": "application/json"
    }
    
    payload = {
        "kwh_amount": str(kwh_amount),
        "reading_timestamp": datetime.utcnow().isoformat() + "Z",
        "meter_signature": "your_meter_signature"
    }
    
    response = requests.post(
        f"{API_URL}/api/meters/submit-reading",
        headers=headers,
        data=json.dumps(payload)
    )
    
    if response.status_code == 200:
        return response.json()
    else:
        raise Exception(f"Failed to submit reading: {response.text}")

# Example usage
reading_id = submit_reading(10.5)
print(f"Submitted reading with ID: {reading_id['id']}")
```

### JavaScript Example
```javascript
const API_URL = "https://api.gridtokenx.com";
const JWT_TOKEN = "your_jwt_token";

async function submitReading(kwhAmount) {
  const response = await fetch(`${API_URL}/api/meters/submit-reading`, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${JWT_TOKEN}`,
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      kwh_amount: kwhAmount.toString(),
      reading_timestamp: new Date().toISOString(),
      meter_signature: "your_meter_signature"
    })
  });
  
  if (response.ok) {
    return await response.json();
  } else {
    throw new Error(`Failed to submit reading: ${response.statusText}`);
  }
}

// Example usage
submitReading("10.5")
  .then(reading => console.log(`Submitted reading with ID: ${reading.id}`))
  .catch(error => console.error(error));
```

## Monitoring and Troubleshooting

### Monitoring Metrics

Monitor these metrics for optimal performance:

1. **Processing Latency**: Time from reading submission to token minting
2. **Success Rate**: Percentage of readings successfully minted
3. **Retry Queue Size**: Number of readings awaiting retry
4. **WebSocket Events**: Number of events broadcast
5. **Blockchain Transaction Fees**: Cost per minting operation

### Common Issues

#### Readings Not Processed
- Check that `TOKENIZATION_AUTO_MINT_ENABLED=true`
- Verify `TOKENIZATION_POLLING_INTERVAL_SECS` is reasonable
- Check if readings exceed `TOKENIZATION_READING_MAX_AGE_DAYS`

#### High Failure Rate
- Review `TOKENIZATION_MAX_READING_KWH` limit
- Check blockchain network congestion
- Verify authority wallet has sufficient SOL for fees

#### WebSocket Events Not Received
- Ensure WebSocket connection is authenticated
- Check for network connectivity issues
- Verify event type subscription

## Advanced Topics

### Meter Verification

For production use, meters should be registered and verified:

1. Register meter via admin API
2. Submit meter proof of ownership
3. Include verified `meter_id` in reading submissions
4. Only verified meters will be processed automatically

### Batch Processing Optimization

The system optimizes batch processing by:

1. Grouping readings by token account
2. Optimizing transaction ordering
3. Balancing transaction size and fee cost
4. Parallel processing where possible

### Custom Tokenization Logic

For specialized use cases, the tokenization ratio can be modified:

1. Set `TOKENIZATION_KWH_TO_TOKEN_RATIO` in configuration
2. Adjust `TOKENIZATION_DECIMALS` for precision
3. Consider dynamic rates based on time of day or market conditions

## Support

For integration support:

1. **Documentation**: [docs.gridtokenx.com](https://docs.gridtokenx.com)
2. **API Reference**: [api.gridtokenx.com/docs](https://api.gridtokenx.com/docs)
3. **Community Forum**: [community.gridtokenx.com](https://community.gridtokenx.com)
4. **Support Email**: support@gridtokenx.com

## Changelog

### v1.2.0 (November 2025)
- Added automated polling service
- Implemented retry queue with exponential backoff
- Enhanced WebSocket events for meter data
- Added batch processing optimization
- Improved token conversion with configurable decimals

### v1.1.0 (October 2025)
- Added meter verification support
- Enhanced error handling for blockchain operations
- Improved performance of batch operations
- Added configuration validation

### v1.0.0 (September 2025)
- Initial smart meter integration
- Basic meter reading submission
- Manual token minting via admin endpoint
- WebSocket support for trading events