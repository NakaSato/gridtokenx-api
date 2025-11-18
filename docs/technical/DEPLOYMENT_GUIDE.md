# Market Clearing Engine - Deployment Guide

Production deployment guide for the Market Clearing Engine.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Infrastructure Requirements](#infrastructure-requirements)
3. [Deployment Options](#deployment-options)
4. [Production Configuration](#production-configuration)
5. [Security Hardening](#security-hardening)
6. [Monitoring & Observability](#monitoring--observability)
7. [Backup & Disaster Recovery](#backup--disaster-recovery)
8. [Scaling Strategies](#scaling-strategies)
9. [CI/CD Pipeline](#cicd-pipeline)

---

## Architecture Overview

### Production Architecture

```
                                    ┌─────────────────┐
                                    │   CloudFlare    │
                                    │   CDN + WAF     │
                                    └────────┬────────┘
                                             │
                                    ┌────────▼────────┐
                                    │  Load Balancer  │
                                    │   (nginx/ALB)   │
                                    └────────┬────────┘
                                             │
                    ┌────────────────────────┼────────────────────────┐
                    │                        │                        │
           ┌────────▼────────┐     ┌────────▼────────┐     ┌────────▼────────┐
           │   API Gateway   │     │   API Gateway   │     │   API Gateway   │
           │   Instance 1    │     │   Instance 2    │     │   Instance 3    │
           └────────┬────────┘     └────────┬────────┘     └────────┬────────┘
                    │                        │                        │
                    └────────────────────────┼────────────────────────┘
                                             │
                    ┌────────────────────────┼────────────────────────┐
                    │                        │                        │
           ┌────────▼────────┐     ┌────────▼────────┐     ┌────────▼────────┐
           │  Redis Cluster  │     │   PostgreSQL    │     │     Solana      │
           │  (Persistence)  │     │   Primary +     │     │    Mainnet      │
           │                 │     │   Replicas      │     │                 │
           └─────────────────┘     └─────────────────┘     └─────────────────┘
                                             │
                                    ┌────────▼────────┐
                                    │   S3 Backups    │
                                    │   + Archives    │
                                    └─────────────────┘
```

### High Availability Setup

- **API Gateway:** 3+ instances behind load balancer
- **PostgreSQL:** Primary with 2+ read replicas
- **Redis:** Cluster mode with 3+ nodes
- **Load Balancer:** Health checks every 10 seconds
- **Auto-scaling:** Based on CPU/memory/request rate

---

## Infrastructure Requirements

### Compute Resources

#### API Gateway Instance (per instance)

| Environment | CPU | RAM | Disk | Network |
|-------------|-----|-----|------|---------|
| Development | 2 cores | 4 GB | 20 GB SSD | 100 Mbps |
| Staging | 4 cores | 8 GB | 50 GB SSD | 1 Gbps |
| Production | 8 cores | 16 GB | 100 GB SSD | 10 Gbps |

**Recommended Instance Types:**
- AWS: `c6i.2xlarge` (production), `t3.large` (dev)
- GCP: `c2-standard-8` (production), `e2-standard-2` (dev)
- Azure: `F8s_v2` (production), `B2s` (dev)

#### PostgreSQL Instance

| Environment | CPU | RAM | Disk | IOPS |
|-------------|-----|-----|------|------|
| Development | 2 cores | 8 GB | 50 GB SSD | 3,000 |
| Production | 8 cores | 32 GB | 500 GB SSD | 20,000 |

**Recommended:**
- AWS RDS: `db.r6g.2xlarge`
- GCP Cloud SQL: `db-custom-8-32768`
- Azure Database: `GP_Gen5_8`

#### Redis Instance

| Environment | CPU | RAM | Network |
|-------------|-----|-----|---------|
| Development | 2 cores | 4 GB | 100 Mbps |
| Production | 4 cores | 16 GB | 10 Gbps |

**Recommended:**
- AWS ElastiCache: `cache.r6g.xlarge`
- GCP Memorystore: `M5` tier
- Azure Cache: `P3` tier

### Network Requirements

- **Latency:** < 5ms between API and Database
- **Bandwidth:** 1 Gbps minimum for production
- **Firewall:** Allow ports 8080 (API), 5432 (PostgreSQL), 6379 (Redis)

---

## Deployment Options

### Option 1: Docker Compose (Single Server)

**Best for:** Development, small deployments

```bash
# Clone repository
git clone https://github.com/NakaSato/gridtokenx-platform.git
cd gridtokenx-platform

# Configure environment
cp .env.example .env
nano .env

# Deploy
docker-compose -f docker-compose.prod.yml up -d

# Verify
docker-compose ps
curl http://localhost:8080/api/health
```

**Limitations:**
- Single point of failure
- Limited scalability
- Manual scaling required

---

### Option 2: Kubernetes (Recommended for Production)

**Best for:** Production, high availability, auto-scaling

#### Prerequisites

- Kubernetes cluster (1.24+)
- `kubectl` configured
- Helm 3 installed

#### Deploy

```bash
# Add Helm repository
helm repo add gridtokenx https://charts.gridtokenx.com
helm repo update

# Install with custom values
helm install gridtokenx gridtokenx/api-gateway \
  --namespace gridtokenx \
  --create-namespace \
  --values values-production.yaml

# Verify deployment
kubectl get pods -n gridtokenx
kubectl get svc -n gridtokenx
```

#### Kubernetes Manifests

**Deployment (`k8s/deployment.yaml`):**

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-gateway
  namespace: gridtokenx
spec:
  replicas: 3
  selector:
    matchLabels:
      app: api-gateway
  template:
    metadata:
      labels:
        app: api-gateway
    spec:
      containers:
      - name: api-gateway
        image: gridtokenx/api-gateway:v1.0.0
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: redis-credentials
              key: url
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "4Gi"
            cpu: "2000m"
          limits:
            memory: "8Gi"
            cpu: "4000m"
        livenessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

**Service (`k8s/service.yaml`):**

```yaml
apiVersion: v1
kind: Service
metadata:
  name: api-gateway
  namespace: gridtokenx
spec:
  type: LoadBalancer
  selector:
    app: api-gateway
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
```

**HorizontalPodAutoscaler (`k8s/hpa.yaml`):**

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-gateway-hpa
  namespace: gridtokenx
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-gateway
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

---

### Option 3: AWS ECS/Fargate

**Best for:** AWS-native deployments, serverless containers

```bash
# Build and push Docker image
aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin <account>.dkr.ecr.us-east-1.amazonaws.com
docker build -t gridtokenx-api:latest .
docker tag gridtokenx-api:latest <account>.dkr.ecr.us-east-1.amazonaws.com/gridtokenx-api:latest
docker push <account>.dkr.ecr.us-east-1.amazonaws.com/gridtokenx-api:latest

# Deploy with Terraform
cd terraform/aws-ecs
terraform init
terraform plan -var-file=production.tfvars
terraform apply
```

**Task Definition:**

```json
{
  "family": "gridtokenx-api",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "4096",
  "memory": "8192",
  "containerDefinitions": [
    {
      "name": "api-gateway",
      "image": "<account>.dkr.ecr.us-east-1.amazonaws.com/gridtokenx-api:latest",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "secrets": [
        {
          "name": "DATABASE_URL",
          "valueFrom": "arn:aws:secretsmanager:us-east-1:123456789012:secret:db-url"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/gridtokenx-api",
          "awslogs-region": "us-east-1",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

---

## Production Configuration

### Environment Variables

**Production `.env`:**

```bash
# Server
PORT=8080
RUST_LOG=info,api_gateway=debug
WORKERS=8
RUST_BACKTRACE=1

# Database (connection pooling)
DATABASE_URL=postgresql://user:pass@db.example.com:5432/gridtokenx?sslmode=require
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5
DATABASE_ACQUIRE_TIMEOUT=30

# Redis (cluster mode)
REDIS_URL=redis://redis-cluster.example.com:6379
REDIS_PASSWORD=secure-password
REDIS_POOL_SIZE=20
REDIS_TIMEOUT=5

# Blockchain (mainnet)
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
BLOCKCHAIN_KEYPAIR_PATH=/secure/vault/mainnet-keypair.json
SOLANA_COMMITMENT=finalized
SOLANA_TIMEOUT=30

# Market Clearing
MATCHING_INTERVAL_MS=1000
ORDER_EXPIRATION_HOURS=24
MAX_ORDER_BOOK_SIZE=100000

# Settlement
SETTLEMENT_FEE_RATE=0.001
SETTLEMENT_RETRY_ATTEMPTS=3
SETTLEMENT_RETRY_DELAY_SECS=5
SETTLEMENT_CONFIRMATION_TIMEOUT_SECS=30

# Security
JWT_SECRET=<generate-with: openssl rand -base64 32>
JWT_EXPIRATION_HOURS=24
CORS_ALLOWED_ORIGINS=https://app.gridtokenx.com,https://admin.gridtokenx.com
ALLOWED_HOSTS=api.gridtokenx.com

# Rate Limiting
RATE_LIMIT_PER_MINUTE=100
RATE_LIMIT_BURST=20
ADMIN_RATE_LIMIT_PER_MINUTE=200

# Monitoring
METRICS_ENABLED=true
METRICS_PORT=9090
TRACING_ENABLED=true
SENTRY_DSN=https://...@sentry.io/...
```

### Systemd Service (Linux)

**/etc/systemd/system/gridtokenx-api.service:**

```ini
[Unit]
Description=GridTokenX API Gateway
After=network.target postgresql.service redis.service

[Service]
Type=simple
User=gridtokenx
Group=gridtokenx
WorkingDirectory=/opt/gridtokenx
EnvironmentFile=/opt/gridtokenx/.env
ExecStart=/opt/gridtokenx/api-gateway
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal
SyslogIdentifier=gridtokenx-api

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/gridtokenx/logs

# Resource limits
LimitNOFILE=65536
LimitNPROC=4096

[Install]
WantedBy=multi-user.target
```

**Enable and start:**

```bash
sudo systemctl daemon-reload
sudo systemctl enable gridtokenx-api
sudo systemctl start gridtokenx-api
sudo systemctl status gridtokenx-api
```

---

## Security Hardening

### 1. Network Security

```bash
# Firewall rules (UFW example)
sudo ufw allow 80/tcp    # HTTP (redirect to HTTPS)
sudo ufw allow 443/tcp   # HTTPS
sudo ufw allow 22/tcp    # SSH (restrict by IP)
sudo ufw deny 8080/tcp   # Block direct API access
sudo ufw enable

# Security groups (AWS example)
# Inbound:
# - Port 443 from 0.0.0.0/0 (HTTPS)
# - Port 22 from <admin-ip>/32 (SSH)
# - Port 5432 from <api-security-group> (PostgreSQL)
# - Port 6379 from <api-security-group> (Redis)
```

### 2. TLS/SSL Configuration

**nginx configuration (`/etc/nginx/sites-available/gridtokenx`):**

```nginx
server {
    listen 80;
    server_name api.gridtokenx.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name api.gridtokenx.com;

    # TLS configuration
    ssl_certificate /etc/letsencrypt/live/api.gridtokenx.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/api.gridtokenx.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "DENY" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api_limit:10m rate=100r/m;
    limit_req zone=api_limit burst=20 nodelay;

    # Proxy to API
    location /api/ {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # WebSocket support
    location /ws {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 86400;
    }
}
```

### 3. Secrets Management

**Using AWS Secrets Manager:**

```bash
# Store database credentials
aws secretsmanager create-secret \
    --name gridtokenx/database-url \
    --secret-string "postgresql://user:pass@db.example.com:5432/gridtokenx"

# Store blockchain keypair
aws secretsmanager create-secret \
    --name gridtokenx/blockchain-keypair \
    --secret-binary fileb://keypair.json
```

**Retrieve in application:**

```rust
use aws_sdk_secretsmanager as secretsmanager;

async fn get_secret(secret_name: &str) -> Result<String> {
    let config = aws_config::load_from_env().await;
    let client = secretsmanager::Client::new(&config);
    
    let resp = client
        .get_secret_value()
        .secret_id(secret_name)
        .send()
        .await?;
    
    Ok(resp.secret_string().unwrap().to_string())
}
```

### 4. Database Security

```sql
-- Create restricted database user
CREATE USER gridtokenx_api WITH PASSWORD 'strong-password';

-- Grant minimal permissions
GRANT CONNECT ON DATABASE gridtokenx TO gridtokenx_api;
GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO gridtokenx_api;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO gridtokenx_api;

-- Enable SSL
ALTER SYSTEM SET ssl = on;
ALTER SYSTEM SET ssl_cert_file = '/path/to/server.crt';
ALTER SYSTEM SET ssl_key_file = '/path/to/server.key';
```

---

## Monitoring & Observability

### Prometheus Metrics

**Metrics to collect:**

```rust
// Example metrics
use prometheus::{IntCounter, Histogram, Gauge};

lazy_static! {
    static ref ORDERS_TOTAL: IntCounter =
        IntCounter::new("market_orders_total", "Total orders created").unwrap();
    
    static ref TRADES_TOTAL: IntCounter =
        IntCounter::new("market_trades_total", "Total trades executed").unwrap();
    
    static ref MATCHING_DURATION: Histogram =
        Histogram::new("market_matching_duration_seconds", "Matching cycle duration").unwrap();
    
    static ref ORDER_BOOK_SIZE: Gauge =
        Gauge::new("market_order_book_size", "Current order book size").unwrap();
}
```

**Prometheus configuration (`prometheus.yml`):**

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'gridtokenx-api'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
```

### Grafana Dashboards

**Key metrics to visualize:**

1. **Orders per minute** (line chart)
2. **Trades per minute** (line chart)
3. **Matching latency** (histogram)
4. **Settlement success rate** (gauge)
5. **Order book size** (gauge)
6. **Spread** (line chart)
7. **WebSocket connections** (gauge)
8. **API response time** (histogram)

### Log Aggregation

**Using Elasticsearch + Kibana:**

```bash
# Install Filebeat
sudo apt-get install filebeat

# Configure Filebeat
sudo nano /etc/filebeat/filebeat.yml
```

```yaml
filebeat.inputs:
  - type: log
    enabled: true
    paths:
      - /var/log/gridtokenx/*.log
    json.keys_under_root: true
    json.add_error_key: true

output.elasticsearch:
  hosts: ["elasticsearch.example.com:9200"]
  username: "filebeat"
  password: "password"
```

---

## Backup & Disaster Recovery

### Database Backups

**Automated PostgreSQL backups:**

```bash
#!/bin/bash
# /opt/gridtokenx/scripts/backup-db.sh

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/postgresql"
DB_NAME="gridtokenx"

# Create backup
pg_dump -h localhost -U gridtokenx -F c -b -v -f \
    "$BACKUP_DIR/gridtokenx_$DATE.backup" $DB_NAME

# Upload to S3
aws s3 cp "$BACKUP_DIR/gridtokenx_$DATE.backup" \
    s3://gridtokenx-backups/postgresql/

# Retention: keep last 30 days
find $BACKUP_DIR -name "*.backup" -mtime +30 -delete
```

**Cron schedule:**

```bash
# Backup every 6 hours
0 */6 * * * /opt/gridtokenx/scripts/backup-db.sh
```

### Redis Persistence

**Enable RDB snapshots (`redis.conf`):**

```
save 900 1      # After 900 sec if at least 1 key changed
save 300 10     # After 300 sec if at least 10 keys changed
save 60 10000   # After 60 sec if at least 10000 keys changed

dir /var/lib/redis
dbfilename dump.rdb
```

### Disaster Recovery Plan

**RTO (Recovery Time Objective):** 15 minutes  
**RPO (Recovery Point Objective):** 1 hour

**Recovery steps:**

1. **Declare incident** - Notify team
2. **Assess damage** - Check what's affected
3. **Restore database** - From latest backup (< 5 min)
4. **Restore Redis** - From RDB snapshot (< 2 min)
5. **Redeploy application** - From Docker image (< 5 min)
6. **Verify health** - Run health checks (< 2 min)
7. **Resume traffic** - Update load balancer (< 1 min)

---

## Scaling Strategies

### Horizontal Scaling

**Auto-scaling based on metrics:**

```yaml
# Kubernetes HPA
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-gateway-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-gateway
  minReplicas: 3
  maxReplicas: 20
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "1000"
```

### Database Scaling

**Read replicas for read-heavy workloads:**

```rust
// Connection pool with read replicas
let primary_pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(&env::var("DATABASE_PRIMARY_URL")?).await?;

let replica_pool = PgPoolOptions::new()
    .max_connections(40)
    .connect(&env::var("DATABASE_REPLICA_URL")?).await?;

// Use replica for reads
let stats = sqlx::query_as!(MarketStats,
    "SELECT * FROM market_stats WHERE date = $1",
    today
)
.fetch_one(&replica_pool)
.await?;

// Use primary for writes
sqlx::query!("INSERT INTO orders ...")
    .execute(&primary_pool)
    .await?;
```

### Redis Cluster

**Sharding for high throughput:**

```bash
# Create Redis cluster
redis-cli --cluster create \
    192.168.1.1:6379 \
    192.168.1.2:6379 \
    192.168.1.3:6379 \
    --cluster-replicas 1
```

---

## CI/CD Pipeline

### GitHub Actions

**.github/workflows/deploy-production.yml:**

```yaml
name: Deploy to Production

on:
  push:
    branches: [main]
    tags: ['v*']

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run tests
        run: cd api-gateway && cargo test --lib
      
      - name: Run integration tests
        run: cd tests && npm test

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Build Docker image
        run: docker build -t gridtokenx/api-gateway:${{ github.sha }} .
      
      - name: Push to ECR
        run: |
          aws ecr get-login-password | docker login --username AWS --password-stdin ${{ secrets.ECR_REGISTRY }}
          docker tag gridtokenx/api-gateway:${{ github.sha }} ${{ secrets.ECR_REGISTRY }}/gridtokenx-api:latest
          docker push ${{ secrets.ECR_REGISTRY }}/gridtokenx-api:latest

  deploy:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Deploy to Kubernetes
        run: |
          kubectl set image deployment/api-gateway \
            api-gateway=${{ secrets.ECR_REGISTRY }}/gridtokenx-api:latest \
            -n gridtokenx
          
          kubectl rollout status deployment/api-gateway -n gridtokenx
      
      - name: Run smoke tests
        run: |
          curl -f https://api.gridtokenx.com/api/health || exit 1
```

---

## Post-Deployment Checklist

- [ ] Health checks passing
- [ ] Database migrations applied
- [ ] Redis connected and persisting
- [ ] Blockchain service connected
- [ ] Matching engine running
- [ ] WebSocket connections working
- [ ] Monitoring dashboards showing data
- [ ] Logs flowing to aggregation system
- [ ] Backups configured and tested
- [ ] SSL/TLS certificates valid
- [ ] Rate limiting working
- [ ] Load balancer distributing traffic
- [ ] Auto-scaling tested
- [ ] Disaster recovery plan documented

---

**Last Updated:** November 14, 2025  
**Version:** 1.0.0
