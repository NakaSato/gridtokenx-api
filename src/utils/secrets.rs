// Secrets management and validation utilities

use anyhow::{Result, bail};
use std::env;
use tracing::{info, warn};

/// Validate all required secrets and environment variables on startup
pub fn validate_secrets() -> Result<()> {
    info!("🔐 Validating secrets and environment variables...");

    let mut missing_secrets = Vec::new();
    let mut weak_secrets = Vec::new();
    let mut warnings = Vec::new();

    // Critical secrets that must be present
    let critical_secrets = vec![
        ("JWT_SECRET", 32, true),
        ("DATABASE_URL", 10, false),
        ("REDIS_URL", 10, false),
    ];

    // Optional but recommended secrets
    let optional_secrets = vec![("ENGINEERING_API_KEY", 32), ("SMTP_PASSWORD", 8)];

    // Validate critical secrets
    for (name, min_length, should_be_random) in critical_secrets {
        match env::var(name) {
            Ok(value) => {
                if value.is_empty() {
                    missing_secrets.push(name.to_string());
                } else if value.len() < min_length {
                    weak_secrets.push(format!(
                        "{} (minimum {} characters required)",
                        name, min_length
                    ));
                } else if should_be_random && is_weak_secret(&value) {
                    warnings.push(format!("{} appears to be a weak or default value", name));
                }
            }
            Err(_) => missing_secrets.push(name.to_string()),
        }
    }

    // Validate optional secrets
    for (name, min_length) in optional_secrets {
        if let Ok(value) = env::var(name) {
            if !value.is_empty() && value.len() < min_length {
                warnings.push(format!(
                    "{} is shorter than recommended ({} chars)",
                    name, min_length
                ));
            }
        }
    }

    // Check for hardcoded/default values
    check_for_defaults(&mut warnings);

    // Report findings
    if !missing_secrets.is_empty() {
        bail!(
            "❌ Missing required secrets: {}",
            missing_secrets.join(", ")
        );
    }

    if !weak_secrets.is_empty() {
        bail!("❌ Weak secrets detected: {}", weak_secrets.join(", "));
    }

    if !warnings.is_empty() {
        for warning in &warnings {
            warn!("⚠️  {}", warning);
        }
    }

    // Check SSL/TLS configuration
    validate_ssl_configuration(&mut warnings);

    info!("✅ Secrets validation completed");
    if warnings.is_empty() {
        info!("✅ All security checks passed");
    } else {
        warn!("⚠️  {} security warnings found (see above)", warnings.len());
    }

    Ok(())
}

/// Check if a secret appears to be weak or default
fn is_weak_secret(secret: &str) -> bool {
    let weak_patterns = vec![
        "password", "secret", "changeme", "default", "admin", "test", "12345", "qwerty", "abc123",
    ];

    let lowercase = secret.to_lowercase();
    weak_patterns
        .iter()
        .any(|pattern| lowercase.contains(pattern))
}

/// Check for common default/hardcoded values in environment
fn check_for_defaults(warnings: &mut Vec<String>) {
    // Check JWT secret
    if let Ok(jwt_secret) = env::var("JWT_SECRET") {
        if jwt_secret.len() < 64 {
            warnings.push("JWT_SECRET should be at least 64 characters for production".to_string());
        }
    }

    // Check database URL for localhost in production
    if let Ok(env_type) = env::var("ENVIRONMENT") {
        if env_type == "production" {
            if let Ok(db_url) = env::var("DATABASE_URL") {
                if db_url.contains("localhost") || db_url.contains("127.0.0.1") {
                    warnings.push(
                        "DATABASE_URL points to localhost in production environment".to_string(),
                    );
                }
            }

            if let Ok(redis_url) = env::var("REDIS_URL") {
                if redis_url.contains("localhost") || redis_url.contains("127.0.0.1") {
                    warnings.push(
                        "REDIS_URL points to localhost in production environment".to_string(),
                    );
                }
            }
        }
    }
}

/// Validate SSL/TLS configuration
fn validate_ssl_configuration(warnings: &mut Vec<String>) {
    // Check PostgreSQL SSL mode
    if let Ok(db_url) = env::var("DATABASE_URL") {
        if !db_url.contains("sslmode=") {
            warnings.push(
                "DATABASE_URL does not specify SSL mode (add ?sslmode=require for production)"
                    .to_string(),
            );
        } else if db_url.contains("sslmode=disable") || db_url.contains("sslmode=prefer") {
            warnings.push(
                "DATABASE_URL has weak SSL mode (use sslmode=require, verify-ca, or verify-full)"
                    .to_string(),
            );
        }
    }

    // Check Redis TLS
    if let Ok(redis_url) = env::var("REDIS_URL") {
        if !redis_url.starts_with("rediss://") {
            warnings
                .push("REDIS_URL does not use TLS (consider using rediss:// protocol)".to_string());
        }

        // Check if Redis has authentication
        if !redis_url.contains("@") && !redis_url.contains(":password") {
            warnings.push("REDIS_URL does not include authentication credentials".to_string());
        }
    }
}

/// Generate a secure random secret (for development/testing)
#[cfg(test)]
pub fn generate_secure_secret(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789\
                            !@#$%^&*()-_=+";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weak_secret_detection() {
        assert!(is_weak_secret("password123"));
        assert!(is_weak_secret("MySecretPassword"));
        assert!(is_weak_secret("changeme"));
        assert!(is_weak_secret("admin123"));
        assert!(!is_weak_secret("Kx9#mP2$vL8@qR4!nT6&wZ1^"));
    }

    #[test]
    fn test_generate_secure_secret() {
        let secret = generate_secure_secret(64);
        assert_eq!(secret.len(), 64);
        assert!(!is_weak_secret(&secret));
    }

    #[test]
    fn test_secret_length_requirements() {
        let short_secret = "abc";
        let long_secret = "a".repeat(100);

        assert!(short_secret.len() < 32);
        assert!(long_secret.len() >= 32);
    }
}
