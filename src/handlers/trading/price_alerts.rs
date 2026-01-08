//! Price Alerts Handler
//!
//! Handles creation, listing, and deletion of price alerts

use axum::{extract::{State, Path}, response::Json};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use tracing::{info, error};
use utoipa::ToSchema;
use validator::Validate;

use crate::auth::middleware::AuthenticatedUser;
use crate::error::{ApiError, Result};
use crate::AppState;

/// Alert condition type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "alert_condition", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AlertCondition {
    /// Trigger when price goes above target
    Above,
    /// Trigger when price goes below target
    Below,
    /// Trigger when price crosses target in either direction
    Crosses,
}

/// Alert status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, ToSchema)]
#[sqlx(type_name = "alert_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    Active,
    Triggered,
    Cancelled,
}

/// Price alert record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
pub struct PriceAlert {
    pub id: Uuid,
    pub user_id: Uuid,
    #[schema(value_type = String)]
    pub target_price: Decimal,
    pub condition: AlertCondition,
    pub status: AlertStatus,
    pub triggered_at: Option<DateTime<Utc>>,
    #[schema(value_type = String)]
    pub triggered_price: Option<Decimal>,
    pub repeat: bool,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Request to create a price alert
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreatePriceAlertRequest {
    /// Target price that triggers the alert
    #[schema(value_type = String, example = "0.15")]
    pub target_price: Decimal,
    
    /// Condition: above, below, or crosses
    pub condition: AlertCondition,
    
    /// Re-arm alert after triggering
    pub repeat: Option<bool>,
    
    /// User note for this alert
    pub note: Option<String>,
}

/// Response for price alert creation
#[derive(Debug, Serialize, ToSchema)]
pub struct PriceAlertResponse {
    pub id: Uuid,
    #[schema(value_type = String)]
    pub target_price: Decimal,
    pub condition: AlertCondition,
    pub status: AlertStatus,
    pub created_at: DateTime<Utc>,
    pub message: String,
}

/// Create a new price alert
/// POST /api/v1/trading/price-alerts
#[utoipa::path(
    post,
    path = "/api/v1/trading/price-alerts",
    tag = "trading",
    request_body = CreatePriceAlertRequest,
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Price alert created", body = PriceAlertResponse),
        (status = 400, description = "Invalid parameters"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_price_alert(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreatePriceAlertRequest>,
) -> Result<Json<PriceAlertResponse>> {
    info!("Creating price alert for user: {}, price: {}, condition: {:?}", 
          user.0.sub, payload.target_price, payload.condition);

    if payload.target_price <= Decimal::ZERO {
        return Err(ApiError::BadRequest("Target price must be positive".to_string()));
    }

    let alert_id = Uuid::new_v4();
    let now = Utc::now();

    let result = sqlx::query!(
        r#"
        INSERT INTO price_alerts (id, user_id, target_price, condition, status, repeat, note, created_at)
        VALUES ($1, $2, $3, $4, 'active', $5, $6, $7)
        "#,
        alert_id,
        user.0.sub,
        payload.target_price,
        payload.condition as AlertCondition,
        payload.repeat.unwrap_or(false),
        payload.note,
        now
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to create price alert: {}", e);
        ApiError::Internal(format!("Failed to create alert: {}", e))
    })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::Internal("Failed to insert alert".to_string()));
    }

    info!("Created price alert {} for user {}", alert_id, user.0.sub);

    Ok(Json(PriceAlertResponse {
        id: alert_id,
        target_price: payload.target_price,
        condition: payload.condition,
        status: AlertStatus::Active,
        created_at: now,
        message: format!(
            "Alert created. You'll be notified when price goes {} {}",
            match payload.condition {
                AlertCondition::Above => "above",
                AlertCondition::Below => "below",
                AlertCondition::Crosses => "to",
            },
            payload.target_price
        ),
    }))
}

/// List user's price alerts
/// GET /api/v1/trading/price-alerts
#[utoipa::path(
    get,
    path = "/api/v1/trading/price-alerts",
    tag = "trading",
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "List of price alerts", body = Vec<PriceAlert>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_price_alerts(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<PriceAlert>>> {
    let alerts = sqlx::query_as!(
        PriceAlert,
        r#"
        SELECT id, user_id, target_price, 
               condition as "condition!: AlertCondition",
               status as "status!: AlertStatus",
               triggered_at, triggered_price, repeat,
               note, created_at as "created_at!"
        FROM price_alerts
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 100
        "#,
        user.0.sub
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to list price alerts: {}", e);
        ApiError::Internal(format!("Failed to list alerts: {}", e))
    })?;

    Ok(Json(alerts))
}

/// Delete a price alert
/// DELETE /api/v1/trading/price-alerts/:id
#[utoipa::path(
    delete,
    path = "/api/v1/trading/price-alerts/{id}",
    tag = "trading",
    params(("id" = Uuid, Path, description = "Alert ID to delete")),
    security(("bearer_auth" = [])),
    responses(
        (status = 200, description = "Alert deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Alert not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_price_alert(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(alert_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    info!("Deleting price alert {} for user {}", alert_id, user.0.sub);

    let result = sqlx::query!(
        "DELETE FROM price_alerts WHERE id = $1 AND user_id = $2",
        alert_id,
        user.0.sub
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to delete price alert: {}", e);
        ApiError::Internal(format!("Failed to delete alert: {}", e))
    })?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound("Alert not found".to_string()));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Price alert deleted"
    })))
}
