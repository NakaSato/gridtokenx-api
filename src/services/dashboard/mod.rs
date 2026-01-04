pub mod types;
 
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use crate::services::websocket::WebSocketService;
use crate::services::event_processor::EventProcessorService;
use crate::services::health_check::HealthChecker;
use crate::services::transaction::metrics::MetricsExporter;
pub use types::{DashboardMetrics, GridStatus};

#[derive(Clone)]
pub struct DashboardService {
    db: sqlx::PgPool,
    health_checker: HealthChecker,
    event_processor: EventProcessorService,
    websocket_service: WebSocketService,
    metrics: Arc<RwLock<GridStatus>>,
}

impl DashboardService {
    pub fn new(
        db: sqlx::PgPool,
        health_checker: HealthChecker,
        event_processor: EventProcessorService,
        websocket_service: WebSocketService,
    ) -> Self {
        Self {
            db,
            health_checker,
            event_processor,
            websocket_service,
                metrics: Arc::new(RwLock::new(GridStatus {
                total_generation: 0.0,
                total_consumption: 0.0,
                net_balance: 0.0,
                active_meters: 0,
                co2_saved_kg: 0.0,
                timestamp: Utc::now(),
            })),
        }
    }

    /// Handle a new meter reading to update aggregate grid status and broadcast
    pub async fn handle_meter_reading(&self, kwh: f64, _meter_serial: &str) -> anyhow::Result<()> {
        let mut metrics = self.metrics.write().await;
        
        // Update aggregate totals
        if kwh > 0.0 {
            metrics.total_generation += kwh;
        } else {
            metrics.total_consumption += kwh.abs();
        }

        // Increment active meters if it was 0 or just maintain (simple logic for now)
        // In a real scenario, we'd track specific meter serials
        if metrics.active_meters < 30 { // Cap for simulation realism
             metrics.active_meters += 1;
        }

        metrics.net_balance = metrics.total_generation - metrics.total_consumption;
        metrics.co2_saved_kg = metrics.total_generation * 0.431;
        metrics.timestamp = Utc::now();

        // Broadcast to all connected clients
        let ws = self.websocket_service.clone();
        let gen = metrics.total_generation;
        let cons = metrics.total_consumption;
        let bal = metrics.net_balance;
        let active = metrics.active_meters;
        let co2 = metrics.co2_saved_kg;

        tokio::spawn(async move {
            ws.broadcast_grid_status_updated(gen, cons, bal, active, co2)
                .await;
        });

        Ok(())
    }

    pub async fn get_grid_status(&self) -> GridStatus {
        let metrics: tokio::sync::RwLockReadGuard<'_, GridStatus> = self.metrics.read().await;
        metrics.clone()
    }

    /// Retrieve historical grid status snapshots
    pub async fn get_grid_history(&self, limit: i64) -> anyhow::Result<Vec<GridStatus>> {
        let history = sqlx::query_as::<_, GridStatus>(
            "SELECT total_generation, total_consumption, net_balance, active_meters, co2_saved_kg, timestamp 
             FROM grid_status_history 
             ORDER BY timestamp DESC 
             LIMIT $1"
        )
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        Ok(history)
    }

    /// Start a background task to record grid status snapshots periodically
    pub async fn start_history_recorder(&self) {
        let self_clone = self.clone();
        let interval_secs = std::env::var("GRID_HISTORY_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(60); // Default to 1 minute

        tokio::spawn(async move {
            tracing::info!("🚀 Starting Grid History Recorder (interval: {}s)", interval_secs);
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            
            loop {
                interval.tick().await;
                
                let current = self_clone.get_grid_status().await;
                let snapshot_time = Utc::now();
                
                // Only record if there's some activity or regularly
                let result = sqlx::query(
                    "INSERT INTO grid_status_history (total_generation, total_consumption, net_balance, active_meters, co2_saved_kg, timestamp)
                     VALUES ($1, $2, $3, $4, $5, $6)"
                )
                .bind(current.total_generation)
                .bind(current.total_consumption)
                .bind(current.net_balance)
                .bind(current.active_meters)
                .bind(current.co2_saved_kg)
                .bind(snapshot_time)
                .execute(&self_clone.db)
                .await;

                if let Err(e) = result {
                    tracing::error!("❌ Failed to record grid history snapshot: {}", e);
                }
            }
        });
    }

    pub async fn get_metrics(&self) -> anyhow::Result<DashboardMetrics> {
        // Fetch metrics in parallel where possible
        let (health_status, event_stats) = tokio::join!(
            self.health_checker.perform_health_check(),
            self.event_processor.get_stats()
        );

        let pending_transactions = MetricsExporter::get_transaction_stats();

        Ok(DashboardMetrics {
            system_health: health_status,
            event_processor: event_stats?,
            pending_transactions,
            grid_status: self.get_grid_status().await,
        })
    }
}
