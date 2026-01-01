use axum::{
    extract::{Query, State},
    response::Json,
};
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::error::Result;
use crate::AppState;

use super::types::*;

/// Get user trading statistics
#[utoipa::path(
    get,
    path = "/api/v1/analytics/my-stats",
    params(AnalyticsTimeframe),
    responses(
        (status = 200, description = "User trading statistics retrieved", body = UserTradingStats),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_user_trading_stats(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Query(params): Query<AnalyticsTimeframe>,
) -> Result<Json<UserTradingStats>> {
    let duration = parse_timeframe(&params.timeframe)?;
    let start_time = Utc::now() - duration;

    // Get seller stats
    let as_seller = get_seller_stats(&state, user.0.sub, start_time).await?;

    // Get buyer stats
    let as_buyer = get_buyer_stats(&state, user.0.sub, start_time).await?;

    // Get overall stats
    let overall = get_overall_user_stats(&state, user.0.sub, start_time).await?;

    Ok(Json(UserTradingStats {
        user_id: user.0.sub.to_string(),
        username: user.0.username.clone(),
        timeframe: params.timeframe,
        as_seller,
        as_buyer,
        overall,
    }))
}

/// Get user wealth history for charting
#[utoipa::path(
    get,
    path = "/api/v1/analytics/my-history",
    params(AnalyticsTimeframe),
    responses(
        (status = 200, description = "User wealth history retrieved", body = UserWealthHistory),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_user_wealth_history(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Query(params): Query<AnalyticsTimeframe>,
) -> Result<Json<UserWealthHistory>> {
    let duration = parse_timeframe(&params.timeframe)?;
    let start_time = Utc::now() - duration;

    // Use a window function to calculate cumulative balance over time
    let rows = sqlx::query(
        r#"
        WITH binned_trades AS (
            SELECT 
                date_trunc('hour', om.match_time) as bucket,
                SUM(CASE WHEN sell_o.user_id = $1 THEN om.matched_amount * om.match_price ELSE -(om.matched_amount * om.match_price) END) as delta
            FROM order_matches om
            JOIN trading_orders sell_o ON om.sell_order_id = sell_o.id
            JOIN trading_orders buy_o ON om.buy_order_id = buy_o.id
            WHERE (sell_o.user_id = $1 OR buy_o.user_id = $1)
            AND om.match_time >= $2
            GROUP BY 1
        )
        SELECT 
            bucket as timestamp,
            SUM(delta) OVER (ORDER BY bucket) as balance
        FROM binned_trades
        ORDER BY bucket ASC
        "#,
    )
    .bind(user.0.sub)
    .bind(start_time)
    .fetch_all(&state.db)
    .await?;

    let history = rows
        .into_iter()
        .map(|row| WealthPoint {
            timestamp: row.get("timestamp"),
            balance_usd: decimal_to_f64(row.get("balance")),
        })
        .collect();

    Ok(Json(UserWealthHistory {
        timeframe: params.timeframe,
        history,
    }))
}

/// Get user transaction history
#[utoipa::path(
    get,
    path = "/api/v1/analytics/transactions",
    params(TransactionQuery),
    responses(
        (status = 200, description = "User transaction history retrieved", body = UserTransactionsResponse),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_user_transactions(
    user: AuthenticatedUser,
    State(state): State<AppState>,
    Query(params): Query<TransactionQuery>,
) -> Result<Json<UserTransactionsResponse>> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);

    let mut where_conditions = vec!["user_id = $1".to_string()];
    let mut bind_count = 2;

    if let Some(_type) = &params.transaction_type {
        where_conditions.push(format!("operation_type = ${}", bind_count));
        bind_count += 1;
    }

    if let Some(_status) = &params.status {
        where_conditions.push(format!("operation_status = ${}", bind_count));
        bind_count += 1;
    }

    let where_clause = where_conditions.join(" AND ");

    // Count total
    let count_query = format!("SELECT COUNT(*) FROM blockchain_operations WHERE {}", where_clause);
    let mut count_sqlx = sqlx::query_scalar::<_, i64>(&count_query).bind(user.0.sub);
    
    if let Some(t) = &params.transaction_type {
        count_sqlx = count_sqlx.bind(t);
    }
    if let Some(s) = &params.status {
        count_sqlx = count_sqlx.bind(s);
    }

    let total = count_sqlx.fetch_one(&state.db).await.unwrap_or(0);

    // Fetch data - map to the view columns
    // The UserTransaction struct in types.rs uses standard names
    let query = format!(
        "SELECT 
            operation_type, operation_id, user_id, signature, tx_type, 
            operation_status as status, attempts, last_error, 
            submitted_at, confirmed_at, created_at, updated_at,
            CASE 
                WHEN operation_type = 'settlement' THEN (
                    SELECT json_build_object(
                        'energy_amount', energy_amount,
                        'price_per_kwh', price_per_kwh,
                        'total_amount', total_amount,
                        'wheeling_charge', wheeling_charge,
                        'loss_cost', loss_cost,
                        'loss_factor', loss_factor,
                        'effective_energy', effective_energy,
                        'buyer_zone_id', buyer_zone_id,
                        'seller_zone_id', seller_zone_id
                    ) FROM settlements WHERE id = operation_id
                )
                WHEN operation_type = 'trading_order' THEN (
                    SELECT json_build_object(
                        'side', side,
                        'energy_amount', energy_amount,
                        'price_per_kwh', price_per_kwh,
                        'zone_id', zone_id
                    ) FROM trading_orders WHERE id = operation_id
                )
                ELSE NULL
            END as metadata
         FROM blockchain_operations 
         WHERE {} 
         ORDER BY created_at DESC 
         LIMIT ${} OFFSET ${}",
        where_clause, bind_count, bind_count + 1
    );

    let mut sqlx_query = sqlx::query_as::<_, UserTransaction>(&query).bind(user.0.sub);

    if let Some(t) = &params.transaction_type {
        sqlx_query = sqlx_query.bind(t);
    }
    if let Some(s) = &params.status {
        sqlx_query = sqlx_query.bind(s);
    }

    sqlx_query = sqlx_query.bind(limit);
    sqlx_query = sqlx_query.bind(offset);

    let transactions = sqlx_query.fetch_all(&state.db).await.unwrap_or_default();

    Ok(Json(UserTransactionsResponse {
        transactions,
        total,
    }))
}

// ==================== HELPER FUNCTIONS ====================

async fn get_seller_stats(
    state: &AppState,
    user_id: Uuid,
    start_time: DateTime<Utc>,
) -> Result<SellerStats> {
    let row = sqlx::query(
        r#"
        SELECT 
            (SELECT COUNT(*) FROM trading_orders WHERE user_id = $1 AND side = 'sell' AND created_at >= $2) as offers_created,
            (SELECT COUNT(*) FROM trading_orders WHERE user_id = $1 AND side = 'sell' AND status = 'filled' AND created_at >= $2) as offers_fulfilled,
            COALESCE(SUM(om.matched_amount), 0) as total_sold,
            COALESCE(SUM(om.matched_amount * om.match_price), 0) as total_revenue,
            COALESCE(AVG(om.match_price), 0) as avg_price
        FROM trading_orders o
        LEFT JOIN order_matches om ON om.sell_order_id = o.id AND om.match_time >= $2
        WHERE o.user_id = $1 AND o.side = 'sell' AND o.created_at >= $2
        "#,
    )
    .bind(user_id)
    .bind(start_time)
    .fetch_one(&state.db)
    .await?;

    Ok(SellerStats {
        offers_created: row.try_get("offers_created").unwrap_or(0),
        offers_fulfilled: row.try_get("offers_fulfilled").unwrap_or(0),
        total_energy_sold_kwh: decimal_to_f64(row.get("total_sold")),
        total_revenue_usd: decimal_to_f64(row.get("total_revenue")),
        average_price_per_kwh: decimal_to_f64(row.get("avg_price")),
    })
}

async fn get_buyer_stats(
    state: &AppState,
    user_id: Uuid,
    start_time: DateTime<Utc>,
) -> Result<BuyerStats> {
    let row = sqlx::query(
        r#"
        SELECT 
            (SELECT COUNT(*) FROM trading_orders WHERE user_id = $1 AND side = 'buy' AND created_at >= $2) as orders_created,
            (SELECT COUNT(*) FROM trading_orders WHERE user_id = $1 AND side = 'buy' AND status = 'filled' AND created_at >= $2) as orders_fulfilled,
            COALESCE(SUM(om.matched_amount), 0) as total_purchased,
            COALESCE(SUM(om.matched_amount * om.match_price), 0) as total_spent,
            COALESCE(AVG(om.match_price), 0) as avg_price
        FROM trading_orders o
        LEFT JOIN order_matches om ON om.buy_order_id = o.id AND om.match_time >= $2
        WHERE o.user_id = $1 AND o.side = 'buy' AND o.created_at >= $2
        "#,
    )
    .bind(user_id)
    .bind(start_time)
    .fetch_one(&state.db)
    .await?;

    Ok(BuyerStats {
        orders_created: row.try_get("orders_created").unwrap_or(0),
        orders_fulfilled: row.try_get("orders_fulfilled").unwrap_or(0),
        total_energy_purchased_kwh: decimal_to_f64(row.get("total_purchased")),
        total_spent_usd: decimal_to_f64(row.get("total_spent")),
        average_price_per_kwh: decimal_to_f64(row.get("avg_price")),
    })
}

async fn get_overall_user_stats(
    state: &AppState,
    user_id: Uuid,
    start_time: DateTime<Utc>,
) -> Result<OverallUserStats> {
    // We combine buy and sell sides
    let row = sqlx::query(
        r#"
        WITH my_matches AS (
            SELECT 
                om.matched_amount,
                om.match_price,
                CASE WHEN sell_o.user_id = $1 THEN 'sell' ELSE 'buy' END as type
            FROM order_matches om
            JOIN trading_orders sell_o ON om.sell_order_id = sell_o.id
            JOIN trading_orders buy_o ON om.buy_order_id = buy_o.id
            WHERE (sell_o.user_id = $1 OR buy_o.user_id = $1)
            AND om.match_time >= $2
        )
        SELECT 
            COUNT(*) as total_transactions,
            COALESCE(SUM(matched_amount), 0) as total_volume,
            COALESCE(SUM(CASE WHEN type = 'sell' THEN matched_amount * match_price ELSE -(matched_amount * match_price) END), 0) as net_revenue
        FROM my_matches
        "#,
    )
    .bind(user_id)
    .bind(start_time)
    .fetch_one(&state.db)
    .await?;

    Ok(OverallUserStats {
        total_transactions: row.try_get("total_transactions").unwrap_or(0),
        total_volume_kwh: decimal_to_f64(row.get("total_volume")),
        net_revenue_usd: decimal_to_f64(row.get("net_revenue")),
        favorite_energy_source: None, // No source data
    })
}
