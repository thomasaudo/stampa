pub mod cloud;
pub mod config;
pub mod database;
pub mod errors;
pub mod handlers;
pub mod image;
pub mod models;
pub mod utils;
pub struct AppState {
    pub database: mongodb::Database,
}
