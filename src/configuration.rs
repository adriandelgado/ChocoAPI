use eyre::{Result, WrapErr};
use secrecy::{ExposeSecret, SecretString};
use sqlx::{
    postgres::{PgConnectOptions, PgSslMode},
    ConnectOptions,
};
use std::{
    convert::{TryFrom, TryInto},
    net::IpAddr,
};

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub redis: RedisSettings,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: IpAddr,
    pub base_url: String,
}

// TODO: We may want to connect to a different database for testing.
#[derive(serde::Deserialize, Clone, Debug)]
pub struct RedisSettings {
    pub host: IpAddr,
    pub port: u16,
}

/// Return a connection object to send commands to Redis.
pub async fn get_redis_client(config: &RedisSettings) -> redis::Client {
    let conn_url = redis::parse_redis_url(&format!("redis://{}:{}", config.host, config.port))
        .expect("failed to build redis connection url");
    redis::Client::open(conn_url).expect("failed to create redis client")
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    #[must_use]
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    #[must_use]
    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);
        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }
}

pub fn extract(environment: Environment) -> Result<Settings> {
    let base_path =
        std::env::current_dir().wrap_err("failed to determine the current directory")?;
    let configuration_directory = base_path.join("configuration");

    // Initialise our configuration reader
    let settings = config::Config::builder()
        .add_source(config::File::from(configuration_directory.join("base")).required(true))
        .add_source(
            config::File::from(configuration_directory.join(environment.as_str())).required(true),
        )
        // Add in settings from environment variables (with a prefix of APP and '__' as separator)
        // E.g. `APP__DATABASE__PORT=5432 would set `Settings.database.port`
        .add_source(config::Environment::with_prefix("app").separator("__"))
        .build()?;

    // Try to convert the configuration values it read into
    // our Settings type
    settings
        .try_deserialize()
        .wrap_err("failed to deserialize config files")
}

/// Detect the running environment.
/// Default to `Environment::Local` if unspecified.
pub fn get_environment() -> Result<Environment> {
    std::env::var("APP_ENVIRONMENT").map_or(Ok(Environment::Local), |s| {
        s.try_into().wrap_err("failed to parse APP_ENVIRONMENT")
    })
}

/// The possible runtime environment for our application.
#[derive(Debug, Clone, Copy)]
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = eyre::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(eyre::eyre!(
                "{other} is not a supported environment. Use either `local` or `production`"
            )),
        }
    }
}
