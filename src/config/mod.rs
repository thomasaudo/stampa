use std::fmt::{self};

use dotenv::dotenv;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: i32,
    pub database_url: String,
    pub database_name: String,
}

impl Config {
    pub fn from_env() -> Result<Config, Box<dyn std::error::Error>> {
        dotenv().ok();
        let mut config = config::Config::new();
        config.merge(config::Environment::default())?;
        config
            .try_into()
            .map_err(|_| "Failed to load configuration from environment.".into())
    }

    pub async fn connect_mongo(&self) -> Result<mongodb::Database, Box<dyn std::error::Error>> {
        let client = mongodb::Client::with_uri_str(&self.database_url)
            .await
            .expect("Failed to connect to MongoDB.");
        Ok(client.database(&self.database_name))
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}",
            self.host, self.port, self.database_url, self.database_name
        )
    }
}
