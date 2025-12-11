pub mod login;
pub mod profile;
pub mod users;

pub use login::*;
pub use profile::*;
pub use users::*;

use uuid::Uuid;

#[derive(sqlx::FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    #[allow(dead_code)]
    pub first_name: Option<String>,
    #[allow(dead_code)]
    pub last_name: Option<String>,
    pub wallet_address: Option<String>,
    #[allow(dead_code)]
    pub blockchain_registered: bool,
    #[allow(dead_code)]
    pub is_active: bool,
    pub email_verified: bool,
    #[allow(dead_code)]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[allow(dead_code)]
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
