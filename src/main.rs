use actix_web::web;
use env_logger::Env;
use stampa::startup::run;
use std::net::TcpListener;

use stampa::AppState;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let app_config = stampa::config::Config::from_env().unwrap();

    let database = app_config.connect_mongo().await.unwrap();

    let app_state = web::Data::new(AppState { database });

    let address = format!("{}:{}", app_config.host, app_config.port);
    let listener = TcpListener::bind(address.to_string())?;

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    run(listener, app_state)?.await
}
