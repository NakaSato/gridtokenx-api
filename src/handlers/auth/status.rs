//! Status Handlers Module
//!
//! System and service status endpoint handlers with comprehensive health checks.

use axum::{
    extract::State,
    Json,
};
use serde::Serialize;
use std::sync::OnceLock;
use std::time::Instant;

use crate::AppState;

/// Global start time for uptime calculation
static START_TIME: OnceLock<Instant> = OnceLock::new();

fn get_uptime_seconds() -> u64 {
    START_TIME.get_or_init(Instant::now).elapsed().as_secs()
}

/// Comprehensive health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: String,
    pub services: ServiceStatus,
}

/// Status of individual services
#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub database: ServiceHealth,
    pub email: ServiceHealth,
    pub blockchain: ServiceHealth,
}

/// Individual service health
#[derive(Debug, Serialize)]
pub struct ServiceHealth {
    pub status: String,
    pub latency_ms: Option<u64>,
    pub message: Option<String>,
}

/// Simple status response for backward compatibility
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub version: String,
    pub uptime: String,
}

/// Get comprehensive system health
/// GET /api/v1/status
pub async fn system_status(
    State(state): State<AppState>,
) -> Json<HealthResponse> {
    let health = state.health_checker.perform_health_check().await;
    
    // Map dependencies to ServiceStatus
    let mut db_health = ServiceHealth {
        status: "unknown".to_string(),
        latency_ms: None,
        message: None,
    };
    let mut email_health = ServiceHealth {
        status: "unknown".to_string(),
        latency_ms: None,
        message: None,
    };
    let mut blockchain_health = ServiceHealth {
        status: "unknown".to_string(),
        latency_ms: None,
        message: None,
    };

    for dep in health.dependencies {
        match dep.name.as_str() {
            "PostgreSQL" => {
                db_health = ServiceHealth {
                    status: match dep.status {
                        crate::services::health_check::HealthCheckStatus::Healthy => "healthy".to_string(),
                        crate::services::health_check::HealthCheckStatus::Degraded => "degraded".to_string(),
                        crate::services::health_check::HealthCheckStatus::Unhealthy => "unhealthy".to_string(),
                        crate::services::health_check::HealthCheckStatus::Unknown => "unknown".to_string(),
                    },
                    latency_ms: dep.response_time_ms,
                    message: dep.error_message,
                };
            }
            "Email Service" => {
                email_health = ServiceHealth {
                    status: match dep.status {
                        crate::services::health_check::HealthCheckStatus::Healthy => "healthy".to_string(),
                        crate::services::health_check::HealthCheckStatus::Degraded => "disabled".to_string(),
                        crate::services::health_check::HealthCheckStatus::Unhealthy => "unhealthy".to_string(),
                        crate::services::health_check::HealthCheckStatus::Unknown => "unknown".to_string(),
                    },
                    latency_ms: None,
                    message: dep.details,
                };
            }
            "Solana RPC" => {
                blockchain_health = ServiceHealth {
                    status: match dep.status {
                        crate::services::health_check::HealthCheckStatus::Healthy => "healthy".to_string(),
                        crate::services::health_check::HealthCheckStatus::Degraded => "degraded".to_string(),
                        crate::services::health_check::HealthCheckStatus::Unhealthy => "unhealthy".to_string(),
                        crate::services::health_check::HealthCheckStatus::Unknown => "unknown".to_string(),
                    },
                    latency_ms: dep.response_time_ms,
                    message: dep.error_message,
                };
            }
            _ => {}
        }
    }
    
    Json(HealthResponse {
        status: health.status,
        version: health.version,
        uptime_seconds: health.uptime_seconds,
        timestamp: health.timestamp.to_rfc3339(),
        services: ServiceStatus {
            database: db_health,
            email: email_health,
            blockchain: blockchain_health,
        },
    })
}

// These helper functions are now redundant as they are handled by health_checker service
// Removing them to avoid confusion

/// Get meter service status
/// GET /api/v1/status/meters
pub async fn meter_status(
    State(state): State<AppState>,
) -> Json<MeterStatusResponse> {
    let health = state.health_checker.perform_health_check().await;
    
    // Find database status from unified health check
    let db_status = health.dependencies.iter()
        .find(|d| d.name == "PostgreSQL")
        .map(|d| match d.status {
            crate::services::health_check::HealthCheckStatus::Healthy => "healthy".to_string(),
            _ => "unhealthy".to_string(),
        })
        .unwrap_or_else(|| "unknown".to_string());

    let db_latency = health.dependencies.iter()
        .find(|d| d.name == "PostgreSQL")
        .and_then(|d| d.response_time_ms);
    
    // Count meters by status
    let meter_counts = get_meter_counts(&state).await;
    
    Json(MeterStatusResponse {
        status: db_status,
        database_latency_ms: db_latency,
        meter_counts,
    })
}

#[derive(Debug, Serialize)]
pub struct MeterStatusResponse {
    pub status: String,
    pub database_latency_ms: Option<u64>,
    pub meter_counts: MeterCounts,
}

#[derive(Debug, Serialize)]
pub struct MeterCounts {
    pub total: i64,
    pub pending: i64,
    pub verified: i64,
    pub active: i64,
}

async fn get_meter_counts(state: &AppState) -> MeterCounts {
    let counts = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        "SELECT 
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE status = 'pending') as pending,
            COUNT(*) FILTER (WHERE status = 'verified') as verified,
            COUNT(*) FILTER (WHERE status = 'active') as active
         FROM smartmeters"
    )
    .fetch_optional(&state.db)
    .await;
    
    match counts {
        Ok(Some((total, pending, verified, active))) => MeterCounts {
            total,
            pending,
            verified,
            active,
        },
        _ => MeterCounts {
            total: 0,
            pending: 0,
            verified: 0,
            active: 0,
        },
    }
}

/// Simple readiness probe for kubernetes/docker
/// GET /api/v1/status/ready
pub async fn readiness_probe(
    State(state): State<AppState>,
) -> Json<ReadinessResponse> {
    let health = state.health_checker.perform_health_check().await;
    
    let db_passed = health.dependencies.iter()
        .find(|d| d.name == "PostgreSQL")
        .map(|d| d.status == crate::services::health_check::HealthCheckStatus::Healthy)
        .unwrap_or(false);
    
    Json(ReadinessResponse {
        ready: health.status == "healthy",
        checks: vec![
            CheckResult {
                name: "database".to_string(),
                passed: db_passed,
            },
            CheckResult {
                name: "overall".to_string(),
                passed: health.status == "healthy" || health.status == "degraded",
            }
        ],
    })
}

#[derive(Debug, Serialize)]
pub struct ReadinessResponse {
    pub ready: bool,
    pub checks: Vec<CheckResult>,
}

#[derive(Debug, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub passed: bool,
}

/// Simple liveness probe for kubernetes/docker
/// GET /api/v1/status/live
pub async fn liveness_probe() -> Json<LivenessResponse> {
    Json(LivenessResponse {
        alive: true,
        uptime_seconds: get_uptime_seconds(),
    })
}

#[derive(Debug, Serialize)]
pub struct LivenessResponse {
    pub alive: bool,
    pub uptime_seconds: u64,
}
