pub mod cloud;
pub mod config;
pub mod errors;
pub mod handlers;
pub mod middlewares;
pub mod models;
pub mod repositories;
pub mod routers;
pub mod startup;
pub mod tests;
pub mod utils;
pub struct AppState {
    pub database: mongodb::Database,
}
