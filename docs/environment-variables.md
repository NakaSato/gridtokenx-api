# GridTokenX API Gateway Environment Variables

This document describes all the environment variables that can be configured for the GridTokenX API Gateway.

## Core Configuration

### `ENVIRONMENT`
- **Description**: Deployment environment (development, staging, production)
- **Required**: Yes
- **Default**: None
- **Example**: `ENVIRONMENT=development`

### `PORT`
- **Description**: Port for the API Gateway to listen on
- **Required**: Yes
- **Default**: None
- **Example**: `PORT=8080`

### `LOG_LEVEL`
- **Description**: Logging level for the application
- **Required**: Yes
- **Default**: None
- **Example**: `LOG_LEVEL=info`

## Database Configuration

### `DATABASE_URL`
- **Description**: PostgreSQL connection URL
- **Required**: Yes
- **Default**: None
- **Example**: `DATABASE_URL=postgresql://user:password@localhost:5432/gridtokenx`

### `INFLUXDB_URL`
- **Description**: TimescaleDB connection URL
- **Required**: No
- **Default**: `http://localhost:8086`
- **Example**: `INFLUXDB_URL=http://localhost:8086`

### `REDIS_URL`
- **Description**: Redis connection URL
- **Required**: Yes
- **Default**: None
- **Example**: `REDIS_URL=redis://password@localhost:6379`

## Authentication Configuration

### `JWT_SECRET`
- **Description**: Secret key for JWT token signing
- **Required**: Yes
- **Default**: None
- **Example**: `JWT_SECRET=your-super-secret-jwt-key`

### `JWT_EXPIRATION`
- **Description**: JWT token expiration time in seconds
- **Required**: No
- **Default**: `86400` (24 hours)
- **Example**: `JWT_EXPIRATION=86400`

## Blockchain Configuration

### `SOLANA_RPC_URL`
- **Description**: Solana RPC endpoint URL
- **Required**: Yes
- **Default**: None
- **Example**: `SOLANA_RPC_URL=https://api.devnet.solana.com`

### `SOLANA_WS_URL`
- **Description**: Solana WebSocket endpoint URL
- **Required**: Yes
- **Default**: None
- **Example**: `SOLANA_WS_URL=wss://api.devnet.solana.com`

### `ENERGY_TOKEN_MINT`
- **Description**: Energy token mint public key
- **Required**: Yes
- **Default**: None
- **Example**: `ENERGY_TOKEN_MINT=9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM`

### `AUTHORITY_WALLET_PATH`
- **Description**: Path to the authority wallet JSON file
- **Required**: No
- **Default**: `dev-wallet.json`
- **Example**: `AUTHORITY_WALLET_PATH=/path/to/authority-wallet.json`

## Application Configuration

### `ENGINEERING_API_KEY`
- **Description**: API key for engineering endpoints
- **Required**: Yes
- **Default**: None
- **Example**: `ENGINEERING_API_KEY=your-engineering-api-key`

### `MAX_CONNECTIONS`
- **Description**: Maximum number of database connections
- **Required**: Yes
- **Default**: None
- **Example**: `MAX_CONNECTIONS=20`

### `REDIS_POOL_SIZE`
- **Description**: Redis connection pool size
- **Required**: Yes
- **Default**: None
- **Example**: `REDIS_POOL_SIZE=10`

### `REQUEST_TIMEOUT`
- **Description**: Request timeout in seconds
- **Required**: Yes
- **Default**: None
- **Example**: `REQUEST_TIMEOUT=30`

### `RATE_LIMIT_WINDOW`
- **Description**: Rate limiting window in seconds
- **Required**: Yes
- **Default**: None
- **Example**: `RATE_LIMIT_WINDOW=60`

### `AUDIT_LOG_ENABLED`
- **Description**: Whether audit logging is enabled
- **Required**: Yes
- **Default**: None
- **Example**: `AUDIT_LOG_ENABLED=true`

### `TEST_MODE`
- **Description**: Whether the application is running in test mode
- **Required**: No
- **Default**: `false`
- **Example**: `TEST_MODE=true`

## Email Configuration

### `SMTP_HOST`
- **Description**: SMTP server hostname
- **Required**: No
- **Default**: `smtp.gmail.com`
- **Example**: `SMTP_HOST=smtp.gmail.com`

### `SMTP_PORT`
- **Description**: SMTP server port
- **Required**: No
- **Default**: `587`
- **Example**: `SMTP_PORT=587`

### `SMTP_USERNAME`
- **Description**: SMTP username
- **Required**: No
- **Default**: `noreply@gridtokenx.com`
- **Example**: `SMTP_USERNAME=your-smtp-username`

### `SMTP_PASSWORD`
- **Description**: SMTP password
- **Required**: No
- **Default**: Empty string
- **Example**: `SMTP_PASSWORD=your-smtp-password`

### `EMAIL_FROM_NAME`
- **Description**: Name to use in from address
- **Required**: No
- **Default**: `GridTokenX Platform`
- **Example**: `EMAIL_FROM_NAME=GridTokenX Platform`

### `EMAIL_FROM_ADDRESS`
- **Description**: Email address to use as sender
- **Required**: No
- **Default**: `noreply@gridtokenx.com`
- **Example**: `EMAIL_FROM_ADDRESS=noreply@gridtokenx.com`

### `EMAIL_VERIFICATION_EXPIRY_HOURS`
- **Description**: Email verification link expiry time in hours
- **Required**: No
- **Default**: `24`
- **Example**: `EMAIL_VERIFICATION_EXPIRY_HOURS=24`

### `EMAIL_VERIFICATION_BASE_URL`
- **Description**: Base URL for email verification links
- **Required**: No
- **Default**: `http://localhost:3000`
- **Example**: `EMAIL_VERIFICATION_BASE_URL=https://app.gridtokenx.com`

### `EMAIL_VERIFICATION_REQUIRED`
- **Description**: Whether email verification is required
- **Required**: No
- **Default**: `true`
- **Example**: `EMAIL_VERIFICATION_REQUIRED=true`

### `EMAIL_VERIFICATION_ENABLED`
- **Description**: Whether email verification is enabled
- **Required**: No
- **Default**: `true`
- **Example**: `EMAIL_VERIFICATION_ENABLED=true`

### `EMAIL_AUTO_LOGIN_AFTER_VERIFICATION`
- **Description**: Whether to automatically log in after email verification
- **Required**: No
- **Default**: `true`
- **Example**: `EMAIL_AUTO_LOGIN_AFTER_VERIFICATION=true`

## Smart Meter Tokenization Configuration

### `TOKENIZATION_KWH_TO_TOKEN_RATIO`
- **Description**: Conversion ratio from kWh to tokens
- **Required**: No
- **Default**: `1.0`
- **Example**: `TOKENIZATION_KWH_TO_TOKEN_RATIO=1.0`

### `TOKENIZATION_DECIMALS`
- **Description**: Number of decimal places for token representation
- **Required**: No
- **Default**: `9`
- **Example**: `TOKENIZATION_DECIMALS=9`

### `TOKENIZATION_MAX_READING_KWH`
- **Description**: Maximum kWh allowed per meter reading
- **Required**: No
- **Default**: `100.0`
- **Example**: `TOKENIZATION_MAX_READING_KWH=100.0`

### `TOKENIZATION_READING_MAX_AGE_DAYS`
- **Description**: Maximum age of a reading in days before it's too old to process
- **Required**: No
- **Default**: `7`
- **Example**: `TOKENIZATION_READING_MAX_AGE_DAYS=7`

### `TOKENIZATION_AUTO_MINT_ENABLED`
- **Description**: Whether automatic minting is enabled
- **Required**: No
- **Default**: `true`
- **Example**: `TOKENIZATION_AUTO_MINT_ENABLED=true`

### `TOKENIZATION_POLLING_INTERVAL_SECS`
- **Description**: Interval in seconds for polling unminted readings
- **Required**: No
- **Default**: `60`
- **Example**: `TOKENIZATION_POLLING_INTERVAL_SECS=60`

### `TOKENIZATION_BATCH_SIZE`
- **Description**: Number of readings to process in one batch
- **Required**: No
- **Default**: `50`
- **Example**: `TOKENIZATION_BATCH_SIZE=50`

### `TOKENIZATION_MAX_RETRY_ATTEMPTS`
- **Description**: Maximum number of retry attempts for failed minting
- **Required**: No
- **Default**: `3`
- **Example**: `TOKENIZATION_MAX_RETRY_ATTEMPTS=3`

### `TOKENIZATION_INITIAL_RETRY_DELAY_SECS`
- **Description**: Initial delay in seconds for retry logic
- **Required**: No
- **Default**: `300` (5 minutes)
- **Example**: `TOKENIZATION_INITIAL_RETRY_DELAY_SECS=300`

### `TOKENIZATION_RETRY_BACKOFF_MULTIPLIER`
- **Description**: Exponential backoff multiplier for retries
- **Required**: No
- **Default**: `2.0`
- **Example**: `TOKENIZATION_RETRY_BACKOFF_MULTIPLIER=2.0`

### `TOKENIZATION_MAX_RETRY_DELAY_SECS`
- **Description**: Maximum delay in seconds between retries
- **Required**: No
- **Default**: `3600` (1 hour)
- **Example**: `TOKENIZATION_MAX_RETRY_DELAY_SECS=3600`

### `TOKENIZATION_TRANSACTION_TIMEOUT_SECS`
- **Description**: Timeout in seconds for blockchain transaction confirmation
- **Required**: No
- **Default**: `60`
- **Example**: `TOKENIZATION_TRANSACTION_TIMEOUT_SECS=60`

### `TOKENIZATION_MAX_TRANSACTIONS_PER_BATCH`
- **Description**: Maximum number of transactions per batch
- **Required**: No
- **Default**: `20`
- **Example**: `TOKENIZATION_MAX_TRANSACTIONS_PER_BATCH=20`

## Security Considerations

1. **Secret Management**: Never commit secrets like `JWT_SECRET` or `SMTP_PASSWORD` to version control. Use secure secret management in production.

2. **Network Security**: Use TLS/SSL for all external service connections:
   - PostgreSQL (DATABASE_URL with sslmode=require)
   - Redis (rediss:// instead of redis://)
   - Solana (https:// instead of http://)

3. **Environment Isolation**: Use different configurations for development, staging, and production environments.

4. **Rate Limiting**: Configure appropriate rate limiting values based on your infrastructure capacity.

5. **Email Security**: Use app-specific passwords for SMTP authentication rather than primary account passwords.

6. **Blockchain Security**: Secure the authority wallet keypair in production using a secure key management system.

## Monitoring and Observability

1. **Logging**: Set appropriate LOG_LEVEL values for different environments:
   - Development: `debug` or `trace`
   - Staging: `info`
   - Production: `warn` or `error`

2. **Metrics**: Prometheus metrics are exposed at `/metrics` endpoint when configured.

3. **Health Checks**: Use the `/health` endpoint to monitor application status.

4. **Audit Logging**: Enable AUDIT_LOG_ENABLED in production for security compliance.

## Performance Tuning

1. **Database**: Adjust MAX_CONNECTIONS based on your database capacity.

2. **Caching**: Configure REDIS_POOL_SIZE based on expected concurrent users.

3. **Batch Processing**: Adjust TOKENIZATION_BATCH_SIZE based on blockchain network congestion and transaction costs.

4. **Rate Limiting**: Fine-tune RATE_LIMIT_WINDOW to balance security and user experience.

5. **Timeouts**: Adjust REQUEST_TIMEOUT and TOKENIZATION_TRANSACTION_TIMEOUT_SECS based on network latency.