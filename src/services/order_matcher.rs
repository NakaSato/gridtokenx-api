use anyhow::Result;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::str::FromStr;
use tracing::{debug, info, warn, instrument, error};
use uuid::Uuid;

use crate::database::schema::types::{OrderSide, OrderStatus};
use crate::services::market_clearing_service::{MarketClearingService, OrderBookEntry, OrderMatch};

#[derive(Debug, Clone)]
pub struct BuyOrder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub energy_amount: Decimal,
    pub price_per_kwh: Decimal,
    pub created_at: DateTime<Utc>,
}

impl PartialEq for BuyOrder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BuyOrder {}

impl PartialOrd for BuyOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BuyOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        // Buy orders: Higher price first (reverse for max-heap)
        match other.price_per_kwh.cmp(&self.price_per_kwh) {
            Ordering::Equal => {
                // Same price: earlier time first
                self.created_at.cmp(&other.created_at)
            }
            other => other,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SellOrder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub energy_amount: Decimal,
    pub price_per_kwh: Decimal,
    pub created_at: DateTime<Utc>,
}

impl PartialEq for SellOrder {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SellOrder {}

impl PartialOrd for SellOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SellOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sell orders: Lower price first
        match self.price_per_kwh.cmp(&other.price_per_kwh) {
            Ordering::Equal => {
                // Same price: earlier time first
                self.created_at.cmp(&other.created_at)
            }
            other => other,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchingResult {
    pub matches: Vec<OrderMatch>,
    pub total_volume: Decimal,
    pub total_value: Decimal,
    pub clearing_price: Decimal,
    pub matched_orders: usize,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct OrderMatcherConfig {
    pub max_orders_per_epoch: usize,
    pub price_precision: u8,
    pub volume_precision: u8,
    pub enable_pro_rata_matching: bool,
}

impl Default for OrderMatcherConfig {
    fn default() -> Self {
        Self {
            max_orders_per_epoch: 10_000,
            price_precision: 4,
            volume_precision: 4,
            enable_pro_rata_matching: true,
        }
    }
}

#[derive(Debug)]
pub struct OrderMatcher {
    db: PgPool,
    config: OrderMatcherConfig,
}

impl OrderMatcher {
    pub fn new(db: PgPool, config: OrderMatcherConfig) -> Self {
        Self { db, config }
    }

    /// Match orders for a specific epoch using price-time priority
    #[instrument(skip(self))]
    pub async fn match_orders(&self, epoch_id: Uuid) -> Result<MatchingResult> {
        let start_time = std::time::Instant::now();
        info!("Starting order matching for epoch: {}", epoch_id);

        // Get order book for this epoch
        let (buy_orders_db, sell_orders_db) = self.get_order_book_for_epoch(epoch_id).await?;
        
        if buy_orders_db.is_empty() || sell_orders_db.is_empty() {
            info!("No orders to match in epoch {}: buy={}, sell={}", 
                   epoch_id, buy_orders_db.len(), sell_orders_db.len());
            
            return Ok(MatchingResult {
                matches: vec![],
                total_volume: Decimal::ZERO,
                total_value: Decimal::ZERO,
                clearing_price: Decimal::ZERO,
                matched_orders: 0,
                processing_time_ms: start_time.elapsed().as_millis() as u64,
            });
        }

        // Convert to heap structures for efficient matching
        let mut buy_orders = BinaryHeap::new();
        let mut sell_orders = BinaryHeap::new();

        for order in buy_orders_db {
            buy_orders.push(BuyOrder {
                id: order.order_id,
                user_id: order.user_id,
                energy_amount: order.energy_amount,
                price_per_kwh: order.price_per_kwh,
                created_at: order.created_at,
            });
        }

        for order in sell_orders_db {
            sell_orders.push(SellOrder {
                id: order.order_id,
                user_id: order.user_id,
                energy_amount: order.energy_amount,
                price_per_kwh: order.price_per_kwh,
                created_at: order.created_at,
            });
        }

        let mut matches = Vec::new();
        let mut total_volume = Decimal::ZERO;
        let mut total_value = Decimal::ZERO;
        let mut match_count = 0;

        // Main matching loop
        while let (Some(buy_order), Some(sell_order)) = (buy_orders.peek(), sell_orders.peek()) {
            // Check if orders can be matched
            if buy_order.price_per_kwh >= sell_order.price_per_kwh {
                let buy_order = buy_orders.pop().unwrap();
                let sell_order = sell_orders.pop().unwrap();

                // Determine match price (market clearing price - use sell order's price)
                let match_price = sell_order.price_per_kwh;
                
                // Calculate match amount
                let match_amount = buy_order.energy_amount.min(sell_order.energy_amount);

                if match_amount > Decimal::ZERO {
                    // Create order match
                    let order_match = OrderMatch {
                        id: Uuid::new_v4(),
                        epoch_id,
                        buy_order_id: buy_order.id,
                        sell_order_id: sell_order.id,
                        matched_amount: match_amount,
                        match_price,
                        match_time: Utc::now(),
                        status: "pending".to_string(),
                    };

                    matches.push(order_match.clone());
                    
                    // Update totals
                    total_volume += match_amount;
                    total_value += match_amount * match_price;
                    match_count += 1;

                    debug!(
                        "Matched orders: Buy {} vs Sell {} at {} for {} kWh",
                        buy_order.id, sell_order.id, match_price, match_amount
                    );

                    // Handle partial fills
                    let buy_remaining = buy_order.energy_amount - match_amount;
                    let sell_remaining = sell_order.energy_amount - match_amount;

                    // Update order amounts in database
                    self.update_order_filled_amount(buy_order.id, match_amount).await?;
                    self.update_order_filled_amount(sell_order.id, match_amount).await?;

                    // Push back remaining orders
                    if buy_remaining > Decimal::ZERO {
                        let mut updated_buy_order = buy_order;
                        updated_buy_order.energy_amount = buy_remaining;
                        buy_orders.push(updated_buy_order);
                    } else {
                        // Mark as fully filled
                        self.mark_order_filled(buy_order.id).await?;
                    }

                    if sell_remaining > Decimal::ZERO {
                        let mut updated_sell_order = sell_order;
                        updated_sell_order.energy_amount = sell_remaining;
                        sell_orders.push(updated_sell_order);
                    } else {
                        // Mark as fully filled
                        self.mark_order_filled(sell_order.id).await?;
                    }
                }
            } else {
                // No more matches possible (best buy price < best sell price)
                break;
            }
        }

        // Calculate market clearing price (weighted average)
        let clearing_price = if total_volume > Decimal::ZERO {
            total_value / total_volume
        } else {
            Decimal::ZERO
        };

        // Save all matches to database
        for order_match in &matches {
            self.save_order_match(order_match).await?;
        }

        // Update epoch statistics
        self.update_epoch_statistics(epoch_id, total_volume, match_count as i64, clearing_price).await?;

        let processing_time = start_time.elapsed().as_millis() as u64;
        
        info!(
            "Order matching completed for epoch: {} - {} matches, {} kWh, clearing price: {}, took {}ms",
            epoch_id, matches.len(), total_volume, clearing_price, processing_time
        );

        Ok(MatchingResult {
            matches,
            total_volume,
            total_value,
            clearing_price,
            matched_orders: match_count,
            processing_time_ms: processing_time,
        })
    }

    /// Handle pro-rata matching for multiple orders at same price
    #[instrument(skip(self))]
    async fn handle_pro_rata_matching(
        &self,
        orders_at_price: Vec<(Uuid, Decimal, DateTime<Utc>)>,
        available_volume: Decimal,
    ) -> Result<Vec<(Uuid, Decimal)>> {
        if orders_at_price.is_empty() {
            return Ok(vec![]);
        }

        let total_requested: Decimal = orders_at_price.iter()
            .map(|(_, amount, _)| *amount)
            .sum();

        if total_requested <= available_volume {
            // All orders can be filled completely
            Ok(orders_at_price.into_iter()
                .map(|(id, amount, _)| (id, amount))
                .collect())
        } else {
            // Pro-rata allocation
            let allocation_ratio = available_volume / total_requested;
            let mut allocations = Vec::new();
            let mut allocated_volume = Decimal::ZERO;

            for (id, amount, _) in orders_at_price {
                let allocated = (amount * allocation_ratio).round_dp(self.config.volume_precision as u32);
                allocations.push((id, allocated));
                allocated_volume += allocated;
            }

            // Handle rounding errors by adjusting the last order
            let rounding_error = available_volume - allocated_volume;
            if rounding_error > Decimal::ZERO && !allocations.is_empty() {
                let last_index = allocations.len() - 1;
                allocations[last_index].1 += rounding_error;
            }

            Ok(allocations)
        }
    }

    /// Get order book for a specific epoch
    async fn get_order_book_for_epoch(&self, epoch_id: Uuid) -> Result<(Vec<OrderBookEntry>, Vec<OrderBookEntry>)> {
        // Get epoch time range
        let epoch = sqlx::query!(
            "SELECT start_time, end_time FROM market_epochs WHERE id = $1",
            epoch_id
        )
        .fetch_optional(&self.db)
        .await?;

        let epoch = epoch.ok_or_else(|| anyhow::anyhow!("Epoch not found: {}", epoch_id))?;

        // Get pending buy orders (sorted by price descending, then time ascending)
        let buy_orders = sqlx::query_as!(
            OrderBookEntry,
            r#"
            SELECT 
                id as order_id, user_id, side as "side: OrderSide", 
                energy_amount, price_per_kwh, created_at
            FROM trading_orders 
            WHERE status = 'Pending'::order_status 
              AND side = 'Buy'::order_side
              AND created_at >= $1 
              AND created_at < $2
            ORDER BY price_per_kwh DESC, created_at ASC
            LIMIT $3
            "#,
            epoch.start_time,
            epoch.end_time,
            self.config.max_orders_per_epoch as i64
        )
        .fetch_all(&self.db)
        .await?;

        // Get pending sell orders (sorted by price ascending, then time ascending)
        let sell_orders = sqlx::query_as!(
            OrderBookEntry,
            r#"
            SELECT 
                id as order_id, user_id, side as "side: OrderSide", 
                energy_amount, price_per_kwh, created_at
            FROM trading_orders 
            WHERE status = 'Pending'::order_status 
              AND side = 'Sell'::order_side
              AND created_at >= $1 
              AND created_at < $2
            ORDER BY price_per_kwh ASC, created_at ASC
            LIMIT $3
            "#,
            epoch.start_time,
            epoch.end_time,
            self.config.max_orders_per_epoch as i64
        )
        .fetch_all(&self.db)
        .await?;

        Ok((buy_orders, sell_orders))
    }

    /// Update order filled amount
    async fn update_order_filled_amount(&self, order_id: Uuid, amount: Decimal) -> Result<()> {
        sqlx::query!(
            "UPDATE trading_orders SET filled_amount = filled_amount + $1 WHERE id = $2",
            amount,
            order_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Mark order as fully filled
    async fn mark_order_filled(&self, order_id: Uuid) -> Result<()> {
        sqlx::query!(
            "UPDATE trading_orders SET status = 'Filled'::order_status, filled_at = NOW() WHERE id = $1",
            order_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Save order match to database
    async fn save_order_match(&self, order_match: &OrderMatch) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO order_matches (
                id, epoch_id, buy_order_id, sell_order_id, 
                matched_amount, match_price, match_time, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            order_match.id,
            order_match.epoch_id,
            order_match.buy_order_id,
            order_match.sell_order_id,
            order_match.matched_amount,
            order_match.match_price,
            order_match.match_time,
            order_match.status
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Update epoch statistics
    async fn update_epoch_statistics(
        &self,
        epoch_id: Uuid,
        total_volume: Decimal,
        matched_orders: i64,
        clearing_price: Decimal,
    ) -> Result<()> {
        // Get total orders count for the epoch
        let total_orders = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) 
            FROM trading_orders 
            WHERE created_at >= (SELECT start_time FROM market_epochs WHERE id = $1)
              AND created_at < (SELECT end_time FROM market_epochs WHERE id = $1)
              AND status IN ('Pending'::order_status, 'Filled'::order_status)
            "#,
            epoch_id
        )
        .fetch_one(&self.db)
        .await?
        .unwrap_or(0);

        sqlx::query!(
            r#"
            UPDATE market_epochs 
            SET total_volume = $1, matched_orders = $2, total_orders = $3, 
                clearing_price = $4, status = 'cleared', updated_at = NOW()
            WHERE id = $5
            "#,
            total_volume,
            matched_orders,
            total_orders,
            clearing_price,
            epoch_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get matching performance metrics
    pub async fn get_performance_metrics(&self, epoch_id: Uuid) -> Result<MatchingMetrics> {
        let metrics = sqlx::query!(
            r#"
            SELECT 
                e.epoch_number,
                e.total_volume,
                e.matched_orders,
                e.total_orders,
                e.clearing_price,
                COUNT(m.id) as match_count,
                AVG(m.matched_amount) as avg_match_amount,
                MAX(m.matched_amount) as max_match_amount,
                MIN(m.matched_amount) as min_match_amount,
                EXTRACT(EPOCH FROM (MAX(m.match_time) - MIN(m.match_time))) * 1000 as matching_duration_ms
            FROM market_epochs e
            LEFT JOIN order_matches m ON e.id = m.epoch_id
            WHERE e.id = $1
            GROUP BY e.id, e.epoch_number, e.total_volume, e.matched_orders, e.total_orders, e.clearing_price
            "#,
            epoch_id
        )
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = metrics {
            Ok(MatchingMetrics {
                epoch_number: row.epoch_number,
                total_volume: row.total_volume,
                matched_orders: row.matched_orders,
                total_orders: row.total_orders,
                clearing_price: row.clearing_price,
                match_count: row.match_count.unwrap_or(0) as i64,
                avg_match_amount: row.avg_match_amount,
                max_match_amount: row.max_match_amount,
                min_match_amount: row.min_match_amount,
                matching_duration_ms: row.matching_duration_ms.map(|v| v as u64),
            })
        } else {
            Err(anyhow::anyhow!("Epoch not found: {}", epoch_id))
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchingMetrics {
    pub epoch_number: i64,
    pub total_volume: Decimal,
    pub matched_orders: i64,
    pub total_orders: i64,
    pub clearing_price: Option<Decimal>,
    pub match_count: i64,
    pub avg_match_amount: Option<Decimal>,
    pub max_match_amount: Option<Decimal>,
    pub min_match_amount: Option<Decimal>,
    pub matching_duration_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    fn create_test_buy_order(id: Uuid, amount: f64, price: f64) -> BuyOrder {
        BuyOrder {
            id,
            user_id: Uuid::new_v4(),
            energy_amount: Decimal::from_str(&amount.to_string()).unwrap(),
            price_per_kwh: Decimal::from_str(&price.to_string()).unwrap(),
            created_at: Utc::now(),
        }
    }

    fn create_test_sell_order(id: Uuid, amount: f64, price: f64) -> SellOrder {
        SellOrder {
            id,
            user_id: Uuid::new_v4(),
            energy_amount: Decimal::from_str(&amount.to_string()).unwrap(),
            price_per_kwh: Decimal::from_str(&price.to_string()).unwrap(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_buy_order_ordering() {
        let now = Utc::now();
        let earlier = now - Duration::minutes(1);
        
        let order1 = BuyOrder {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            energy_amount: Decimal::from(100),
            price_per_kwh: Decimal::from_str("0.15").unwrap(),
            created_at: earlier,
        };
        
        let order2 = BuyOrder {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            energy_amount: Decimal::from(100),
            price_per_kwh: Decimal::from_str("0.16").unwrap(),
            created_at: now,
        };
        
        let order3 = BuyOrder {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            energy_amount: Decimal::from(100),
            price_per_kwh: Decimal::from_str("0.15").unwrap(),
            created_at: now,
        };
        
        let mut heap = BinaryHeap::new();
        heap.push(order1.clone());
        heap.push(order2.clone());
        heap.push(order3.clone());
        
        // Should come out in price order: 0.16 first, then 0.15 (earlier), then 0.15 (later)
        assert_eq!(heap.pop().unwrap().id, order2.id); // Highest price
        assert_eq!(heap.pop().unwrap().id, order1.id); // Same price, earlier time
        assert_eq!(heap.pop().unwrap().id, order3.id); // Same price, later time
    }

    #[test]
    fn test_sell_order_ordering() {
        let now = Utc::now();
        let earlier = now - Duration::minutes(1);
        
        let order1 = SellOrder {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            energy_amount: Decimal::from(100),
            price_per_kwh: Decimal::from_str("0.15").unwrap(),
            created_at: earlier,
        };
        
        let order2 = SellOrder {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            energy_amount: Decimal::from(100),
            price_per_kwh: Decimal::from_str("0.14").unwrap(),
            created_at: now,
        };
        
        let order3 = SellOrder {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            energy_amount: Decimal::from(100),
            price_per_kwh: Decimal::from_str("0.15").unwrap(),
            created_at: now,
        };
        
        let mut heap = BinaryHeap::new();
        heap.push(order1.clone());
        heap.push(order2.clone());
        heap.push(order3.clone());
        
        // Should come out in price order: 0.14 first, then 0.15 (earlier), then 0.15 (later)
        assert_eq!(heap.pop().unwrap().id, order2.id); // Lowest price
        assert_eq!(heap.pop().unwrap().id, order1.id); // Same price, earlier time
        assert_eq!(heap.pop().unwrap().id, order3.id); // Same price, later time
    }

    #[tokio::test]
    async fn test_pro_rata_matching() {
        let config = OrderMatcherConfig::default();
        // Note: This test would need a mock database to be fully functional
        // For now, we'll test the logic structure
        assert!(config.enable_pro_rata_matching);
    }
}
