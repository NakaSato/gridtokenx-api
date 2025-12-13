//! Handlers module - Minimal build for testing Simulator → Gateway → Anchor flow

// Core handlers that don't use disabled services
pub mod blockchain;
pub mod blockchain_test;
pub mod extractors;
pub mod response;
pub mod websocket;

// Minimal meter handler
pub mod meter_stub;

// Re-export commonly used types
pub use extractors::{DateRangeParams, PaginationParams, SearchParams, SortOrder, ValidatedUuid};
pub use response::{ApiResponse, ListResponse, PaginatedResponse};
