use crate::error::{ApiError, ErrorCode};
use regex::Regex;
use once_cell::sync::Lazy;

/// Email validation regex
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .expect("Invalid email regex")
});

/// Wallet address validation regex (Solana: base58, 32-44 chars)
static WALLET_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[1-9A-HJ-NP-Za-km-z]{32,44}$")
        .expect("Invalid wallet regex")
});

/// Validation helper functions
pub struct Validator;

impl Validator {
    /// Validate email address
    pub fn validate_email(email: &str) -> Result<(), ApiError> {
        if email.is_empty() {
            return Err(ApiError::validation_field("email", "Email is required"));
        }

        if email.len() > 255 {
            return Err(ApiError::validation_field("email", "Email is too long"));
        }

        if !EMAIL_REGEX.is_match(email) {
            return Err(ApiError::with_code(
                ErrorCode::InvalidEmail,
                "Invalid email format"
            ));
        }

        Ok(())
    }

    /// Validate password strength
    pub fn validate_password(password: &str) -> Result<(), ApiError> {
        if password.is_empty() {
            return Err(ApiError::validation_field("password", "Password is required"));
        }

        if password.len() < 8 {
            return Err(ApiError::with_code(
                ErrorCode::PasswordTooWeak,
                "Password must be at least 8 characters"
            ));
        }

        if password.len() > 128 {
            return Err(ApiError::validation_field("password", "Password is too long"));
        }

        // Check for at least one letter and one number
        let has_letter = password.chars().any(|c| c.is_alphabetic());
        let has_number = password.chars().any(|c| c.is_numeric());

        if !has_letter || !has_number {
            return Err(ApiError::with_code(
                ErrorCode::PasswordTooWeak,
                "Password must contain at least one letter and one number"
            ));
        }

        Ok(())
    }

    /// Validate wallet address (Solana format)
    pub fn validate_wallet_address(address: &str) -> Result<(), ApiError> {
        if address.is_empty() {
            return Err(ApiError::validation_field("wallet_address", "Wallet address is required"));
        }

        if !WALLET_REGEX.is_match(address) {
            return Err(ApiError::with_code(
                ErrorCode::InvalidWalletAddress,
                "Invalid Solana wallet address format"
            ));
        }

        Ok(())
    }

    /// Validate amount (must be positive)
    pub fn validate_amount(amount: f64, field_name: &str) -> Result<(), ApiError> {
        if amount <= 0.0 {
            return Err(ApiError::validation_field(
                field_name,
                format!("{} must be greater than zero", field_name)
            ));
        }

        if amount.is_infinite() || amount.is_nan() {
            return Err(ApiError::validation_field(
                field_name,
                format!("{} must be a valid number", field_name)
            ));
        }

        Ok(())
    }

    /// Validate token amount (must be positive integer)
    pub fn validate_token_amount(amount: i64, field_name: &str) -> Result<(), ApiError> {
        if amount <= 0 {
            return Err(ApiError::validation_field(
                field_name,
                format!("{} must be greater than zero", field_name)
            ));
        }

        Ok(())
    }

    /// Validate energy reading (kWh, must be reasonable)
    pub fn validate_energy_reading(kwh: f64) -> Result<(), ApiError> {
        if kwh < 0.0 {
            return Err(ApiError::with_code(
                ErrorCode::MeterReadingInvalid,
                "Energy reading cannot be negative"
            ));
        }

        if kwh > 1000000.0 {
            return Err(ApiError::with_code(
                ErrorCode::MeterReadingInvalid,
                "Energy reading is unreasonably high (max 1,000,000 kWh)"
            ));
        }

        Ok(())
    }

    /// Validate price (must be positive and reasonable)
    pub fn validate_price(price: f64) -> Result<(), ApiError> {
        Self::validate_amount(price, "price")?;

        if price > 1000.0 {
            return Err(ApiError::validation_field(
                "price",
                "Price is unreasonably high (max 1000)"
            ));
        }

        Ok(())
    }

    /// Validate required string field
    pub fn validate_required_string(value: &str, field_name: &str) -> Result<(), ApiError> {
        if value.trim().is_empty() {
            return Err(ApiError::validation_field(
                field_name,
                format!("{} is required", field_name)
            ));
        }

        Ok(())
    }

    /// Validate string length
    pub fn validate_string_length(
        value: &str,
        field_name: &str,
        min: usize,
        max: usize,
    ) -> Result<(), ApiError> {
        let len = value.len();

        if len < min {
            return Err(ApiError::validation_field(
                field_name,
                format!("{} must be at least {} characters", field_name, min)
            ));
        }

        if len > max {
            return Err(ApiError::validation_field(
                field_name,
                format!("{} must be at most {} characters", field_name, max)
            ));
        }

        Ok(())
    }

    /// Validate UUID format
    pub fn validate_uuid(value: &str, field_name: &str) -> Result<(), ApiError> {
        uuid::Uuid::parse_str(value).map_err(|_| {
            ApiError::validation_field(
                field_name,
                format!("{} must be a valid UUID", field_name)
            )
        })?;

        Ok(())
    }

    /// Validate positive integer
    pub fn validate_positive_integer(value: i64, field_name: &str) -> Result<(), ApiError> {
        if value <= 0 {
            return Err(ApiError::validation_field(
                field_name,
                format!("{} must be a positive integer", field_name)
            ));
        }

        Ok(())
    }

    /// Validate date is not in the future
    pub fn validate_not_future_date(date: chrono::DateTime<chrono::Utc>, field_name: &str) -> Result<(), ApiError> {
        if date > chrono::Utc::now() {
            return Err(ApiError::validation_field(
                field_name,
                format!("{} cannot be in the future", field_name)
            ));
        }

        Ok(())
    }

    /// Validate date range (from_date must be before to_date)
    pub fn validate_date_range(
        from_date: Option<chrono::DateTime<chrono::Utc>>,
        to_date: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), ApiError> {
        if let (Some(from), Some(to)) = (from_date, to_date) {
            if from > to {
                return Err(ApiError::validation_error(
                    "from_date must be before to_date",
                    None
                ));
            }
        }

        Ok(())
    }

    /// Validate energy source
    pub fn validate_energy_source(source: &str) -> Result<(), ApiError> {
        let valid_sources = ["solar", "wind", "hydro", "mixed", "biomass", "geothermal"];
        
        if !valid_sources.contains(&source.to_lowercase().as_str()) {
            return Err(ApiError::validation_field(
                "energy_source",
                format!("Energy source must be one of: {}", valid_sources.join(", "))
            ));
        }

        Ok(())
    }

    /// Validate order side
    pub fn validate_order_side(side: &str) -> Result<(), ApiError> {
        let valid_sides = ["buy", "sell"];
        
        if !valid_sides.contains(&side.to_lowercase().as_str()) {
            return Err(ApiError::validation_field(
                "side",
                "Order side must be 'buy' or 'sell'"
            ));
        }

        Ok(())
    }

    /// Validate order status
    pub fn validate_order_status(status: &str) -> Result<(), ApiError> {
        let valid_statuses = ["pending", "partial", "filled", "cancelled", "expired"];
        
        if !valid_statuses.contains(&status.to_lowercase().as_str()) {
            return Err(ApiError::validation_field(
                "status",
                format!("Status must be one of: {}", valid_statuses.join(", "))
            ));
        }

        Ok(())
    }

    /// Validate transaction status
    pub fn validate_transaction_status(status: &str) -> Result<(), ApiError> {
        let valid_statuses = ["pending", "processing", "completed", "failed", "cancelled"];
        
        if !valid_statuses.contains(&status.to_lowercase().as_str()) {
            return Err(ApiError::validation_field(
                "status",
                format!("Status must be one of: {}", valid_statuses.join(", "))
            ));
        }

        Ok(())
    }

    /// Validate certificate status
    pub fn validate_certificate_status(status: &str) -> Result<(), ApiError> {
        let valid_statuses = ["active", "retired", "expired", "cancelled"];
        
        if !valid_statuses.contains(&status.to_lowercase().as_str()) {
            return Err(ApiError::validation_field(
                "status",
                format!("Status must be one of: {}", valid_statuses.join(", "))
            ));
        }

        Ok(())
    }

    /// Validate user role
    pub fn validate_user_role(role: &str) -> Result<(), ApiError> {
        let valid_roles = ["consumer", "producer", "prosumer", "admin", "rec"];
        
        if !valid_roles.contains(&role.to_lowercase().as_str()) {
            return Err(ApiError::validation_field(
                "role",
                format!("Role must be one of: {}", valid_roles.join(", "))
            ));
        }

        Ok(())
    }

    /// Validate percentage (0-100)
    pub fn validate_percentage(value: f64, field_name: &str) -> Result<(), ApiError> {
        if value < 0.0 || value > 100.0 {
            return Err(ApiError::validation_field(
                field_name,
                format!("{} must be between 0 and 100", field_name)
            ));
        }

        Ok(())
    }

    /// Validate coordinate (latitude/longitude)
    pub fn validate_latitude(lat: f64) -> Result<(), ApiError> {
        if lat < -90.0 || lat > 90.0 {
            return Err(ApiError::validation_field(
                "latitude",
                "Latitude must be between -90 and 90"
            ));
        }

        Ok(())
    }

    pub fn validate_longitude(lng: f64) -> Result<(), ApiError> {
        if lng < -180.0 || lng > 180.0 {
            return Err(ApiError::validation_field(
                "longitude",
                "Longitude must be between -180 and 180"
            ));
        }

        Ok(())
    }

    /// Validate BigDecimal amount (for database storage)
    pub fn validate_bigdecimal_amount(amount: &bigdecimal::BigDecimal, field_name: &str) -> Result<(), ApiError> {
        use bigdecimal::Zero;
        
        if amount.is_zero() || amount < &bigdecimal::BigDecimal::zero() {
            return Err(ApiError::validation_field(
                field_name,
                format!("{} must be greater than zero", field_name)
            ));
        }

        Ok(())
    }

    /// Validate username format
    pub fn validate_username(username: &str) -> Result<(), ApiError> {
        if username.is_empty() {
            return Err(ApiError::validation_field("username", "Username is required"));
        }

        if username.len() < 3 {
            return Err(ApiError::validation_field("username", "Username must be at least 3 characters"));
        }

        if username.len() > 50 {
            return Err(ApiError::validation_field("username", "Username must be at most 50 characters"));
        }

        // Only alphanumeric, underscore, hyphen allowed
        if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(ApiError::validation_field(
                "username",
                "Username can only contain letters, numbers, underscores, and hyphens"
            ));
        }

        Ok(())
    }

    /// Validate phone number (basic international format)
    pub fn validate_phone_number(phone: &str) -> Result<(), ApiError> {
        if phone.is_empty() {
            return Ok(()); // Optional field
        }

        // Remove common formatting characters
        let cleaned: String = phone.chars()
            .filter(|c| c.is_numeric() || *c == '+')
            .collect();

        if cleaned.len() < 10 || cleaned.len() > 15 {
            return Err(ApiError::validation_field(
                "phone_number",
                "Phone number must be between 10 and 15 digits"
            ));
        }

        Ok(())
    }

    /// Validate time range (hours, for queries)
    pub fn validate_time_range_hours(hours: i32) -> Result<(), ApiError> {
        if hours <= 0 {
            return Err(ApiError::validation_field("hours", "Hours must be positive"));
        }

        if hours > 8760 { // Max 1 year
            return Err(ApiError::validation_field("hours", "Hours cannot exceed 8760 (1 year)"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email() {
        // Valid emails
        assert!(Validator::validate_email("test@example.com").is_ok());
        assert!(Validator::validate_email("user.name+tag@example.co.uk").is_ok());

        // Invalid emails
        assert!(Validator::validate_email("").is_err());
        assert!(Validator::validate_email("invalid").is_err());
        assert!(Validator::validate_email("@example.com").is_err());
        assert!(Validator::validate_email("test@").is_err());
    }

    #[test]
    fn test_validate_password() {
        // Valid passwords
        assert!(Validator::validate_password("password123").is_ok());
        assert!(Validator::validate_password("MyPass123").is_ok());

        // Invalid passwords
        assert!(Validator::validate_password("").is_err()); // Empty
        assert!(Validator::validate_password("short1").is_err()); // Too short
        assert!(Validator::validate_password("nodigits").is_err()); // No numbers
        assert!(Validator::validate_password("12345678").is_err()); // No letters
    }

    #[test]
    fn test_validate_wallet_address() {
        // Valid Solana address (32-44 base58 chars)
        assert!(Validator::validate_wallet_address("GvPhiX9W1v3fj8WbN5D2TzzPwf1Kp1TfMg1e8KW1Pump").is_ok());

        // Invalid addresses
        assert!(Validator::validate_wallet_address("").is_err());
        assert!(Validator::validate_wallet_address("short").is_err());
        assert!(Validator::validate_wallet_address("0x1234567890").is_err()); // Ethereum format
    }

    #[test]
    fn test_validate_amount() {
        // Valid amounts
        assert!(Validator::validate_amount(10.5, "amount").is_ok());
        assert!(Validator::validate_amount(0.01, "amount").is_ok());

        // Invalid amounts
        assert!(Validator::validate_amount(0.0, "amount").is_err());
        assert!(Validator::validate_amount(-5.0, "amount").is_err());
    }

    #[test]
    fn test_validate_energy_reading() {
        // Valid readings
        assert!(Validator::validate_energy_reading(100.5).is_ok());
        assert!(Validator::validate_energy_reading(0.0).is_ok());

        // Invalid readings
        assert!(Validator::validate_energy_reading(-1.0).is_err());
        assert!(Validator::validate_energy_reading(2000000.0).is_err()); // Too high
    }

    #[test]
    fn test_validate_price() {
        // Valid prices
        assert!(Validator::validate_price(10.5).is_ok());
        assert!(Validator::validate_price(0.01).is_ok());

        // Invalid prices
        assert!(Validator::validate_price(0.0).is_err());
        assert!(Validator::validate_price(-5.0).is_err());
        assert!(Validator::validate_price(1500.0).is_err()); // Too high
    }
}
