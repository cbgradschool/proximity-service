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
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        //Hierarchial/Layered config composition.
        let s = Config::builder()
            // Load from defaults configs.
            .add_source(File::with_name("config/default"))
            // Load from "env" based configs.
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Load from ENVIRONMENT
            .add_source(Environment::default())
            .build()?;

        // Deserialize and freeze configs as is.
        s.try_deserialize()
    }
}
