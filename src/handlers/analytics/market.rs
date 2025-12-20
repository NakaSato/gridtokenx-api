use axum::{
    extract::{Query, State},
    response::Json,
};
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::error::Result;
use crate::AppState;

use super::types::*;

/// Get market analytics
#[utoipa::path(
    get,
    path = "/api/v1/analytics/market",
    params(AnalyticsTimeframe),
    responses(
        (status = 200, description = "Market analytics retrieved", body = MarketAnalytics),
        (status = 400, description = "Invalid timeframe")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_market_analytics(
    State(state): State<AppState>,
    Query(params): Query<AnalyticsTimeframe>,
) -> Result<Json<MarketAnalytics>> {
    // Parse timeframe
    let duration = parse_timeframe(&params.timeframe)?;
    let start_time = Utc::now() - duration;
    let prev_start_time = start_time - duration; // For trend calculation

    // Get market overview
    let market_overview = get_market_overview(&state, start_time).await?;

    // Get trading volume
    let trading_volume = get_trading_volume(&state, start_time, prev_start_time).await?;

    // Get price statistics
    let price_statistics = get_price_statistics(&state, start_time, prev_start_time).await?;

    // Get energy source breakdown (Mocked/Simplified as column missing)
    let energy_source_breakdown = get_energy_source_breakdown(&state, start_time).await?;

    // Get top traders
    let top_traders = get_top_traders(&state, start_time, 10).await?;

    Ok(Json(MarketAnalytics {
        timeframe: params.timeframe,
        market_overview,
        trading_volume,
        price_statistics,
        energy_source_breakdown,
        top_traders,
    }))
}

// ==================== HELPER FUNCTIONS ====================

async fn get_market_overview(
    state: &AppState,
    start_time: DateTime<Utc>,
) -> Result<MarketOverview> {
    let row = sqlx::query(
        r#"
        SELECT 
            (SELECT COUNT(*) FROM trading_orders WHERE status = 'active' AND side = 'sell') as active_offers,
            (SELECT COUNT(*) FROM trading_orders WHERE status = 'pending') as pending_orders,
            (SELECT COUNT(*) FROM order_matches WHERE match_time >= $1) as completed_transactions,
            (SELECT COUNT(DISTINCT user_id) 
             FROM trading_orders 
             WHERE created_at >= $1) as users_trading,
            0.0 as avg_match_time
        "#,
    )
    .bind(start_time)
    .fetch_one(&state.db)
    .await?;

    // Note: avg_match_time is hard to calculate accurately without complex join, setting to 0 for now.

    Ok(MarketOverview {
        total_active_offers: row.try_get("active_offers").unwrap_or(0),
        total_pending_orders: row.try_get("pending_orders").unwrap_or(0),
        total_completed_transactions: row.try_get("completed_transactions").unwrap_or(0),
        total_users_trading: row.try_get("users_trading").unwrap_or(0),
        average_match_time_seconds: row.try_get("avg_match_time").unwrap_or(0.0), // f64 inferred?
    })
}

async fn get_trading_volume(
    state: &AppState,
    start_time: DateTime<Utc>,
    prev_start_time: DateTime<Utc>,
) -> Result<TradingVolume> {
    // Current period
    let current = sqlx::query(
        r#"
        SELECT 
            COALESCE(SUM(matched_amount), 0) as total_energy,
            COALESCE(SUM(matched_amount * match_price), 0) as total_value,
            COUNT(*) as transaction_count
        FROM order_matches
        WHERE match_time >= $1
        "#,
    )
    .bind(start_time)
    .fetch_one(&state.db)
    .await?;

    // Previous period for trend
    let previous = sqlx::query(
        r#"
        SELECT COALESCE(SUM(matched_amount), 0) as total_energy
        FROM order_matches
        WHERE match_time >= $1 AND match_time < $2
        "#,
    )
    .bind(prev_start_time)
    .bind(start_time)
    .fetch_one(&state.db)
    .await?;

    let current_energy = decimal_to_f64(current.get("total_energy"));
    let current_value = decimal_to_f64(current.get("total_value"));
    let transaction_count: i64 = current.get("transaction_count");
    let previous_energy = decimal_to_f64(previous.get("total_energy"));

    let volume_trend = if previous_energy > 0.0 {
        ((current_energy - previous_energy) / previous_energy) * 100.0
    } else {
        0.0
    };

    let avg_transaction_size = if transaction_count > 0 {
        current_energy / transaction_count as f64
    } else {
        0.0
    };

    Ok(TradingVolume {
        total_energy_traded_kwh: current_energy,
        total_value_usd: current_value,
        number_of_transactions: transaction_count,
        average_transaction_size_kwh: avg_transaction_size,
        volume_trend_percent: volume_trend,
    })
}

async fn get_price_statistics(
    state: &AppState,
    start_time: DateTime<Utc>,
    prev_start_time: DateTime<Utc>,
) -> Result<PriceStatistics> {
    // Current period stats
    let current = sqlx::query(
        r#"
        SELECT 
            COALESCE(AVG(match_price), 0) as avg_price,
            COALESCE(MIN(match_price), 0) as min_price,
            COALESCE(MAX(match_price), 0) as max_price,
            COALESCE(STDDEV(match_price), 0) as stddev_price,
            PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY match_price) as median_price
        FROM order_matches
        WHERE match_time >= $1
        "#,
    )
    .bind(start_time)
    .fetch_one(&state.db)
    .await?;

    // Previous period for trend
    let previous = sqlx::query(
        r#"
        SELECT COALESCE(AVG(match_price), 0) as avg_price
        FROM order_matches
        WHERE match_time >= $1 AND match_time < $2
        "#,
    )
    .bind(prev_start_time)
    .bind(start_time)
    .fetch_one(&state.db)
    .await?;

    let current_avg = decimal_to_f64(current.get("avg_price"));
    let min_price = decimal_to_f64(current.get("min_price"));
    let max_price = decimal_to_f64(current.get("max_price"));
    let stddev = decimal_to_f64(current.get("stddev_price"));
    let median = decimal_to_f64(current.try_get("median_price").unwrap_or(rust_decimal::Decimal::ZERO));
    let previous_avg = decimal_to_f64(previous.get("avg_price"));

    let price_trend = if previous_avg > 0.0 {
        ((current_avg - previous_avg) / previous_avg) * 100.0
    } else {
        0.0
    };

    let volatility = if current_avg > 0.0 {
        (stddev / current_avg) * 100.0
    } else {
        0.0
    };

    Ok(PriceStatistics {
        current_avg_price_per_kwh: current_avg,
        lowest_price_per_kwh: min_price,
        highest_price_per_kwh: max_price,
        median_price_per_kwh: median,
        price_volatility_percent: volatility,
        price_trend_percent: price_trend,
    })
}

async fn get_energy_source_breakdown(
    _state: &AppState,
    _start_time: DateTime<Utc>,
) -> Result<Vec<EnergySourceStats>> {
    // Mocked as column is missing
    Ok(vec![
        EnergySourceStats {
            energy_source: "Solar".to_string(),
            total_volume_kwh: 0.0,
            average_price_per_kwh: 0.0,
            transaction_count: 0,
            market_share_percent: 0.0,
        }
    ])
}

async fn get_top_traders(
    state: &AppState,
    start_time: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<TraderStats>> {
    let rows = sqlx::query(
        r#"
        WITH user_trades AS (
            SELECT 
                o.user_id,
                SUM(om.matched_amount) as total_volume,
                COUNT(*) as transaction_count,
                AVG(om.match_price) as avg_price
            FROM order_matches om
            JOIN trading_orders o ON om.sell_order_id = o.id
            WHERE om.match_time >= $1
            GROUP BY o.user_id
            ORDER BY total_volume DESC
            LIMIT $2
        )
        SELECT 
            ut.user_id,
            u.username,
            ut.total_volume,
            ut.transaction_count,
            ut.avg_price,
            u.role::text as role
        FROM user_trades ut
        JOIN users u ON ut.user_id = u.id
        "#,
    )
    .bind(start_time)
    .bind(limit)
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| TraderStats {
            user_id: row.get::<Uuid, _>("user_id").to_string(),
            username: row.get("username"),
            total_volume_kwh: decimal_to_f64(row.get("total_volume")),
            transaction_count: row.get("transaction_count"),
            average_price_per_kwh: decimal_to_f64(row.get("avg_price")),
            role: row.get("role"),
        })
        .collect())
}
