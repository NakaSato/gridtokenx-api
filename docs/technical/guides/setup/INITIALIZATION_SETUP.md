# Database Initialization Setup Guide

This document describes the initialization scripts for PostgreSQL, Kafka, and InfluxDB in the GridTokenX platform.

## Overview

The platform uses initialization scripts to automatically set up databases, message queues, and time-series storage when containers are first started. This ensures a consistent and reproducible environment.

## Components

### 1. PostgreSQL Database (`docker/postgres/init.sql`)

**Purpose:** Initializes the relational database schema for the P2P energy trading platform.

**Features:**
- Automatic schema creation on first container start
- UUID and crypto extensions enabled
- Complete table structure with relationships
- Indexes for query optimization
- Triggers for automatic timestamp updates
- Sample development data

**Tables Created:**
- `users` - User accounts and KYC status
- `smart_meters` - Smart meter registrations
- `energy_transactions` - Blockchain transaction records
- `market_orders` - Trading orders
- `energy_logs` - Energy generation/consumption logs
- `market_analytics` - Market metrics
- `rec_regulators` - REC regulator accounts
- `oracle_requests` - Oracle data requests

**Usage:**
The script runs automatically when the PostgreSQL container starts for the first time. The initialization file is mounted as:
```yaml
volumes:
  - ./docker/postgres/init.sql:/docker-entrypoint-initdb.d/01-init.sql:ro
```

### 2. Kafka Topics (`docker/kafka/kafka-setup.sh`)

**Purpose:** Creates and configures Kafka topics for message streaming.

**Features:**
- Automatic topic creation with retry logic
- Configurable retention and segmentation
- Compression enabled (Snappy)
- Health checks and verification
- Detailed logging and status reporting

**Topics Created:**
- `energy-readings` - Real-time energy meter data (7 days retention)
- `trading-opportunities` - Market trading events (3 days retention)
- `renewable-certificates` - REC issuance events (30 days retention)
- `market-orders` - Order book updates (7 days retention)
- `transaction-events` - Blockchain transaction events (30 days retention)
- `meter-alerts` - Smart meter alerts (7 days retention)
- `system-events` - System-wide events (30 days retention)

**Configuration:**
Each topic is configured with:
- Partition count (2-3 partitions)
- Replication factor (1 for development)
- Retention period (3-30 days)
- Segment duration (12-24 hours)
- Compression type (Snappy)

**Usage:**
The script runs via the `kafka-init` service which waits for Kafka to be healthy:
```yaml
kafka-init:
  image: confluentinc/cp-kafka:7.5.0
  depends_on:
    kafka:
      condition: service_healthy
  command: ["/bin/bash", "/scripts/kafka-setup.sh"]
  restart: "no"
```

### 3. InfluxDB Buckets (`docker/influxdb/init-influxdb.sh`)

**Purpose:** Sets up InfluxDB organization, buckets, and access tokens.

**Features:**
- Organization and user setup
- Multiple buckets with different retention policies
- Read-only and read/write token generation
- Idempotent execution (can be run multiple times)
- Detailed status reporting

**Buckets Created:**
- `energy_data` - Energy measurements (7 days retention)
- `trading_data` - Trading metrics (30 days retention)
- `meter_readings` - Smart meter readings (7 days retention)
- `market_analytics` - Market analysis data (90 days retention)
- `system_metrics` - System performance metrics (7 days retention)

**Access Tokens:**
- Admin token - Full access (from environment variable)
- Read-only token - Monitoring access
- Read/Write token - Service access

**Usage:**
The script runs via the `influxdb-init` service:
```yaml
influxdb-init:
  image: influxdb:2.7
  depends_on:
    influxdb:
      condition: service_healthy
  command: ["/bin/bash", "/scripts/init-influxdb.sh"]
  restart: "no"
```

## Environment Variables

Ensure these variables are set in your `.env` file:

### PostgreSQL
```env
POSTGRES_DB=p2p_energy_trading
POSTGRES_USER=p2p_user
POSTGRES_PASSWORD=your_secure_password
DATABASE_URL=postgresql://p2p_user:your_secure_password@postgres:5432/p2p_energy_trading
```

### InfluxDB
```env
INFLUXDB_ORG=gridtoken
INFLUXDB_BUCKET=energy_readings
INFLUXDB_TOKEN=your-influxdb-token
INFLUXDB_URL=http://localhost:8086
```

**Important Notes:**
- The `INFLUXDB_TOKEN` must be set to a secure value before starting
- The Docker image will automatically create the organization and primary bucket on first start
- Additional buckets are created by the init script
- Make sure the token matches across all services that use InfluxDB

### Kafka
```env
KAFKA_BROKER_ID=1
KAFKA_ZOOKEEPER_CONNECT=zookeeper:2181
KAFKA_ADVERTISED_LISTENERS=PLAINTEXT://kafka:9092
KAFKA_LISTENER_SECURITY_PROTOCOL_MAP=PLAINTEXT:PLAINTEXT
KAFKA_INTER_BROKER_LISTENER_NAME=PLAINTEXT
KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR=1
KAFKA_GROUP_INITIAL_REBALANCE_DELAY_MS=0
KAFKA_BOOTSTRAP_SERVERS=kafka:9092
```

### Zookeeper
```env
ZOOKEEPER_CLIENT_PORT=2181
ZOOKEEPER_TICK_TIME=2000
```

## Running the Setup

### First-time Setup

1. Start all services:
   ```bash
   docker-compose up -d
   ```

2. Monitor initialization:
   ```bash
   # Watch PostgreSQL initialization
   docker logs p2p-postgres
   
   # Watch Kafka topic creation
   docker logs p2p-kafka-init
   
   # Watch InfluxDB setup
   docker logs p2p-influxdb-init
   ```

### Verifying the Setup

**PostgreSQL:**
```bash
# Connect to PostgreSQL
docker exec -it p2p-postgres psql -U p2p_user -d p2p_energy_trading

# List tables
\dt

# Check sample data
SELECT * FROM users;
```

**Kafka:**
```bash
# List topics
docker exec p2p-kafka kafka-topics --list --bootstrap-server localhost:9092

# Describe a topic
docker exec p2p-kafka kafka-topics --describe --bootstrap-server localhost:9092 --topic energy-readings
```

**InfluxDB:**
```bash
# Access InfluxDB UI
open http://localhost:8086

# Or use CLI to list buckets
docker exec p2p-influxdb-init influx bucket list --host http://influxdb:8086 --token YOUR_TOKEN --org gridtokenx
```

## Resetting the Environment

### Complete Reset
```bash
# Stop all services
docker-compose down

# Remove all volumes (WARNING: This deletes all data!)
docker-compose down -v

# Start fresh
docker-compose up -d
```

### Reset Individual Services

**PostgreSQL:**
```bash
docker-compose stop postgres
docker volume rm gridtokenx-platform_postgres_data
docker-compose up -d postgres
```

**Kafka:**
```bash
docker-compose stop kafka kafka-init
docker volume rm gridtokenx-platform_kafka_data
docker-compose up -d zookeeper kafka kafka-init
```

**InfluxDB:**
```bash
docker-compose stop influxdb influxdb-init
docker volume rm gridtokenx-platform_influxdb_data
docker-compose up -d influxdb influxdb-init
```

## Troubleshooting

### PostgreSQL Issues

**Problem:** Tables not created
- **Solution:** Check logs with `docker logs p2p-postgres`
- Ensure init.sql has no syntax errors
- Verify the file is mounted correctly

### Kafka Issues

**Problem:** Topics not created
- **Solution:** Check kafka-init logs: `docker logs p2p-kafka-init`
- Ensure Kafka is healthy before init runs
- Verify Zookeeper is running

**Problem:** Connection refused
- **Solution:** Wait for Kafka to fully start (can take 30-60 seconds)
- Check health status: `docker inspect p2p-kafka`

### InfluxDB Issues

**Problem:** Buckets not created
- **Solution:** Check logs: `docker logs p2p-influxdb-init`
- Verify InfluxDB is healthy
- Check token in environment variables

**Problem:** Token authentication failed
- **Solution:** Ensure INFLUXDB_TOKEN matches in all configurations
- Tokens are stored in `/run/secrets/` inside the container

## Best Practices

1. **Security:**
   - Use strong passwords in production
   - Rotate tokens regularly
   - Use secrets management for sensitive data

2. **Backup:**
   - Regular database backups
   - Export Kafka topics for critical data
   - Backup InfluxDB data periodically

3. **Monitoring:**
   - Watch initialization logs on first start
   - Set up alerts for service health
   - Monitor disk usage for data volumes

4. **Development:**
   - Use different retention policies for dev/prod
   - Keep sample data minimal in production
   - Document schema changes

## Support

For issues or questions:
- Check service logs: `docker logs <container-name>`
- Review environment variables
- Consult service-specific documentation
- Report bugs using `/reportbug` command

## Version History

- **v1.0** (2025-10-29) - Initial setup with PostgreSQL, Kafka, and InfluxDB initialization scripts
