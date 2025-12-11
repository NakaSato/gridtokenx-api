pub mod admin;
pub mod analytics;
pub mod audit;
pub mod auth;
pub mod authorization;
pub mod blockchain;
pub mod blockchain_test;
pub mod dashboard;
pub mod email_verification;

pub mod epochs;
pub mod erc;
pub mod extractors;
pub mod governance;
pub mod health;
pub mod key_rotation;
pub mod market_data;
pub mod meter;
// pub mod meter_registration; // Removed
// pub mod meter_verification; // Removed
// pub use meter::*; // Ambiguous export
pub mod metrics;
pub mod oracle;
pub mod registry;
pub mod response;
pub mod swap;
pub mod token;
pub mod trading;
pub mod transactions;
pub mod user;
// pub use erc::*; // Ambiguous export
pub mod wallet_auth;
pub mod websocket;

// Re-export commonly used types
pub use authorization::{
    can_access_user_data, can_submit_meter_readings, can_trade, can_view_analytics, require_admin,
    require_admin_or_owner, require_any_role, require_role, roles,
};
pub use extractors::{DateRangeParams, PaginationParams, SearchParams, SortOrder, ValidatedUuid};
pub use response::{ApiResponse, ListResponse, PaginatedResponse};
