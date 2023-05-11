use config::{Config, ConfigError, Environment, File};
use dotenvy::dotenv;
use serde_derive::Deserialize;
use std::env;

#[derive(Clone, Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub db_max_connections: u32,
    pub honeycomb_api_key: String,
    pub honeycomb_dataset: String,
    pub honeycomb_host: String,
    pub honeycomb_port: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        dotenv().ok();

        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        //Hierarchial/Layered config composition.
        let s = Config::builder()
            // Load from defaults configs.
            .add_source(File::with_name("config/default"))
            // Load from local configs. NOT CHECKED INTO VERSION CONTROL
            .add_source(File::with_name("config/local").required(false))
            // Load from "env" based configs.
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Load from ENVIRONMENT
            .add_source(Environment::default())
            .build()?;

        // Deserialize and freeze configs as is.
        s.try_deserialize()
    }
}
