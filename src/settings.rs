use config::{Config, ConfigError, Environment, File};
use serde_derive::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Settings {
    pub database_url: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        /*
         * Hierarchial/Layered config composition.
         * 1. Load from defaults configs.
         * 2. Load from "env" based configs.
         * 3. Load from ENVIRONMENT
         */
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let s = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .add_source(Environment::default())
            .build()?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }
}
