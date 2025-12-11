//! Services module - Business logic and external integrations

// Core services
pub mod amm;
// pub mod amm_service_test;
pub mod audit_logger;
pub mod auth;
pub mod cache;
pub mod dashboard;
pub mod email;
pub mod epoch_scheduler;
pub mod erc;
pub mod event_processor;
pub mod health_check;
pub mod key_rotation;
pub mod market_clearing;
pub mod order_matching_engine;
pub mod priority_fee;
pub mod settlement;
pub mod token;
pub mod validation;
pub mod wallet;
pub mod webhook;
pub mod websocket;

// Modular services (reorganized into subdirectories)
pub mod blockchain;
pub mod market;
pub mod meter;
pub mod redis;
pub mod transaction;

// Validation submodule
// pub mod validation;

// Re-exports for backward compatibility
pub use amm::AmmService;
pub use audit_logger::AuditLogger;
pub use auth::AuthService;
pub use blockchain::BlockchainService;
pub use cache::CacheService;
pub use dashboard::DashboardService;
pub use email::EmailService;
pub use epoch_scheduler::{EpochConfig, EpochScheduler};
pub use erc::ErcService;
pub use event_processor::EventProcessorService;
pub use health_check::HealthChecker;
pub use key_rotation::{KeyRotationService, RotationReport, RotationStatus};
pub use market::MarketClearingEngine;
pub use market_clearing::MarketClearingService;
pub use meter::polling::MeterPollingService;
pub use meter::service::MeterService;
pub use meter::verification::MeterVerificationService;
pub use order_matching_engine::OrderMatchingEngine;
pub use priority_fee::PriorityFeeService;
// pub use redis::*;
pub use event_processor::EventType;
pub use redis::json::RedisJSONService;
pub use redis::lock::{LockInfo, RedisLock};
pub use redis::pubsub::RedisPubSubService as RedisPubSub;
pub use settlement::{SettlementConfig, SettlementService};
pub use token::TokenService;
// pub use token::TokenService;
pub use webhook::WebhookService;
// pub use transaction::*;
pub use transaction::metrics::TransactionMetrics;
pub use transaction::service::TransactionService;
// pub use wallet::*;
pub use validation::TransactionValidationService as ValidationService;
pub use wallet::audit_logger::{WalletAuditEntry, WalletAuditLogger};
pub use wallet::initialization::{WalletInitializationService, WalletStatus};
pub use wallet::WalletService;
pub use websocket::WebSocketService;

// Re-export common types if needed
pub use audit_logger::{AuditEvent, AuditEventRecord};
pub use event_processor::EventProcessorStats;
pub use market::ClearingPrice;
// pub use transaction::TransactionMetrics; // Already exported above
