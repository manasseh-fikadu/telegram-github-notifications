use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub telegram: TelegramConfig,
    pub github: GitHubConfig,
    pub routing: Vec<RouteConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TelegramConfig {
    pub bot_token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubConfig {
    pub webhook_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RouteConfig {
    pub repo_pattern: String,
    pub chat_id: i64,
    pub events: Vec<String>,
}

impl Settings {
    pub fn load() -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .add_source(config::Environment::with_prefix("APP"))
            .build()?;

        settings.try_deserialize().map_err(|e| anyhow::anyhow!(e))
    }
}
