//! Trading Status Endpoints
//!
//! Provides status information for matching engine and settlements

use axum::{extract::State, response::Json};
use serde::Serialize;
use sqlx::Row;
use utoipa::ToSchema;

use crate::error::{ApiError, Result};
use crate::AppState;

/// Matching engine status response
#[derive(Debug, Serialize, ToSchema)]
pub struct MatchingStatus {
    pub pending_buy_orders: i64,
    pub pending_sell_orders: i64,
    pub pending_matches: i64,
    pub buy_price_range: PriceRange,
    pub sell_price_range: PriceRange,
    pub can_match: bool,
    pub match_reason: String,
}

/// Price range for orders
#[derive(Debug, Serialize, ToSchema)]
pub struct PriceRange {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

/// Settlement statistics response
#[derive(Debug, Serialize, ToSchema)]
pub struct SettlementStatusResponse {
    pub pending_count: i64,
    pub processing_count: i64,
    pub completed_count: i64,
    pub failed_count: i64,
    pub total_settled_value: f64,
    pub recent_settlements: Vec<RecentSettlement>,
}

/// Recent settlement info
#[derive(Debug, Serialize, ToSchema)]
pub struct RecentSettlement {
    pub id: String,
    pub status: String,
    pub energy_amount: f64,
    pub total_amount: f64,
    pub created_at: String,
}

/// Get matching engine status
/// GET /api/v1/trading/matching-status
#[utoipa::path(
    get,
    path = "/api/v1/trading/matching-status",
    tag = "trading",
    responses(
        (status = 200, description = "Matching status", body = MatchingStatus),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_matching_status(
    State(state): State<AppState>,
) -> Result<Json<MatchingStatus>> {
    // Get pending order counts
    let order_counts = sqlx::query(
        r#"
        SELECT 
            side::text,
            COUNT(*) as count,
            MIN(price_per_kwh)::float8 as min_price,
            MAX(price_per_kwh)::float8 as max_price
        FROM trading_orders 
        WHERE status = 'pending'
        GROUP BY side
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(ApiError::Database)?;

    let mut pending_buy_orders: i64 = 0;
    let mut pending_sell_orders: i64 = 0;
    let mut buy_min: Option<f64> = None;
    let mut buy_max: Option<f64> = None;
    let mut sell_min: Option<f64> = None;
    let mut sell_max: Option<f64> = None;

    for row in order_counts {
        let side: String = row.get("side");
        let count: i64 = row.get("count");
        let min_price: Option<f64> = row.get("min_price");
        let max_price: Option<f64> = row.get("max_price");

        if side == "buy" {
            pending_buy_orders = count;
            buy_min = min_price;
            buy_max = max_price;
        } else if side == "sell" {
            pending_sell_orders = count;
            sell_min = min_price;
            sell_max = max_price;
        }
    }

    // Get pending matches count
    let pending_matches: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM order_matches WHERE status::text = 'pending'"
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    // Determine if matching is possible
    let (can_match, match_reason) = match (buy_max, sell_min) {
        (Some(buy_max_val), Some(sell_min_val)) => {
            if buy_max_val >= sell_min_val {
                (true, format!(
                    "Matching possible: buy max {:.4} >= sell min {:.4}",
                    buy_max_val, sell_min_val
                ))
            } else {
                (false, format!(
                    "Price gap: buy max {:.4} < sell min {:.4}",
                    buy_max_val, sell_min_val
                ))
            }
        }
        (None, _) => (false, "No pending buy orders".to_string()),
        (_, None) => (false, "No pending sell orders".to_string()),
    };

    Ok(Json(MatchingStatus {
        pending_buy_orders,
        pending_sell_orders,
        pending_matches,
        buy_price_range: PriceRange {
            min: buy_min,
            max: buy_max,
        },
        sell_price_range: PriceRange {
            min: sell_min,
            max: sell_max,
        },
        can_match,
        match_reason,
    }))
}

/// Get settlement statistics
/// GET /api/v1/trading/settlement-stats
#[utoipa::path(
    get,
    path = "/api/v1/trading/settlement-stats",
    tag = "trading",
    responses(
        (status = 200, description = "Settlement statistics", body = SettlementStatusResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_settlement_stats(
    State(state): State<AppState>,
) -> Result<Json<SettlementStatusResponse>> {
    // Get settlement counts by status
    let stats = sqlx::query(
        r#"
        SELECT 
            COUNT(*) FILTER (WHERE status = 'pending') as pending_count,
            COUNT(*) FILTER (WHERE status = 'processing') as processing_count,
            COUNT(*) FILTER (WHERE status = 'completed') as completed_count,
            COUNT(*) FILTER (WHERE status = 'failed') as failed_count,
            COALESCE(SUM(CASE WHEN status = 'completed' THEN total_amount::float8 ELSE 0 END), 0) as total_settled_value
        FROM settlements
        "#
    )
    .fetch_one(&state.db)
    .await
    .map_err(ApiError::Database)?;

    let pending_count: i64 = stats.get("pending_count");
    let processing_count: i64 = stats.get("processing_count");
    let completed_count: i64 = stats.get("completed_count");
    let failed_count: i64 = stats.get("failed_count");
    let total_settled_value: f64 = stats.get("total_settled_value");

    // Get recent settlements
    let recent = sqlx::query(
        r#"
        SELECT id, status, energy_amount::float8 as energy, total_amount::float8 as total, created_at
        FROM settlements
        ORDER BY created_at DESC
        LIMIT 10
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(ApiError::Database)?;

    let recent_settlements: Vec<RecentSettlement> = recent
        .iter()
        .map(|row| {
            let id: uuid::Uuid = row.get("id");
            let status: String = row.get("status");
            let energy_amount: f64 = row.get("energy");
            let total_amount: f64 = row.get("total");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            
            RecentSettlement {
                id: id.to_string(),
                status,
                energy_amount,
                total_amount,
                created_at: created_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(Json(SettlementStatusResponse {
        pending_count,
        processing_count,
        completed_count,
        failed_count,
        total_settled_value,
        recent_settlements,
    }))
}
