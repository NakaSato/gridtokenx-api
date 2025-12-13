//! Authentication service - Minimal version without audit logging

use crate::{
    auth::jwt::JwtService,
    config::Config,
    services::EmailService,
};
use sqlx::PgPool;

/// Service for handling authentication-related logic (minimal version)
#[derive(Clone)]
pub struct AuthService {
    db: PgPool,
    config: Config,
    email_service: Option<EmailService>,
    jwt_service: JwtService,
}

impl AuthService {
    pub fn new(
        db: PgPool,
        config: Config,
        email_service: Option<EmailService>,
        jwt_service: JwtService,
    ) -> Self {
        Self {
            db,
            config,
            email_service,
            jwt_service,
        }
    }

    /// Get the database pool
    pub fn db(&self) -> &PgPool {
        &self.db
    }

    /// Get the config
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get the JWT service
    pub fn jwt_service(&self) -> &JwtService {
        &self.jwt_service
    }
}
