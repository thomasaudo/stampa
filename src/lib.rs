pub mod cloud;
pub mod config;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod utils;
pub struct AppState {
    pub database: mongodb::Database,
}
