// Database schema definitions will be added here
// This module will contain SQL schema definitions and migrations

pub mod types {
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
    #[sqlx(type_name = "user_role", rename_all = "lowercase")]
    pub enum UserRole {
        User,
        Admin,
        Ami,
        Producer,
        Consumer,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, ToSchema)]
    #[sqlx(type_name = "order_type")]
    pub enum OrderType {
        Market,
        Limit,
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, ToSchema)]
    #[sqlx(type_name = "order_side")]
    pub enum OrderSide {
        Buy,
        Sell,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::Type, ToSchema)]
    #[sqlx(type_name = "order_status")]
    pub enum OrderStatus {
        Pending,
        Active,
        PartiallyFilled,
        Filled,
        Settled,
        Cancelled,
        Expired,
    }
}