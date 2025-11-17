# Week 3: Priority Fee System - Design Document

## Executive Summary

**Week 3 Objective**: Implement dynamic priority fee estimation and optimal transaction timing to minimize Solana network costs while ensuring reliable settlement execution.

**Status**: Design Phase - Ready for Implementation  
**Estimated Effort**: 5-7 days  
**Dependencies**: Week 2 (Batch Transaction System) - ✅ Complete

### Key Features

1. **Dynamic Fee Estimation**: Real-time calculation of optimal priority fees based on network conditions
2. **Congestion Monitoring**: Track Solana network metrics (TPS, slot time, fee percentiles)
3. **Intelligent Timing**: Predict optimal submission windows to minimize fees
4. **Historical Analytics**: Learn from past transactions to improve fee predictions
5. **Cost Optimization**: Balance between execution speed and fee minimization

## 1. System Architecture

### 1.1 Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                   Priority Fee System                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │         FeeEstimationService                       │    │
│  │  • Dynamic fee calculation                         │    │
│  │  • Network congestion analysis                     │    │
│  │  • Optimal timing prediction                       │    │
│  └───────────────┬────────────────────────────────────┘    │
│                  │                                          │
│  ┌───────────────▼───────────────┐  ┌─────────────────┐   │
│  │  NetworkMetricsCollector      │  │  FeeOracle      │   │
│  │  • TPS monitoring              │  │  • Recent fees  │   │
│  │  • Slot time tracking          │  │  • Percentiles  │   │
│  │  • Fee sampling                │  │  • Predictions  │   │
│  └───────────────┬───────────────┘  └────────┬────────┘   │
│                  │                            │             │
│  ┌───────────────▼───────────────┬────────────▼────────┐   │
│  │  HistoricalFeeAnalyzer        │  TimingOptimizer    │   │
│  │  • Fee trends                  │  • Submission       │   │
│  │  • Success rates               │  │  windows        │   │
│  │  • Cost analysis               │  • Queue backlog    │   │
│  └────────────────────────────────┴─────────────────────┘   │
│                                                              │
└──────────────────────────────────────────────────────────────┘
                          │
                          ▼
            ┌─────────────────────────┐
            │ BatchTransactionService │
            │  (Week 2 Integration)   │
            └─────────────────────────┘
```

### 1.2 Service Interactions

```
┌─────────────┐     estimate_priority_fee()     ┌──────────────────────┐
│  Batch      ├────────────────────────────────►│ FeeEstimation        │
│  Service    │◄────────────────────────────────┤ Service              │
│  (Week 2)   │     FeeEstimate                 └──────────┬───────────┘
└─────────────┘                                            │
                                                           │ collect_metrics()
                                                           ▼
                                          ┌────────────────────────────┐
                                          │ NetworkMetricsCollector     │
                                          │  • RPC sampling             │
                                          │  • Metric aggregation       │
                                          └────────────────────────────┘
                                                           │
                                                           │ store_metrics()
                                                           ▼
                                          ┌────────────────────────────┐
                                          │ PostgreSQL Database         │
                                          │  • network_metrics          │
                                          │  • fee_history             │
                                          │  • timing_windows          │
                                          └────────────────────────────┘
```

## 2. Data Model

### 2.1 Database Schema

```sql
-- Network metrics collected from Solana RPC
CREATE TABLE network_metrics (
    id BIGSERIAL PRIMARY KEY,
    collected_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Network performance
    current_tps DECIMAL(10, 2) NOT NULL,
    slot_time_ms INTEGER NOT NULL,
    recent_blockhash VARCHAR(88) NOT NULL,
    
    -- Fee metrics
    min_priority_fee BIGINT NOT NULL,
    median_priority_fee BIGINT NOT NULL,
    p75_priority_fee BIGINT NOT NULL,
    p90_priority_fee BIGINT NOT NULL,
    max_priority_fee BIGINT NOT NULL,
    
    -- Congestion indicators
    congestion_level VARCHAR(20) NOT NULL, -- 'low', 'medium', 'high', 'critical'
    failed_tx_rate DECIMAL(5, 2),
    
    -- Metadata
    sample_size INTEGER NOT NULL,
    rpc_node VARCHAR(255) NOT NULL,
    
    CONSTRAINT check_congestion_level 
        CHECK (congestion_level IN ('low', 'medium', 'high', 'critical'))
);

CREATE INDEX idx_network_metrics_collected_at ON network_metrics(collected_at DESC);
CREATE INDEX idx_network_metrics_congestion ON network_metrics(congestion_level, collected_at DESC);

-- Historical fee analysis for learning
CREATE TABLE fee_history (
    id BIGSERIAL PRIMARY KEY,
    submitted_at TIMESTAMPTZ NOT NULL,
    
    -- Transaction details
    transaction_signature VARCHAR(88) UNIQUE NOT NULL,
    priority_fee_paid BIGINT NOT NULL,
    compute_units_used INTEGER NOT NULL,
    
    -- Execution details
    slot_submitted BIGINT NOT NULL,
    slot_confirmed BIGINT,
    confirmation_time_ms INTEGER,
    
    -- Network state at submission
    network_tps DECIMAL(10, 2),
    network_congestion VARCHAR(20),
    median_fee_at_submission BIGINT,
    
    -- Outcome
    success BOOLEAN NOT NULL,
    error_message TEXT,
    
    -- Cost analysis
    overpayment_lamports BIGINT, -- How much more than median was paid
    underpayment_risk BOOLEAN DEFAULT FALSE, -- Fee was below recommended
    
    CONSTRAINT check_confirmation_data 
        CHECK ((success = true AND slot_confirmed IS NOT NULL) OR 
               (success = false AND error_message IS NOT NULL))
);

CREATE INDEX idx_fee_history_submitted_at ON fee_history(submitted_at DESC);
CREATE INDEX idx_fee_history_success ON fee_history(success, submitted_at DESC);
CREATE INDEX idx_fee_history_congestion ON fee_history(network_congestion, submitted_at DESC);

-- Optimal timing windows for different priority levels
CREATE TABLE timing_windows (
    id BIGSERIAL PRIMARY KEY,
    analyzed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- Time window
    hour_of_day INTEGER NOT NULL CHECK (hour_of_day >= 0 AND hour_of_day < 24),
    day_of_week INTEGER NOT NULL CHECK (day_of_week >= 0 AND day_of_week < 7),
    
    -- Fee statistics for this window
    avg_median_fee BIGINT NOT NULL,
    avg_p90_fee BIGINT NOT NULL,
    avg_tps DECIMAL(10, 2) NOT NULL,
    
    -- Success metrics
    avg_confirmation_time_ms INTEGER NOT NULL,
    success_rate DECIMAL(5, 2) NOT NULL,
    
    -- Recommendations
    recommended_min_fee BIGINT NOT NULL,
    optimal_window BOOLEAN DEFAULT FALSE,
    
    -- Sample size
    transactions_analyzed INTEGER NOT NULL,
    
    CONSTRAINT unique_time_window UNIQUE (hour_of_day, day_of_week)
);

CREATE INDEX idx_timing_windows_optimal ON timing_windows(optimal_window, hour_of_day);

-- Fee prediction cache (short-lived, for quick lookups)
CREATE TABLE fee_predictions (
    id BIGSERIAL PRIMARY KEY,
    predicted_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    
    -- Prediction parameters
    priority_level VARCHAR(20) NOT NULL,
    target_confirmation_time_ms INTEGER NOT NULL,
    
    -- Predicted fees
    recommended_fee BIGINT NOT NULL,
    min_safe_fee BIGINT NOT NULL,
    max_reasonable_fee BIGINT NOT NULL,
    
    -- Confidence metrics
    confidence_score DECIMAL(3, 2) NOT NULL CHECK (confidence_score >= 0 AND confidence_score <= 1),
    based_on_samples INTEGER NOT NULL,
    
    -- Context
    network_congestion VARCHAR(20) NOT NULL,
    current_tps DECIMAL(10, 2),
    
    CONSTRAINT check_fee_range 
        CHECK (min_safe_fee <= recommended_fee AND recommended_fee <= max_reasonable_fee),
    CONSTRAINT check_priority_level 
        CHECK (priority_level IN ('low', 'medium', 'high', 'critical'))
);

CREATE INDEX idx_fee_predictions_expires_at ON fee_predictions(expires_at);
CREATE INDEX idx_fee_predictions_priority ON fee_predictions(priority_level, predicted_at DESC);

-- Cleanup old predictions periodically
CREATE OR REPLACE FUNCTION cleanup_expired_predictions()
RETURNS void AS $$
BEGIN
    DELETE FROM fee_predictions WHERE expires_at < CURRENT_TIMESTAMP;
END;
$$ LANGUAGE plpgsql;
```

### 2.2 Views for Analytics

```sql
-- Real-time network status
CREATE VIEW v_current_network_status AS
SELECT 
    collected_at,
    current_tps,
    slot_time_ms,
    congestion_level,
    median_priority_fee,
    p90_priority_fee,
    failed_tx_rate
FROM network_metrics
WHERE collected_at > CURRENT_TIMESTAMP - INTERVAL '5 minutes'
ORDER BY collected_at DESC
LIMIT 1;

-- Fee effectiveness analysis
CREATE VIEW v_fee_effectiveness AS
SELECT 
    DATE_TRUNC('hour', submitted_at) as hour,
    network_congestion,
    COUNT(*) as total_transactions,
    AVG(priority_fee_paid) as avg_fee_paid,
    AVG(median_fee_at_submission) as avg_median_fee,
    AVG(CASE WHEN success THEN priority_fee_paid ELSE NULL END) as avg_fee_success,
    AVG(CASE WHEN success THEN confirmation_time_ms ELSE NULL END) as avg_confirmation_time,
    SUM(CASE WHEN success THEN 1 ELSE 0 END)::DECIMAL / COUNT(*) * 100 as success_rate,
    AVG(overpayment_lamports) as avg_overpayment,
    SUM(CASE WHEN underpayment_risk THEN 1 ELSE 0 END) as underpayment_count
FROM fee_history
WHERE submitted_at > CURRENT_TIMESTAMP - INTERVAL '24 hours'
GROUP BY DATE_TRUNC('hour', submitted_at), network_congestion
ORDER BY hour DESC;

-- Optimal timing recommendations
CREATE VIEW v_optimal_timing AS
SELECT 
    hour_of_day,
    day_of_week,
    CASE day_of_week
        WHEN 0 THEN 'Sunday'
        WHEN 1 THEN 'Monday'
        WHEN 2 THEN 'Tuesday'
        WHEN 3 THEN 'Wednesday'
        WHEN 4 THEN 'Thursday'
        WHEN 5 THEN 'Friday'
        WHEN 6 THEN 'Saturday'
    END as day_name,
    avg_median_fee,
    avg_p90_fee,
    avg_tps,
    avg_confirmation_time_ms,
    success_rate,
    recommended_min_fee,
    optimal_window,
    transactions_analyzed
FROM timing_windows
WHERE optimal_window = true
ORDER BY day_of_week, hour_of_day;

-- Fee prediction accuracy
CREATE VIEW v_prediction_accuracy AS
SELECT 
    fp.priority_level,
    fp.predicted_at,
    fp.recommended_fee,
    fp.confidence_score,
    fh.priority_fee_paid as actual_fee_paid,
    fh.success,
    fh.confirmation_time_ms as actual_confirmation_time,
    ABS(fp.recommended_fee - fh.priority_fee_paid) as fee_difference,
    CASE 
        WHEN fh.priority_fee_paid BETWEEN fp.min_safe_fee AND fp.max_reasonable_fee THEN true
        ELSE false
    END as within_predicted_range
FROM fee_predictions fp
JOIN fee_history fh ON 
    fh.submitted_at BETWEEN fp.predicted_at AND fp.expires_at
    AND ABS(EXTRACT(EPOCH FROM (fh.submitted_at - fp.predicted_at))) < 300 -- Within 5 minutes
WHERE fp.predicted_at > CURRENT_TIMESTAMP - INTERVAL '24 hours'
ORDER BY fp.predicted_at DESC;
```

## 3. Service Implementation

### 3.1 FeeEstimationService

```rust
/// Priority Fee Estimation Service
/// 
/// Provides dynamic priority fee calculations based on real-time network
/// conditions, historical data, and transaction priority requirements.
pub struct FeeEstimationService {
    db_pool: Arc<PgPool>,
    rpc_client: Arc<RpcClient>,
    metrics_collector: Arc<NetworkMetricsCollector>,
    historical_analyzer: Arc<HistoricalFeeAnalyzer>,
    timing_optimizer: Arc<TimingOptimizer>,
    config: FeeEstimationConfig,
    
    // Caching
    prediction_cache: Arc<RwLock<HashMap<String, FeeEstimate>>>,
    last_network_update: Arc<RwLock<Instant>>,
}

#[derive(Debug, Clone)]
pub struct FeeEstimationConfig {
    /// How often to collect network metrics (seconds)
    pub metrics_collection_interval: u64,
    
    /// How long to cache fee predictions (seconds)
    pub prediction_cache_ttl: u64,
    
    /// Safety multiplier for fee estimates (1.0 = no margin, 1.5 = 50% margin)
    pub safety_multiplier: f64,
    
    /// Maximum fee willing to pay (lamports)
    pub max_fee_limit: u64,
    
    /// Minimum fee to ever use (lamports)
    pub min_fee_floor: u64,
    
    /// Target confirmation time by priority (milliseconds)
    pub target_confirmation_times: HashMap<TransactionPriority, u64>,
    
    /// Enable historical learning
    pub use_historical_data: bool,
    
    /// Enable timing optimization
    pub use_timing_optimization: bool,
}

impl Default for FeeEstimationConfig {
    fn default() -> Self {
        let mut target_times = HashMap::new();
        target_times.insert(TransactionPriority::Critical, 500);    // 0.5s
        target_times.insert(TransactionPriority::High, 2000);       // 2s
        target_times.insert(TransactionPriority::Medium, 5000);     // 5s
        target_times.insert(TransactionPriority::Low, 10000);       // 10s
        
        Self {
            metrics_collection_interval: 30,  // 30 seconds
            prediction_cache_ttl: 60,         // 1 minute
            safety_multiplier: 1.2,           // 20% safety margin
            max_fee_limit: 100_000,           // 0.0001 SOL
            min_fee_floor: 1_000,             // 0.000001 SOL
            target_confirmation_times: target_times,
            use_historical_data: true,
            use_timing_optimization: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FeeEstimate {
    /// Recommended priority fee (lamports per compute unit)
    pub recommended_fee: u64,
    
    /// Minimum safe fee for this priority level
    pub min_safe_fee: u64,
    
    /// Maximum reasonable fee (beyond this is wasteful)
    pub max_reasonable_fee: u64,
    
    /// Expected confirmation time (milliseconds)
    pub expected_confirmation_time: u64,
    
    /// Confidence in this estimate (0.0 - 1.0)
    pub confidence_score: f64,
    
    /// Current network congestion level
    pub network_congestion: CongestionLevel,
    
    /// When this estimate was generated
    pub generated_at: DateTime<Utc>,
    
    /// When this estimate expires
    pub expires_at: DateTime<Utc>,
    
    /// Number of recent samples used
    pub sample_size: usize,
    
    /// Optional: Suggested delay before submission (seconds)
    pub suggested_delay: Option<u64>,
    
    /// Optional: Reason for the recommendation
    pub recommendation_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CongestionLevel {
    Low,      // < 500 TPS, fast confirmations
    Medium,   // 500-1500 TPS, normal conditions
    High,     // 1500-2500 TPS, slower confirmations
    Critical, // > 2500 TPS or high failure rate
}

impl FeeEstimationService {
    /// Create new fee estimation service
    pub async fn new(
        db_pool: Arc<PgPool>,
        rpc_client: Arc<RpcClient>,
        config: FeeEstimationConfig,
    ) -> Result<Self, ServiceError> {
        let metrics_collector = Arc::new(
            NetworkMetricsCollector::new(db_pool.clone(), rpc_client.clone())
        );
        
        let historical_analyzer = Arc::new(
            HistoricalFeeAnalyzer::new(db_pool.clone())
        );
        
        let timing_optimizer = Arc::new(
            TimingOptimizer::new(db_pool.clone())
        );
        
        Ok(Self {
            db_pool,
            rpc_client,
            metrics_collector,
            historical_analyzer,
            timing_optimizer,
            config,
            prediction_cache: Arc::new(RwLock::new(HashMap::new())),
            last_network_update: Arc::new(RwLock::new(Instant::now())),
        })
    }
    
    /// Estimate priority fee for a transaction
    pub async fn estimate_priority_fee(
        &self,
        priority: TransactionPriority,
        compute_units: u32,
    ) -> Result<FeeEstimate, ServiceError> {
        // Check cache first
        let cache_key = format!("{:?}_{}", priority, compute_units);
        if let Some(cached) = self.get_cached_estimate(&cache_key).await {
            if cached.expires_at > Utc::now() {
                return Ok(cached);
            }
        }
        
        // Ensure we have recent network metrics
        self.ensure_fresh_metrics().await?;
        
        // Get current network state
        let network_state = self.get_current_network_state().await?;
        
        // Calculate base fee from network metrics
        let base_fee = self.calculate_base_fee(&network_state, priority).await?;
        
        // Adjust based on historical data
        let historical_adjustment = if self.config.use_historical_data {
            self.historical_analyzer
                .get_fee_adjustment(priority, &network_state)
                .await?
        } else {
            1.0
        };
        
        // Apply safety multiplier
        let adjusted_fee = (base_fee as f64 * historical_adjustment * self.config.safety_multiplier) as u64;
        
        // Clamp to configured limits
        let recommended_fee = adjusted_fee
            .max(self.config.min_fee_floor)
            .min(self.config.max_fee_limit);
        
        // Calculate fee range
        let min_safe_fee = (recommended_fee as f64 * 0.8) as u64;
        let max_reasonable_fee = (recommended_fee as f64 * 1.5) as u64;
        
        // Get expected confirmation time
        let expected_confirmation_time = self.estimate_confirmation_time(
            priority,
            recommended_fee,
            &network_state,
        ).await?;
        
        // Check if timing optimization suggests a delay
        let suggested_delay = if self.config.use_timing_optimization {
            self.timing_optimizer
                .suggest_submission_delay(priority, &network_state)
                .await?
        } else {
            None
        };
        
        // Calculate confidence score
        let confidence_score = self.calculate_confidence_score(&network_state).await?;
        
        // Build recommendation reason
        let recommendation_reason = self.build_recommendation_reason(
            priority,
            &network_state,
            recommended_fee,
            suggested_delay,
        );
        
        let estimate = FeeEstimate {
            recommended_fee,
            min_safe_fee,
            max_reasonable_fee,
            expected_confirmation_time,
            confidence_score,
            network_congestion: network_state.congestion_level,
            generated_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::seconds(self.config.prediction_cache_ttl as i64),
            sample_size: network_state.sample_size,
            suggested_delay,
            recommendation_reason: Some(recommendation_reason),
        };
        
        // Cache the estimate
        self.cache_estimate(&cache_key, estimate.clone()).await;
        
        // Store prediction for later accuracy analysis
        self.store_prediction(&estimate, priority, compute_units).await?;
        
        Ok(estimate)
    }
    
    /// Record actual transaction outcome for learning
    pub async fn record_transaction_outcome(
        &self,
        signature: &str,
        priority_fee_paid: u64,
        compute_units_used: u32,
        slot_submitted: u64,
        slot_confirmed: Option<u64>,
        success: bool,
        error_message: Option<String>,
    ) -> Result<(), ServiceError> {
        // Get network state at submission time
        let network_state = self.get_network_state_at_slot(slot_submitted).await?;
        
        let confirmation_time_ms = if let Some(confirmed_slot) = slot_confirmed {
            Some(((confirmed_slot - slot_submitted) as f64 * 400.0) as i32) // ~400ms per slot
        } else {
            None
        };
        
        // Calculate overpayment
        let overpayment = if let Some(median) = network_state.median_priority_fee {
            if priority_fee_paid > median {
                Some((priority_fee_paid - median) as i64)
            } else {
                Some(0)
            }
        } else {
            None
        };
        
        let underpayment_risk = network_state.median_priority_fee
            .map(|median| priority_fee_paid < median)
            .unwrap_or(false);
        
        // Store in fee history
        sqlx::query!(
            r#"
            INSERT INTO fee_history (
                submitted_at, transaction_signature, priority_fee_paid, 
                compute_units_used, slot_submitted, slot_confirmed,
                confirmation_time_ms, network_tps, network_congestion,
                median_fee_at_submission, success, error_message,
                overpayment_lamports, underpayment_risk
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            Utc::now(),
            signature,
            priority_fee_paid as i64,
            compute_units_used as i32,
            slot_submitted as i64,
            slot_confirmed.map(|s| s as i64),
            confirmation_time_ms,
            network_state.current_tps.map(|tps| BigDecimal::from_f64(tps).unwrap()),
            network_state.congestion_level.to_string(),
            network_state.median_priority_fee.map(|f| f as i64),
            success,
            error_message,
            overpayment,
            underpayment_risk,
        )
        .execute(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::Database(e.to_string()))?;
        
        Ok(())
    }
    
    /// Get current network metrics summary
    pub async fn get_network_summary(&self) -> Result<NetworkSummary, ServiceError> {
        let state = self.get_current_network_state().await?;
        
        Ok(NetworkSummary {
            congestion_level: state.congestion_level,
            current_tps: state.current_tps.unwrap_or(0.0),
            median_priority_fee: state.median_priority_fee.unwrap_or(0),
            p90_priority_fee: state.p90_priority_fee.unwrap_or(0),
            recommendation: self.get_general_recommendation(&state),
        })
    }
    
    // ========================================================================
    // Private Helper Methods
    // ========================================================================
    
    async fn ensure_fresh_metrics(&self) -> Result<(), ServiceError> {
        let last_update = *self.last_network_update.read().await;
        let elapsed = last_update.elapsed().as_secs();
        
        if elapsed > self.config.metrics_collection_interval {
            self.metrics_collector.collect_and_store().await?;
            *self.last_network_update.write().await = Instant::now();
        }
        
        Ok(())
    }
    
    async fn get_current_network_state(&self) -> Result<NetworkState, ServiceError> {
        let row = sqlx::query!(
            r#"
            SELECT 
                current_tps, slot_time_ms, recent_blockhash,
                min_priority_fee, median_priority_fee, p75_priority_fee,
                p90_priority_fee, max_priority_fee, congestion_level,
                failed_tx_rate, sample_size
            FROM network_metrics
            WHERE collected_at > $1
            ORDER BY collected_at DESC
            LIMIT 1
            "#,
            Utc::now() - chrono::Duration::minutes(5)
        )
        .fetch_optional(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::Database(e.to_string()))?
        .ok_or_else(|| ServiceError::NotFound("No recent network metrics found".to_string()))?;
        
        Ok(NetworkState {
            current_tps: row.current_tps.and_then(|d| d.to_string().parse().ok()),
            slot_time_ms: row.slot_time_ms,
            recent_blockhash: row.recent_blockhash,
            min_priority_fee: Some(row.min_priority_fee as u64),
            median_priority_fee: Some(row.median_priority_fee as u64),
            p75_priority_fee: Some(row.p75_priority_fee as u64),
            p90_priority_fee: Some(row.p90_priority_fee as u64),
            max_priority_fee: Some(row.max_priority_fee as u64),
            congestion_level: CongestionLevel::from_str(&row.congestion_level)?,
            failed_tx_rate: row.failed_tx_rate.and_then(|d| d.to_string().parse().ok()),
            sample_size: row.sample_size as usize,
        })
    }
    
    async fn calculate_base_fee(
        &self,
        network_state: &NetworkState,
        priority: TransactionPriority,
    ) -> Result<u64, ServiceError> {
        // Select fee percentile based on priority
        let base_fee = match priority {
            TransactionPriority::Critical => network_state.p90_priority_fee.unwrap_or(10_000),
            TransactionPriority::High => network_state.p75_priority_fee.unwrap_or(5_000),
            TransactionPriority::Medium => network_state.median_priority_fee.unwrap_or(2_000),
            TransactionPriority::Low => network_state.min_priority_fee.unwrap_or(1_000),
        };
        
        Ok(base_fee)
    }
    
    async fn estimate_confirmation_time(
        &self,
        priority: TransactionPriority,
        fee: u64,
        network_state: &NetworkState,
    ) -> Result<u64, ServiceError> {
        // Base time from priority configuration
        let base_time = self.config.target_confirmation_times
            .get(&priority)
            .copied()
            .unwrap_or(5000);
        
        // Adjust based on congestion
        let congestion_multiplier = match network_state.congestion_level {
            CongestionLevel::Low => 0.8,
            CongestionLevel::Medium => 1.0,
            CongestionLevel::High => 1.5,
            CongestionLevel::Critical => 2.5,
        };
        
        Ok((base_time as f64 * congestion_multiplier) as u64)
    }
    
    async fn calculate_confidence_score(&self, network_state: &NetworkState) -> Result<f64, ServiceError> {
        let mut score = 1.0;
        
        // Reduce confidence if sample size is small
        if network_state.sample_size < 10 {
            score *= 0.5;
        } else if network_state.sample_size < 50 {
            score *= 0.8;
        }
        
        // Reduce confidence in high congestion
        match network_state.congestion_level {
            CongestionLevel::Critical => score *= 0.6,
            CongestionLevel::High => score *= 0.8,
            _ => {}
        }
        
        // Reduce confidence if high failure rate
        if let Some(failure_rate) = network_state.failed_tx_rate {
            if failure_rate > 5.0 {
                score *= 0.7;
            }
        }
        
        Ok(score.max(0.0).min(1.0))
    }
    
    fn build_recommendation_reason(
        &self,
        priority: TransactionPriority,
        network_state: &NetworkState,
        recommended_fee: u64,
        suggested_delay: Option<u64>,
    ) -> String {
        let mut reason = format!(
            "{:?} priority in {:?} congestion. ",
            priority, network_state.congestion_level
        );
        
        if let Some(delay) = suggested_delay {
            reason.push_str(&format!(
                "Consider delaying {}s for better fees. ",
                delay
            ));
        }
        
        if network_state.congestion_level == CongestionLevel::Critical {
            reason.push_str("Network heavily congested - fees elevated. ");
        }
        
        reason
    }
    
    async fn get_cached_estimate(&self, key: &str) -> Option<FeeEstimate> {
        self.prediction_cache.read().await.get(key).cloned()
    }
    
    async fn cache_estimate(&self, key: &str, estimate: FeeEstimate) {
        self.prediction_cache.write().await.insert(key.to_string(), estimate);
    }
    
    async fn store_prediction(
        &self,
        estimate: &FeeEstimate,
        priority: TransactionPriority,
        compute_units: u32,
    ) -> Result<(), ServiceError> {
        sqlx::query!(
            r#"
            INSERT INTO fee_predictions (
                predicted_at, expires_at, priority_level, target_confirmation_time_ms,
                recommended_fee, min_safe_fee, max_reasonable_fee,
                confidence_score, based_on_samples, network_congestion, current_tps
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            estimate.generated_at,
            estimate.expires_at,
            format!("{:?}", priority),
            estimate.expected_confirmation_time as i32,
            estimate.recommended_fee as i64,
            estimate.min_safe_fee as i64,
            estimate.max_reasonable_fee as i64,
            BigDecimal::from_f64(estimate.confidence_score).unwrap(),
            estimate.sample_size as i32,
            format!("{:?}", estimate.network_congestion),
            estimate.network_congestion as i32, // Placeholder
        )
        .execute(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::Database(e.to_string()))?;
        
        Ok(())
    }
    
    fn get_general_recommendation(&self, state: &NetworkState) -> String {
        match state.congestion_level {
            CongestionLevel::Low => "Excellent time for transactions - low fees and fast confirmations".to_string(),
            CongestionLevel::Medium => "Normal network conditions - standard fees apply".to_string(),
            CongestionLevel::High => "Network busy - consider batching or delaying non-urgent transactions".to_string(),
            CongestionLevel::Critical => "Network heavily congested - only submit critical transactions".to_string(),
        }
    }
    
    async fn get_network_state_at_slot(&self, slot: u64) -> Result<NetworkState, ServiceError> {
        // Query historical network metrics around the given slot
        // For now, return current state (would need slot-to-timestamp mapping)
        self.get_current_network_state().await
    }
}

#[derive(Debug)]
struct NetworkState {
    current_tps: Option<f64>,
    slot_time_ms: i32,
    recent_blockhash: String,
    min_priority_fee: Option<u64>,
    median_priority_fee: Option<u64>,
    p75_priority_fee: Option<u64>,
    p90_priority_fee: Option<u64>,
    max_priority_fee: Option<u64>,
    congestion_level: CongestionLevel,
    failed_tx_rate: Option<f64>,
    sample_size: usize,
}

#[derive(Debug)]
pub struct NetworkSummary {
    pub congestion_level: CongestionLevel,
    pub current_tps: f64,
    pub median_priority_fee: u64,
    pub p90_priority_fee: u64,
    pub recommendation: String,
}

impl CongestionLevel {
    fn from_str(s: &str) -> Result<Self, ServiceError> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "critical" => Ok(Self::Critical),
            _ => Err(ServiceError::InvalidInput(format!("Invalid congestion level: {}", s))),
        }
    }
    
    fn to_string(&self) -> String {
        match self {
            Self::Low => "low".to_string(),
            Self::Medium => "medium".to_string(),
            Self::High => "high".to_string(),
            Self::Critical => "critical".to_string(),
        }
    }
}
```

### 3.2 NetworkMetricsCollector

```rust
/// Collects real-time network metrics from Solana RPC
pub struct NetworkMetricsCollector {
    db_pool: Arc<PgPool>,
    rpc_client: Arc<RpcClient>,
}

impl NetworkMetricsCollector {
    pub fn new(db_pool: Arc<PgPool>, rpc_client: Arc<RpcClient>) -> Self {
        Self { db_pool, rpc_client }
    }
    
    /// Collect current network metrics and store in database
    pub async fn collect_and_store(&self) -> Result<(), ServiceError> {
        // Get recent performance samples
        let performance_samples = self.rpc_client
            .get_recent_performance_samples(Some(10))
            .await
            .map_err(|e| ServiceError::Blockchain(e.to_string()))?;
        
        // Calculate average TPS
        let avg_tps = if !performance_samples.is_empty() {
            performance_samples.iter()
                .map(|s| s.num_transactions as f64 / s.sample_period_secs as f64)
                .sum::<f64>() / performance_samples.len() as f64
        } else {
            0.0
        };
        
        // Get recent priority fees
        let recent_fees = self.collect_recent_priority_fees().await?;
        
        // Determine congestion level
        let congestion_level = self.determine_congestion_level(avg_tps, &recent_fees);
        
        // Get recent blockhash
        let recent_blockhash = self.rpc_client
            .get_latest_blockhash()
            .await
            .map_err(|e| ServiceError::Blockchain(e.to_string()))?
            .to_string();
        
        // Calculate slot time (approximate)
        let slot_time_ms = 400; // Solana targets ~400ms per slot
        
        // Store metrics
        sqlx::query!(
            r#"
            INSERT INTO network_metrics (
                collected_at, current_tps, slot_time_ms, recent_blockhash,
                min_priority_fee, median_priority_fee, p75_priority_fee,
                p90_priority_fee, max_priority_fee, congestion_level,
                sample_size, rpc_node
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            Utc::now(),
            BigDecimal::from_f64(avg_tps).unwrap(),
            slot_time_ms,
            recent_blockhash,
            recent_fees.min as i64,
            recent_fees.median as i64,
            recent_fees.p75 as i64,
            recent_fees.p90 as i64,
            recent_fees.max as i64,
            congestion_level.to_string(),
            recent_fees.sample_size as i32,
            "default",  // TODO: Get actual RPC node URL
        )
        .execute(&*self.db_pool)
        .await
        .map_err(|e| ServiceError::Database(e.to_string()))?;
        
        Ok(())
    }
    
    async fn collect_recent_priority_fees(&self) -> Result<FeeDistribution, ServiceError> {
        // In a real implementation, would sample recent transactions
        // For now, using placeholder values
        // TODO: Implement actual fee sampling from recent blocks
        
        Ok(FeeDistribution {
            min: 1_000,
            median: 5_000,
            p75: 10_000,
            p90: 20_000,
            max: 100_000,
            sample_size: 100,
        })
    }
    
    fn determine_congestion_level(&self, tps: f64, fees: &FeeDistribution) -> CongestionLevel {
        if tps > 2500.0 || fees.median > 50_000 {
            CongestionLevel::Critical
        } else if tps > 1500.0 || fees.median > 20_000 {
            CongestionLevel::High
        } else if tps > 500.0 {
            CongestionLevel::Medium
        } else {
            CongestionLevel::Low
        }
    }
}

#[derive(Debug)]
struct FeeDistribution {
    min: u64,
    median: u64,
    p75: u64,
    p90: u64,
    max: u64,
    sample_size: usize,
}
```

## 4. Integration with Week 2

### 4.1 BatchTransactionService Integration

```rust
// In BatchTransactionService

use crate::services::fee_estimation_service::{FeeEstimationService, TransactionPriority};

pub struct BatchTransactionService {
    // ... existing fields ...
    fee_service: Option<Arc<FeeEstimationService>>,
}

impl BatchTransactionService {
    pub async fn new_with_fee_estimation(
        db_pool: Arc<PgPool>,
        rpc_client: Arc<RpcClient>,
        settlement_service: Arc<SettlementBlockchainService>,
        config: BatchConfig,
        fee_config: FeeEstimationConfig,
    ) -> Result<Self, ServiceError> {
        let fee_service = Arc::new(
            FeeEstimationService::new(
                db_pool.clone(),
                rpc_client.clone(),
                fee_config,
            ).await?
        );
        
        Ok(Self {
            db_pool,
            rpc_client,
            settlement_service,
            config,
            fee_service: Some(fee_service),
        })
    }
    
    async fn submit_batch_with_dynamic_fees(
        &self,
        batch: &BatchTransaction,
        transactions: &[TransactionQueueItem],
    ) -> Result<String, ServiceError> {
        // Estimate optimal priority fee
        let priority = self.determine_batch_priority(transactions);
        let compute_units = self.estimate_compute_units(transactions);
        
        let fee_estimate = if let Some(fee_service) = &self.fee_service {
            fee_service.estimate_priority_fee(priority, compute_units).await?
        } else {
            // Fallback to static fee
            return self.submit_batch_static_fee(batch, transactions).await;
        };
        
        // Check if we should delay submission
        if let Some(delay) = fee_estimate.suggested_delay {
            if delay > 0 && priority != TransactionPriority::Critical {
                info!(
                    "Delaying batch {} submission by {}s for better fees (current: {} lamports, expected savings: ~20%)",
                    batch.id, delay, fee_estimate.recommended_fee
                );
                tokio::time::sleep(Duration::from_secs(delay)).await;
                
                // Re-estimate after delay
                let new_estimate = self.fee_service.as_ref().unwrap()
                    .estimate_priority_fee(priority, compute_units).await?;
                
                return self.submit_with_fee_estimate(batch, transactions, &new_estimate).await;
            }
        }
        
        // Submit with recommended fee
        self.submit_with_fee_estimate(batch, transactions, &fee_estimate).await
    }
    
    async fn submit_with_fee_estimate(
        &self,
        batch: &BatchTransaction,
        transactions: &[TransactionQueueItem],
        fee_estimate: &FeeEstimate,
    ) -> Result<String, ServiceError> {
        info!(
            "Submitting batch {} with dynamic priority fee: {} lamports (confidence: {:.2}, congestion: {:?})",
            batch.id,
            fee_estimate.recommended_fee,
            fee_estimate.confidence_score,
            fee_estimate.network_congestion
        );
        
        // Build transaction with priority fee
        let signature = self.settlement_service
            .submit_batch_settlement(
                transactions,
                Some(fee_estimate.recommended_fee),
            ).await?;
        
        // Record outcome for learning
        if let Some(fee_service) = &self.fee_service {
            // This would be called after confirmation
            // For now, just store the signature
            self.pending_fee_tracking.write().await.insert(
                signature.clone(),
                (fee_estimate.clone(), Utc::now()),
            );
        }
        
        Ok(signature)
    }
    
    fn determine_batch_priority(&self, transactions: &[TransactionQueueItem]) -> TransactionPriority {
        // Determine priority based on transaction types and ages
        if transactions.iter().any(|t| t.priority == TransactionPriority::Critical) {
            TransactionPriority::Critical
        } else if transactions.iter().any(|t| t.priority == TransactionPriority::High) {
            TransactionPriority::High
        } else if transactions.iter().all(|t| t.priority == TransactionPriority::Low) {
            TransactionPriority::Low
        } else {
            TransactionPriority::Medium
        }
    }
}
```

## 5. Configuration

### 5.1 Environment Variables

```bash
# Week 3: Priority Fee System Configuration

# Fee Estimation
ENABLE_DYNAMIC_FEES=true
METRICS_COLLECTION_INTERVAL_SECS=30
FEE_PREDICTION_CACHE_TTL_SECS=60
FEE_SAFETY_MULTIPLIER=1.2
MAX_PRIORITY_FEE_LAMPORTS=100000
MIN_PRIORITY_FEE_LAMPORTS=1000

# Target confirmation times by priority (milliseconds)
TARGET_CONFIRMATION_TIME_CRITICAL_MS=500
TARGET_CONFIRMATION_TIME_HIGH_MS=2000
TARGET_CONFIRMATION_TIME_MEDIUM_MS=5000
TARGET_CONFIRMATION_TIME_LOW_MS=10000

# Learning and optimization
USE_HISTORICAL_FEE_DATA=true
USE_TIMING_OPTIMIZATION=true
FEE_HISTORY_RETENTION_DAYS=30

# Network monitoring
NETWORK_CONGESTION_THRESHOLD_LOW_TPS=500
NETWORK_CONGESTION_THRESHOLD_MEDIUM_TPS=1500
NETWORK_CONGESTION_THRESHOLD_HIGH_TPS=2500
```

### 5.2 Runtime Configuration

```rust
// In config.rs or service initialization

pub fn load_fee_estimation_config() -> FeeEstimationConfig {
    let mut target_times = HashMap::new();
    target_times.insert(
        TransactionPriority::Critical,
        std::env::var("TARGET_CONFIRMATION_TIME_CRITICAL_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(500),
    );
    target_times.insert(
        TransactionPriority::High,
        std::env::var("TARGET_CONFIRMATION_TIME_HIGH_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(2000),
    );
    target_times.insert(
        TransactionPriority::Medium,
        std::env::var("TARGET_CONFIRMATION_TIME_MEDIUM_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5000),
    );
    target_times.insert(
        TransactionPriority::Low,
        std::env::var("TARGET_CONFIRMATION_TIME_LOW_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(10000),
    );
    
    FeeEstimationConfig {
        metrics_collection_interval: std::env::var("METRICS_COLLECTION_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30),
        prediction_cache_ttl: std::env::var("FEE_PREDICTION_CACHE_TTL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60),
        safety_multiplier: std::env::var("FEE_SAFETY_MULTIPLIER")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1.2),
        max_fee_limit: std::env::var("MAX_PRIORITY_FEE_LAMPORTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(100_000),
        min_fee_floor: std::env::var("MIN_PRIORITY_FEE_LAMPORTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1_000),
        target_confirmation_times: target_times,
        use_historical_data: std::env::var("USE_HISTORICAL_FEE_DATA")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true),
        use_timing_optimization: std::env::var("USE_TIMING_OPTIMIZATION")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(true),
    }
}
```

## 6. Testing Strategy

### 6.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fee_estimation_low_congestion() {
        // Test fee estimation in low congestion
    }
    
    #[tokio::test]
    async fn test_fee_estimation_high_congestion() {
        // Test fee estimation in high congestion
    }
    
    #[tokio::test]
    async fn test_confidence_score_calculation() {
        // Test confidence score logic
    }
    
    #[tokio::test]
    async fn test_timing_optimization() {
        // Test suggested delay calculation
    }
    
    #[tokio::test]
    async fn test_historical_learning() {
        // Test fee adjustment based on history
    }
}
```

### 6.2 Integration Tests

```rust
#[tokio::test]
async fn test_end_to_end_fee_estimation() {
    // Test full flow: collect metrics -> estimate fee -> submit -> record outcome
}

#[tokio::test]
async fn test_batch_service_integration() {
    // Test BatchTransactionService using FeeEstimationService
}
```

## 7. Monitoring & Metrics

### 7.1 Prometheus Metrics

```rust
// Fee estimation metrics
static FEE_ESTIMATES_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    register_int_counter_vec!(
        "gridtokenx_fee_estimates_total",
        "Total number of fee estimates generated",
        &["priority", "congestion_level"]
    ).unwrap()
});

static FEE_ESTIMATE_ACCURACY: LazyLock<HistogramVec> = LazyLock::new(|| {
    register_histogram_vec!(
        "gridtokenx_fee_estimate_accuracy",
        "Difference between estimated and actual fees",
        &["priority"],
        vec![0.0, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0]
    ).unwrap()
});

static NETWORK_CONGESTION_LEVEL: LazyLock<IntGauge> = LazyLock::new(|| {
    register_int_gauge!(
        "gridtokenx_network_congestion_level",
        "Current network congestion level (0=low, 1=medium, 2=high, 3=critical)"
    ).unwrap()
});

static FEE_SAVINGS_TOTAL: LazyLock<Counter> = LazyLock::new(|| {
    register_counter!(
        "gridtokenx_fee_savings_total_lamports",
        "Total lamports saved through fee optimization"
    ).unwrap()
});
```

### 7.2 Grafana Dashboard

```json
{
  "dashboard": {
    "title": "Week 3: Priority Fee System",
    "panels": [
      {
        "title": "Network Congestion Level",
        "targets": ["gridtokenx_network_congestion_level"]
      },
      {
        "title": "Fee Estimate Accuracy",
        "targets": ["gridtokenx_fee_estimate_accuracy"]
      },
      {
        "title": "Total Fee Savings",
        "targets": ["gridtokenx_fee_savings_total_lamports"]
      },
      {
        "title": "Fee Estimates by Priority",
        "targets": ["gridtokenx_fee_estimates_total"]
      }
    ]
  }
}
```

## 8. Implementation Checklist

### Phase 1: Core Infrastructure (Days 1-2)
- [ ] Create database migration for Week 3 schema
  - [ ] `network_metrics` table
  - [ ] `fee_history` table
  - [ ] `timing_windows` table
  - [ ] `fee_predictions` table
  - [ ] Analytics views
- [ ] Implement `FeeEstimationService` skeleton
- [ ] Implement `NetworkMetricsCollector`
- [ ] Add configuration loading

### Phase 2: Fee Logic (Days 3-4)
- [ ] Implement base fee calculation
- [ ] Add congestion level detection
- [ ] Implement confidence score calculation
- [ ] Add fee range calculation (min/max)
- [ ] Implement caching layer

### Phase 3: Learning & Optimization (Day 5)
- [ ] Implement `HistoricalFeeAnalyzer`
- [ ] Implement `TimingOptimizer`
- [ ] Add transaction outcome recording
- [ ] Create prediction accuracy tracking

### Phase 4: Integration (Day 6)
- [ ] Integrate FeeEstimationService with BatchTransactionService
- [ ] Update settlement service for dynamic fees
- [ ] Add Prometheus metrics
- [ ] Create Grafana dashboard

### Phase 5: Testing & Documentation (Day 7)
- [ ] Write unit tests (50+ tests)
- [ ] Write integration tests
- [ ] Create usage documentation
- [ ] Update configuration guides
- [ ] Performance testing

## 9. Expected Outcomes

### Performance Targets

- **Fee Reduction**: 20-40% lower average priority fees vs static pricing
- **Confidence Score**: > 0.8 average confidence in non-critical congestion
- **Prediction Accuracy**: < 20% difference between estimated and actual fees
- **Cost Savings**: $50-100/day in reduced transaction fees (at 1000 tx/day)

### Success Metrics

1. **Fee Optimization**
   - Average fee paid vs network median
   - Fee savings compared to conservative static pricing
   - Percentage of transactions within predicted fee range

2. **Reliability**
   - Transaction success rate with dynamic fees
   - Confirmation time variance from target
   - Percentage of transactions requiring fee boost

3. **Learning Performance**
   - Prediction accuracy improvement over time
   - Historical data utilization rate
   - Timing optimization hit rate

## 10. Risk Mitigation

### Technical Risks

1. **Network Volatility**: Sudden congestion spikes
   - Mitigation: Safety multiplier, maximum fee caps

2. **RPC Reliability**: Metrics collection failures
   - Mitigation: Fallback to cached values, multiple RPC endpoints

3. **Learning Accuracy**: Historical data not predictive
   - Mitigation: Confidence scores, ability to disable historical learning

### Operational Risks

1. **Fee Underpayment**: Transactions failing due to low fees
   - Mitigation: Minimum fee floors, underpayment detection

2. **Cost Overruns**: Maximum fee limits too high
   - Mitigation: Configurable caps, alerts on high fees

## 11. Next Steps (Week 4 Preview)

After Week 3 completion:

- **Week 4**: Transaction Retry Logic & Error Recovery
  - Automatic retry with fee escalation
  - Exponential backoff
  - Circuit breaker patterns
  - Transaction cancellation and replacement

---

**Document Version**: 1.0  
**Last Updated**: 2025-01-10  
**Status**: Ready for Implementation  
**Estimated Completion**: 2025-01-17
