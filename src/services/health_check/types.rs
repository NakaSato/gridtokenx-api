use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemMetrics {
    pub cpu_usage: Option<f64>,
    pub memory_used_mb: Option<u64>,
    pub memory_total_mb: Option<u64>,
    pub disk_used_gb: Option<u64>,
    pub disk_total_gb: Option<u64>,
    pub active_connections: u64,
}

/// Detailed health status with metrics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DetailedHealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub environment: String,
    pub uptime_seconds: u64,
    pub dependencies: Vec<DependencyHealth>,
    pub metrics: SystemMetrics,
}

/// Dependency health information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DependencyHealth {
    pub name: String,
    pub status: HealthCheckStatus,
    pub response_time_ms: Option<u64>,
    pub last_check: DateTime<Utc>,
    pub error_message: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum HealthCheckStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
