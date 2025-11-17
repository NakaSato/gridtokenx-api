// Business logic services
// Authentication, blockchain client, trading engine, etc.

pub mod audit_logger;
// pub mod batch_transaction_service; // Temporarily disabled - depends on market_clearing_service
pub mod blockchain_service;
// pub mod cache_service; // Temporarily disabled - compilation errors need fixing
pub mod email_service;
pub mod email_templates;
pub mod erc_service;
pub mod epoch_scheduler;
pub mod health_check;
pub mod market_clearing;
pub mod market_clearing_service;
pub mod meter_service;
// pub mod order_matcher; // Temporarily disabled due to type compatibility issues
pub mod order_matching_engine;
pub mod settlement_service;
// pub mod settlement_blockchain_service; // Temporarily disabled - depends on market_clearing_service
pub mod token_service;
pub mod transaction_service;
pub mod wallet_service;
pub mod websocket_service;

pub use audit_logger::{AuditLogger, AuditEvent, AuditEventRecord};
// pub use batch_transaction_service::{
//     BatchTransactionService, BatchConfig, TransactionBatch, BatchStatus,
//     TransactionPriority, BatchStatistics
// };
pub use blockchain_service::BlockchainService;
// pub use cache_service::{CacheService, CacheStats}; // Temporarily disabled
pub use email_service::EmailService;
pub use erc_service::ErcService;
pub use epoch_scheduler::{EpochScheduler, EpochConfig, EpochTransitionEvent};
pub use health_check::{HealthChecker, DetailedHealthStatus, DependencyHealth, HealthCheckStatus, SystemMetrics};
pub use market_clearing::{MarketClearingEngine, OrderBook, OrderBookSnapshot, BookOrder, TradeMatch, ClearingPrice};
pub use market_clearing_service::{MarketClearingService, MarketEpoch, OrderMatch, Settlement as EpochSettlement, OrderBookEntry};
pub use meter_service::MeterService;
pub use order_matching_engine::OrderMatchingEngine;
pub use settlement_service::{
    SettlementService, Settlement, SettlementStatus, SettlementTransaction,
    SettlementConfig, SettlementStats
};
// pub use order_matcher::{ // Temporarily disabled due to type compatibility issues
//     OrderMatcher, OrderMatcherConfig, MatchingResult, MatchingMetrics,
//     BuyOrder, SellOrder
// };
// pub use settlement_blockchain_service::{
//     SettlementBlockchainService, SettlementTransaction, SettlementTransactionStatus
// };
pub use token_service::TokenService;
pub use transaction_service::{
    TransactionService, TransactionMonitor, TransactionRetryService,
    TransactionSubmissionResult, TransactionStatus, TradingTransactionResult, MintTransactionResult,
    BatchTransaction, RetryConfig
};
pub use wallet_service::WalletService;
pub use websocket_service::WebSocketService;
