use sea_orm::DatabaseConnection;
use tera::Tera;

pub use actix_session::Session;
use redis::aio::ConnectionManager;
pub use serde::{Deserialize, Serialize};

pub use validator::{Validate, ValidationError};

#[derive(Clone)]
pub struct AppState {
    pub view: Tera,
    pub db: DatabaseConnection,
    pub redis: ConnectionManager,
}
