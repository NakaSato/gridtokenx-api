//! Price Monitor Service
//!
//! Background service that monitors market prices and triggers conditional orders
//! (stop-loss, take-profit, trailing stop) when conditions are met.

use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};
use chrono::Utc;
use uuid::Uuid;

use crate::models::trading::{TriggerType, TriggerStatus};
use crate::database::schema::types::{OrderSide, OrderType, OrderStatus};

/// Price monitor configuration
#[derive(Debug, Clone)]
pub struct PriceMonitorConfig {
    /// How often to check prices (in seconds)
    pub check_interval_secs: u64,
    /// Whether the monitor is enabled
    pub enabled: bool,
}

impl Default for PriceMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 10,
            enabled: true,
        }
    }
}

/// Price monitor service
#[derive(Clone)]
pub struct PriceMonitor {
    db: PgPool,
    config: PriceMonitorConfig,
}

impl PriceMonitor {
    pub fn new(db: PgPool, config: PriceMonitorConfig) -> Self {
        Self { db, config }
    }

    /// Start the price monitoring loop
    pub async fn start(self: Arc<Self>) {
        if !self.config.enabled {
            info!("Price monitor is disabled");
            return;
        }

        info!("Starting price monitor with {}s interval", self.config.check_interval_secs);
        
        let mut check_interval = interval(Duration::from_secs(self.config.check_interval_secs));

        loop {
            check_interval.tick().await;
            
            if let Err(e) = self.check_and_trigger_orders().await {
                error!("Price monitor error: {}", e);
            }
        }
    }

    /// Check pending conditional orders and trigger if conditions are met
    async fn check_and_trigger_orders(&self) -> anyhow::Result<()> {
        // Get current market price (average of recent trades)
        let current_price = self.get_current_market_price().await?;
        
        if current_price <= Decimal::ZERO {
            // No recent trades to determine price
            return Ok(());
        }

        // Get pending conditional orders
        let pending_orders = sqlx::query!(
            r#"
            SELECT id, user_id, side as "side: OrderSide", energy_amount, trigger_price, 
                   trigger_type as "trigger_type: TriggerType", price_per_kwh as limit_price,
                   trailing_offset
            FROM trading_orders
            WHERE trigger_type IS NOT NULL 
              AND trigger_status = 'pending'
              AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY created_at ASC
            LIMIT 100
            "#
        )
        .fetch_all(&self.db)
        .await?;

        if pending_orders.is_empty() {
            return Ok(());
        }

        info!("Checking {} pending conditional orders against price {}", pending_orders.len(), current_price);

        for order in pending_orders {
            // Skip orders with missing required fields
            let Some(trigger_type) = order.trigger_type else { continue };
            let Some(side) = order.side else { continue };
            
            let should_trigger = self.check_trigger_condition(
                &trigger_type,
                &side,
                order.trigger_price.unwrap_or(Decimal::ZERO),
                current_price,
                order.trailing_offset,
            );

            if should_trigger {
                info!("Triggering conditional order {} at price {}", order.id, current_price);
                
                if let Err(e) = self.trigger_order(
                    order.id,
                    order.user_id,
                    side,
                    order.energy_amount,
                    Some(order.limit_price),
                ).await {
                    error!("Failed to trigger order {}: {}", order.id, e);
                }
            }
        }

        Ok(())
    }

    /// Get current market price from recent trades
    async fn get_current_market_price(&self) -> anyhow::Result<Decimal> {
        let result = sqlx::query!(
            r#"
            SELECT AVG(price_per_kwh) as "avg_price!"
            FROM trading_orders
            WHERE status = 'filled' 
              AND filled_at > NOW() - INTERVAL '1 hour'
            "#
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(result.map(|r| r.avg_price).unwrap_or(Decimal::ZERO))
    }

    /// Check if a trigger condition is met
    fn check_trigger_condition(
        &self,
        trigger_type: &TriggerType,
        side: &OrderSide,
        trigger_price: Decimal,
        current_price: Decimal,
        _trailing_offset: Option<Decimal>,
    ) -> bool {
        match (trigger_type, side) {
            // Stop-loss for sell: trigger when price falls below trigger_price
            (TriggerType::StopLoss, OrderSide::Sell) => current_price <= trigger_price,
            // Stop-loss for buy: trigger when price rises above trigger_price
            (TriggerType::StopLoss, OrderSide::Buy) => current_price >= trigger_price,
            
            // Take-profit for sell: trigger when price rises above trigger_price
            (TriggerType::TakeProfit, OrderSide::Sell) => current_price >= trigger_price,
            // Take-profit for buy: trigger when price falls below trigger_price
            (TriggerType::TakeProfit, OrderSide::Buy) => current_price <= trigger_price,
            
            // Trailing stop: more complex logic (simplified for now)
            (TriggerType::TrailingStop, _) => {
                // TODO: Implement trailing stop with peak price tracking
                false
            }
        }
    }

    /// Trigger a conditional order by creating an actual trading order
    async fn trigger_order(
        &self,
        order_id: Uuid,
        user_id: Uuid,
        side: OrderSide,
        energy_amount: Decimal,
        limit_price: Option<Decimal>,
    ) -> anyhow::Result<()> {
        let now = Utc::now();
        
        // Begin transaction
        let mut tx = self.db.begin().await?;

        // Update the conditional order status to triggered
        sqlx::query!(
            r#"
            UPDATE trading_orders
            SET trigger_status = 'triggered', triggered_at = $1
            WHERE id = $2
            "#,
            now,
            order_id
        )
        .execute(&mut *tx)
        .await?;

        // Create the actual trading order
        let new_order_id = Uuid::new_v4();
        let order_type = if limit_price.is_some() && limit_price.unwrap_or(Decimal::ZERO) > Decimal::ZERO {
            OrderType::Limit
        } else {
            OrderType::Market
        };

        sqlx::query!(
            r#"
            INSERT INTO trading_orders (
                id, user_id, order_type, side, energy_amount, price_per_kwh,
                filled_amount, status, created_at, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            new_order_id,
            user_id,
            order_type as OrderType,
            side as OrderSide,
            energy_amount,
            limit_price.unwrap_or(Decimal::ZERO),
            Decimal::ZERO,
            OrderStatus::Pending as OrderStatus,
            now,
            now + chrono::Duration::hours(24)
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!(
            "Triggered conditional order {} -> created trading order {}",
            order_id, new_order_id
        );

        // TODO: Send WebSocket notification to user

        Ok(())
    }
}
