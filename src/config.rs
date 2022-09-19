use once_cell::sync::Lazy;
use serde::Deserialize;
use std::time::Duration;

pub static CONF: Lazy<Config> = Lazy::new(|| {
    dotenv::dotenv().expect("dotenv");
    envy::from_env().expect("config")
});

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub teloxide_token: String,
    #[serde(with = "humantime_serde")]
    pub spam_token_lifetime: Duration,
    /// No more than `rate_limit_messages` per `rate_limit_duration`
    pub rate_limit_messages: usize,
    #[serde(with = "humantime_serde")]
    pub rate_limit_duration: Duration,
    #[serde(default)]
    pub journal_logging: bool,
    pub moderators: Vec<i64>,
}
