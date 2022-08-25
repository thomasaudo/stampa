use std::net::TcpListener;

use actix_web::web::Data;
use mongodb::Database;
use uuid::Uuid;

use crate::{startup::run, AppState};

pub struct TestApp {
    pub address: String,
    pub database: Database,
}

pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut configuration = crate::config::Config::load_test_configuration().unwrap();
    configuration.database_name = Uuid::new_v4().to_string();
    let database = configuration.connect_mongo().await.unwrap();
    let app_state = Data::new(AppState {
        database: database.clone(),
    });

    let server = run(listener, app_state).expect("Failed to bind address");

    let _ = tokio::spawn(server);

    TestApp { address, database }
}

#[tokio::test]
async fn register_user() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let mut map = std::collections::HashMap::new();
    map.insert("username", "test-username");
    map.insert("password", "test-password");

    let response = client
        .post(&format!("{}/login", &app.address))
        .json(&map)
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn login_user() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let mut map = std::collections::HashMap::new();
    map.insert("username", "test-username");
    map.insert("password", "test-password");

    let response = client
        .post(&format!("{}/login", &app.address))
        .json(&map)
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(200, response.status().as_u16());
}
