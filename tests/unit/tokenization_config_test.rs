use gridtokenx_apigateway::config::{TokenizationConfig, ValidationError};

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = TokenizationConfig::default();

        assert_eq!(config.kwh_to_token_ratio, 1.0);
        assert_eq!(config.decimals, 9);
        assert_eq!(config.max_reading_kwh, 100.0);
        assert_eq!(config.reading_max_age_days, 7);
        assert!(config.auto_mint_enabled);
        assert_eq!(config.polling_interval_secs, 60);
        assert_eq!(config.batch_size, 50);
        assert_eq!(config.max_retry_attempts, 3);
        assert_eq!(config.initial_retry_delay_secs, 300);
        assert_eq!(config.retry_backoff_multiplier, 2.0);
        assert_eq!(config.max_retry_delay_secs, 3600);
        assert_eq!(config.transaction_timeout_secs, 60);
        assert_eq!(config.max_transactions_per_batch, 20);
    }

    #[test]
    fn test_kwh_to_tokens_conversion() {
        let config = TokenizationConfig::default();

        // Basic conversion: 1 kWh = 1 token with 9 decimals
        assert_eq!(config.kwh_to_tokens(1.0).unwrap(), 1_000_000_000);

        // Zero amount
        assert_eq!(config.kwh_to_tokens(0.0).unwrap(), 0);

        // Fractional amount
        assert_eq!(config.kwh_to_tokens(0.5).unwrap(), 500_000_000);

        // Larger amount
        assert_eq!(config.kwh_to_tokens(10.0).unwrap(), 10_000_000_000);
    }

    #[test]
    fn test_kwh_to_tokens_with_different_ratio() {
        let mut config = TokenizationConfig::default();
        config.kwh_to_token_ratio = 2.5;
        config.decimals = 9;

        // 1 kWh should equal 2.5 tokens with 9 decimals
        assert_eq!(config.kwh_to_tokens(1.0).unwrap(), 2_500_000_000);

        // 10 kWh should equal 25 tokens with 9 decimals
        assert_eq!(config.kwh_to_tokens(10.0).unwrap(), 25_000_000_000);
    }

    #[test]
    fn test_kwh_to_tokens_with_different_decimals() {
        let mut config = TokenizationConfig::default();
        config.kwh_to_token_ratio = 1.0;
        config.decimals = 6; // Change to 6 decimals

        // 1 kWh should equal 1 token with 6 decimals
        assert_eq!(config.kwh_to_tokens(1.0).unwrap(), 1_000_000);
    }

    #[test]
    fn test_kwh_to_tokens_negative_amount() {
        let config = TokenizationConfig::default();

        // Negative amount should return error
        assert!(matches!(
            config.kwh_to_tokens(-1.0),
            Err(ValidationError::NegativeAmount)
        ));
    }

    #[test]
    fn test_kwh_to_tokens_exceeds_max() {
        let config = TokenizationConfig::default();

        // Amount exceeding max_reading_kwh should return error
        assert!(matches!(
            config.kwh_to_tokens(1000.0),
            Err(ValidationError::AmountTooHigh(_))
        ));
    }

    #[test]
    fn test_tokens_to_kwh_conversion() {
        let config = TokenizationConfig::default();

        // Basic conversion: 1 token with 9 decimals = 1 kWh
        assert_eq!(config.tokens_to_kwh(1_000_000_000), 1.0);

        // Zero amount
        assert_eq!(config.tokens_to_kwh(0), 0.0);

        // Larger amount
        assert_eq!(config.tokens_to_kwh(10_000_000_000), 10.0);

        // Fractional amount
        assert_eq!(config.tokens_to_kwh(500_000_000), 0.5);
    }

    #[test]
    fn test_tokens_to_kwh_with_different_ratio() {
        let mut config = TokenizationConfig::default();
        config.kwh_to_token_ratio = 2.5;
        config.decimals = 9;

        // 2.5 tokens with 9 decimals should equal 1 kWh
        assert_eq!(config.tokens_to_kwh(2_500_000_000), 1.0);

        // 1 token with 9 decimals should equal 0.4 kWh
        assert_eq!(config.tokens_to_kwh(1_000_000_000), 0.4);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = TokenizationConfig::default();

        // First attempt (0) should return 0
        assert_eq!(config.calculate_retry_delay(0), 0);

        // Initial retry (1) should return initial_delay
        assert_eq!(
            config.calculate_retry_delay(1),
            config.initial_retry_delay_secs
        );

        // Subsequent retries should increase exponentially
        let expected_2 = config.initial_retry_delay_secs * 2;
        assert_eq!(config.calculate_retry_delay(2), expected_2);

        let expected_3 = config.initial_retry_delay_secs * 4;
        assert_eq!(config.calculate_retry_delay(3), expected_3);

        let expected_4 = config.initial_retry_delay_secs * 8;
        assert_eq!(config.calculate_retry_delay(4), expected_4);
    }

    #[test]
    fn test_retry_delay_max_limit() {
        let mut config = TokenizationConfig::default();
        config.initial_retry_delay_secs = 100;
        config.max_retry_delay_secs = 500; // Lower than exponential would reach
        config.retry_backoff_multiplier = 2.0;

        // Should not exceed max_retry_delay_secs
        assert_eq!(config.calculate_retry_delay(1), 100);
        assert_eq!(config.calculate_retry_delay(2), 200);
        assert_eq!(config.calculate_retry_delay(3), 400);
        assert_eq!(config.calculate_retry_delay(4), 500); // Capped at max
        assert_eq!(config.calculate_retry_delay(5), 500); // Still capped
    }

    #[test]
    fn test_config_from_env_valid_values() {
        // Set environment variables
        env::set_var("TOKENIZATION_KWH_TO_TOKEN_RATIO", "2.5");
        env::set_var("TOKENIZATION_DECIMALS", "18");
        env::set_var("TOKENIZATION_MAX_READING_KWH", "200.0");
        env::set_var("TOKENIZATION_READING_MAX_AGE_DAYS", "14");
        env::set_var("TOKENIZATION_AUTO_MINT_ENABLED", "false");
        env::set_var("TOKENIZATION_POLLING_INTERVAL_SECS", "30");
        env::set_var("TOKENIZATION_BATCH_SIZE", "100");
        env::set_var("TOKENIZATION_MAX_RETRY_ATTEMPTS", "5");
        env::set_var("TOKENIZATION_INITIAL_RETRY_DELAY_SECS", "60");
        env::set_var("TOKENIZATION_RETRY_BACKOFF_MULTIPLIER", "3.0");
        env::set_var("TOKENIZATION_MAX_RETRY_DELAY_SECS", "1800");
        env::set_var("TOKENIZATION_TRANSACTION_TIMEOUT_SECS", "30");
        env::set_var("TOKENIZATION_MAX_TRANSACTIONS_PER_BATCH", "50");

        // Load config
        let config = TokenizationConfig::from_env().unwrap();

        // Verify values
        assert_eq!(config.kwh_to_token_ratio, 2.5);
        assert_eq!(config.decimals, 18);
        assert_eq!(config.max_reading_kwh, 200.0);
        assert_eq!(config.reading_max_age_days, 14);
        assert!(!config.auto_mint_enabled);
        assert_eq!(config.polling_interval_secs, 30);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.max_retry_attempts, 5);
        assert_eq!(config.initial_retry_delay_secs, 60);
        assert_eq!(config.retry_backoff_multiplier, 3.0);
        assert_eq!(config.max_retry_delay_secs, 1800);
        assert_eq!(config.transaction_timeout_secs, 30);
        assert_eq!(config.max_transactions_per_batch, 50);

        // Clean up
        env::remove_var("TOKENIZATION_KWH_TO_TOKEN_RATIO");
        env::remove_var("TOKENIZATION_DECIMALS");
        env::remove_var("TOKENIZATION_MAX_READING_KWH");
        env::remove_var("TOKENIZATION_READING_MAX_AGE_DAYS");
        env::remove_var("TOKENIZATION_AUTO_MINT_ENABLED");
        env::remove_var("TOKENIZATION_POLLING_INTERVAL_SECS");
        env::remove_var("TOKENIZATION_BATCH_SIZE");
        env::remove_var("TOKENIZATION_MAX_RETRY_ATTEMPTS");
        env::remove_var("TOKENIZATION_INITIAL_RETRY_DELAY_SECS");
        env::remove_var("TOKENIZATION_RETRY_BACKOFF_MULTIPLIER");
        env::remove_var("TOKENIZATION_MAX_RETRY_DELAY_SECS");
        env::remove_var("TOKENIZATION_TRANSACTION_TIMEOUT_SECS");
        env::remove_var("TOKENIZATION_MAX_TRANSACTIONS_PER_BATCH");
    }

    #[test]
    fn test_config_from_env_invalid_values() {
        // Test invalid kWh to token ratio (zero)
        env::set_var("TOKENIZATION_KWH_TO_TOKEN_RATIO", "0");
        assert!(TokenizationConfig::from_env().is_err());
        env::remove_var("TOKENIZATION_KWH_TO_TOKEN_RATIO");

        // Test invalid decimals (> 18)
        env::set_var("TOKENIZATION_DECIMALS", "20");
        assert!(TokenizationConfig::from_env().is_err());
        env::remove_var("TOKENIZATION_DECIMALS");

        // Test invalid max reading kWh (zero)
        env::set_var("TOKENIZATION_MAX_READING_KWH", "0");
        assert!(TokenizationConfig::from_env().is_err());
        env::remove_var("TOKENIZATION_MAX_READING_KWH");

        // Test invalid reading max age days (zero)
        env::set_var("TOKENIZATION_READING_MAX_AGE_DAYS", "0");
        assert!(TokenizationConfig::from_env().is_err());
        env::remove_var("TOKENIZATION_READING_MAX_AGE_DAYS");

        // Test invalid polling interval (< 10 when auto-mint is enabled)
        env::set_var("TOKENIZATION_POLLING_INTERVAL_SECS", "5");
        env::set_var("TOKENIZATION_AUTO_MINT_ENABLED", "true");
        assert!(TokenizationConfig::from_env().is_err());
        env::remove_var("TOKENIZATION_POLLING_INTERVAL_SECS");
        env::remove_var("TOKENIZATION_AUTO_MINT_ENABLED");

        // Test invalid batch size (zero)
        env::set_var("TOKENIZATION_BATCH_SIZE", "0");
        assert!(TokenizationConfig::from_env().is_err());
        env::remove_var("TOKENIZATION_BATCH_SIZE");

        // Test invalid max retry delay (< initial)
        env::set_var("TOKENIZATION_INITIAL_RETRY_DELAY_SECS", "300");
        env::set_var("TOKENIZATION_MAX_RETRY_DELAY_SECS", "200");
        assert!(TokenizationConfig::from_env().is_err());
        env::remove_var("TOKENIZATION_INITIAL_RETRY_DELAY_SECS");
        env::remove_var("TOKENIZATION_MAX_RETRY_DELAY_SECS");
    }

    #[test]
    fn test_config_from_env_uses_defaults() {
        // Don't set any environment variables, should use defaults
        let config = TokenizationConfig::from_env().unwrap();

        // Should match defaults
        let default_config = TokenizationConfig::default();
        assert_eq!(config.kwh_to_token_ratio, default_config.kwh_to_token_ratio);
        assert_eq!(config.decimals, default_config.decimals);
        assert_eq!(config.max_reading_kwh, default_config.max_reading_kwh);
        assert_eq!(
            config.reading_max_age_days,
            default_config.reading_max_age_days
        );
        assert_eq!(config.auto_mint_enabled, default_config.auto_mint_enabled);
        assert_eq!(
            config.polling_interval_secs,
            default_config.polling_interval_secs
        );
        assert_eq!(config.batch_size, default_config.batch_size);
        assert_eq!(config.max_retry_attempts, default_config.max_retry_attempts);
        assert_eq!(
            config.initial_retry_delay_secs,
            default_config.initial_retry_delay_secs
        );
        assert_eq!(
            config.retry_backoff_multiplier,
            default_config.retry_backoff_multiplier
        );
        assert_eq!(
            config.max_retry_delay_secs,
            default_config.max_retry_delay_secs
        );
        assert_eq!(
            config.transaction_timeout_secs,
            default_config.transaction_timeout_secs
        );
        assert_eq!(
            config.max_transactions_per_batch,
            default_config.max_transactions_per_batch
        );
    }

    #[test]
    fn test_edge_cases() {
        let config = TokenizationConfig::default();

        // Very small fractional value
        assert_eq!(config.kwh_to_tokens(0.000_000_001).unwrap(), 1);

        // Maximum value that fits in u64
        let max_kwh = u64::MAX as f64 / 1_000_000_000.0 - 0.001;
        let max_tokens = config.kwh_to_tokens(max_kwh).unwrap();
        assert!(max_tokens < u64::MAX);

        // Just above maximum
        let too_large_kwh = u64::MAX as f64 / 1_000_000_000.0 + 0.001;
        assert!(matches!(
            config.kwh_to_tokens(too_large_kwh),
            Err(ValidationError::AmountExceedsMaximum)
        ));
    }
}
