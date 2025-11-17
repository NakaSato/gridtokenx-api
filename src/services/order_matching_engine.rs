use anyhow::Result;
use sqlx::types::BigDecimal;
use sqlx::PgPool;
use sqlx::Row;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::WebSocketService;

/// Background service that automatically matches orders with offers
#[derive(Clone)]
pub struct OrderMatchingEngine {
    db: PgPool,
    running: Arc<RwLock<bool>>,
    match_interval_secs: u64,
    websocket_service: Option<WebSocketService>,
}

impl OrderMatchingEngine {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            running: Arc::new(RwLock::new(false)),
            match_interval_secs: 5, // Check every 5 seconds
            websocket_service: None,
        }
    }

    /// Set the WebSocket service for broadcasting match events
    pub fn with_websocket(mut self, ws_service: WebSocketService) -> Self {
        self.websocket_service = Some(ws_service);
        self
    }

    /// Start the background matching engine
    pub async fn start(&self) {
        let mut running = self.running.write().await;
        if *running {
            warn!("Order matching engine is already running");
            return;
        }
        *running = true;
        drop(running);

        info!("🚀 Starting automated order matching engine (interval: {}s)", self.match_interval_secs);

        let engine = self.clone();
        tokio::spawn(async move {
            engine.run_matching_loop().await;
        });
    }

    /// Stop the background matching engine
    pub async fn stop(&self) {
        let mut running = self.running.write().await;
        *running = false;
        info!("⏹️  Stopped automated order matching engine");
    }

    /// Main matching loop
    async fn run_matching_loop(&self) {
        loop {
            // Check if we should continue running
            {
                let running = self.running.read().await;
                if !*running {
                    break;
                }
            }

            // Run one matching cycle
            match self.match_orders_cycle().await {
                Ok(matches) => {
                    if matches > 0 {
                        info!("✅ Matching cycle completed: {} new transactions created", matches);
                    } else {
                        debug!("Matching cycle completed: no new matches");
                    }
                }
                Err(e) => {
                    error!("❌ Error in matching cycle: {}", e);
                }
            }

            // Sleep before next cycle
            sleep(Duration::from_secs(self.match_interval_secs)).await;
        }

        info!("Order matching loop terminated");
    }

    /// Run one matching cycle
    async fn match_orders_cycle(&self) -> Result<usize> {
        debug!("Running order matching cycle...");

        // Get all pending buy orders
        let pending_orders = sqlx::query(
            r#"
            SELECT 
                id, 
                created_by, 
                energy_amount, 
                max_price_per_kwh,
                preferred_source,
                required_from,
                required_until
            FROM orders
            WHERE status = 'pending'
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(&self.db)
        .await?;

        if pending_orders.is_empty() {
            return Ok(0);
        }

        debug!("Found {} pending orders to process", pending_orders.len());

        let mut matches_created = 0;

        // Try to match each order
        for order in pending_orders {
            let order_id: Uuid = order.try_get("id")?;
            let created_by: Uuid = order.try_get("created_by")?;
            let energy_amount: BigDecimal = order.try_get("energy_amount")?;
            let max_price_per_kwh: BigDecimal = order.try_get("max_price_per_kwh")?;
            let preferred_source: Option<String> = order.try_get("preferred_source")?;

            // Find compatible active offers
            let compatible_offers = if let Some(ref source) = preferred_source {
                sqlx::query(
                    r#"
                    SELECT 
                        id,
                        created_by,
                        energy_amount,
                        price_per_kwh,
                        energy_source,
                        available_from,
                        available_until
                    FROM offers
                    WHERE status = 'active'
                        AND energy_source = $1
                        AND price_per_kwh <= $2
                        AND energy_amount > 0
                    ORDER BY price_per_kwh ASC, created_at ASC
                    LIMIT 10
                    "#,
                )
                .bind(source)
                .bind(&max_price_per_kwh)
                .fetch_all(&self.db)
                .await?
            } else {
                sqlx::query(
                    r#"
                    SELECT 
                        id,
                        created_by,
                        energy_amount,
                        price_per_kwh,
                        energy_source,
                        available_from,
                        available_until
                    FROM offers
                    WHERE status = 'active'
                        AND price_per_kwh <= $1
                        AND energy_amount > 0
                    ORDER BY price_per_kwh ASC, created_at ASC
                    LIMIT 10
                    "#,
                )
                .bind(&max_price_per_kwh)
                .fetch_all(&self.db)
                .await?
            };

            if compatible_offers.is_empty() {
                debug!("No compatible offers found for order {}", order_id);
                continue;
            }

            debug!(
                "Found {} compatible offers for order {} (needs {} kWh at max ${}/kWh)",
                compatible_offers.len(),
                order_id,
                energy_amount,
                max_price_per_kwh
            );

            // Match with the best offer
            let mut remaining_order_amount = energy_amount.clone();
            
            for offer in compatible_offers {
                // Check if order is fulfilled
                let zero = BigDecimal::from_str("0").unwrap();
                if remaining_order_amount <= zero {
                    break;
                }

                let offer_id: Uuid = offer.try_get("id")?;
                let offer_created_by: Uuid = offer.try_get("created_by")?;
                let offer_energy_amount: BigDecimal = offer.try_get("energy_amount")?;
                let offer_price_per_kwh: BigDecimal = offer.try_get("price_per_kwh")?;

                // Calculate match amount (min of order and offer)
                let match_amount = if remaining_order_amount < offer_energy_amount {
                    remaining_order_amount.clone()
                } else {
                    offer_energy_amount.clone()
                };
                let match_price = offer_price_per_kwh.clone();
                let total_price = &match_amount * &match_price;

                debug!(
                    "Matching order {} with offer {}: {} kWh at ${}/kWh (total: ${})",
                    order_id, offer_id, match_amount, match_price, total_price
                );

                // Create transaction
                match self
                    .create_transaction(
                        &offer_id,
                        &order_id,
                        &offer_created_by,
                        &created_by,
                        match_amount.clone(),
                        match_price.clone(),
                        total_price.clone(),
                    )
                    .await
                {
                    Ok(transaction_id) => {
                        info!(
                            "✅ Created transaction {}: {} kWh from offer {} to order {} at ${}/kWh",
                            transaction_id, match_amount, offer_id, order_id, match_price
                        );
                        matches_created += 1;

                        // Update offer energy amount
                        let new_offer_amount = &offer_energy_amount - &match_amount;
                        let zero = BigDecimal::from_str("0").unwrap();
                        if new_offer_amount <= zero {
                            // Mark offer as completed (don't update energy_amount to avoid constraint violation)
                            sqlx::query(
                                "UPDATE offers SET status = 'completed', updated_at = NOW() WHERE id = $1"
                            )
                            .bind(offer_id)
                            .execute(&self.db)
                            .await?;
                            debug!("Offer {} fully consumed and marked as completed", offer_id);
                        } else {
                            // Update remaining amount
                            sqlx::query(
                                "UPDATE offers SET energy_amount = $1, updated_at = NOW() WHERE id = $2"
                            )
                            .bind(&new_offer_amount)
                            .bind(offer_id)
                            .execute(&self.db)
                            .await?;
                            debug!("Offer {} updated: {} kWh remaining", offer_id, new_offer_amount);
                        }

                        // Update remaining order amount
                        remaining_order_amount = &remaining_order_amount - &match_amount;

                        // Update order status
                        if remaining_order_amount <= zero {
                            // Order fully filled
                            sqlx::query(
                                "UPDATE orders SET status = 'completed', updated_at = NOW() WHERE id = $1"
                            )
                            .bind(order_id)
                            .execute(&self.db)
                            .await?;
                            debug!("Order {} fully filled and marked as completed", order_id);
                            break; // Move to next order
                        } else {
                            // Order partially filled
                            sqlx::query(
                                "UPDATE orders SET status = 'partial', energy_amount = $1, updated_at = NOW() WHERE id = $2"
                            )
                            .bind(&remaining_order_amount)
                            .bind(order_id)
                            .execute(&self.db)
                            .await?;
                            debug!(
                                "Order {} partially filled: {} kWh remaining",
                                order_id,
                                remaining_order_amount
                            );
                            // Continue to next offer for this order
                        }
                    }
                    Err(e) => {
                        error!("Failed to create transaction: {}", e);
                        continue;
                    }
                }
            }
        }

        Ok(matches_created)
    }

    /// Create a transaction record
    async fn create_transaction(
        &self,
        offer_id: &Uuid,
        order_id: &Uuid,
        seller_id: &Uuid,
        buyer_id: &Uuid,
        energy_amount: BigDecimal,
        price_per_kwh: BigDecimal,
        total_price: BigDecimal,
    ) -> Result<Uuid> {
        let transaction_id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO transactions (
                id,
                offer_id,
                order_id,
                seller_id,
                buyer_id,
                energy_amount,
                price_per_kwh,
                total_price,
                status,
                created_at,
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending', NOW(), NOW())
            "#,
        )
        .bind(transaction_id)
        .bind(offer_id)
        .bind(order_id)
        .bind(seller_id)
        .bind(buyer_id)
        .bind(&energy_amount)
        .bind(&price_per_kwh)
        .bind(&total_price)
        .execute(&self.db)
        .await?;

        // Broadcast order matched event via WebSocket
        if let Some(ws_service) = &self.websocket_service {
            let energy_f64 = energy_amount.to_string().parse::<f64>().unwrap_or(0.0);
            let price_f64 = price_per_kwh.to_string().parse::<f64>().unwrap_or(0.0);
            
            tokio::spawn({
                let ws = ws_service.clone();
                let tid = transaction_id.to_string();
                let oid = order_id.to_string();
                let ofid = offer_id.to_string();
                async move {
                    ws.broadcast_order_matched(oid, ofid, tid, energy_f64, price_f64).await;
                }
            });
        }

        Ok(transaction_id)
    }

    /// Manually trigger a matching cycle (for testing or API endpoints)
    pub async fn trigger_matching(&self) -> Result<usize> {
        info!("Manual matching trigger requested");
        self.match_orders_cycle().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_engine_creation() {
        // This is a placeholder test since we need a real database for full testing
        // In production, you would use a test database
        let pool = PgPool::connect_lazy("postgresql://localhost/test").unwrap();
        let engine = OrderMatchingEngine::new(pool);
        assert_eq!(engine.match_interval_secs, 5);
    }
}
