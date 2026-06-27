use figment::{providers::{Env, Format, Yaml}, Figment};
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct OAuthProvider {
    pub client_id: String,
    pub client_secret: String,
    pub issuer_url: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct OAuthConfig {
    pub google: Option<OAuthProvider>,
    pub github: Option<OAuthProvider>,
    pub authentik: Option<OAuthProvider>,
}

#[derive(Clone, Deserialize)]
pub struct AiConfig {
    pub default_provider: String,
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
}

impl AiConfig {
    pub fn build_adapter(&self, _http: &reqwest::Client) -> anyhow::Result<Box<dyn crate::ai::provider::AiProviderAdapter>> {
        match self.default_provider.as_str() {
            "anthropic" => {
                let key = self.anthropic_api_key.clone()
                    .ok_or_else(|| anyhow::anyhow!("ANTHROPIC_API_KEY not configured"))?;
                Ok(Box::new(crate::ai::anthropic::AnthropicAdapter::new(key)))
            }
            p => Err(anyhow::anyhow!("unknown AI provider: {p}")),
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub listen_addr: String,
    pub app_base_url: String,
    pub trusted_proxy_cidrs: Vec<String>,
    pub session_secret: String,
    pub secret_encryption_key: String,
    pub bootstrap_super_admin_email: Option<String>,
    pub oauth: OAuthConfig,
    pub ai: AiConfig,
    /// OTLP HTTP endpoint, e.g. http://otel-collector:4318/v1/traces
    pub otel_endpoint: Option<String>,
    /// Sentry DSN for error reporting
    pub sentry_dsn: Option<String>,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let cfg: Self = Figment::new()
            .merge(Yaml::file("config/default.yaml"))
            .merge(Env::raw().split("__"))
            .extract()?;
        Ok(cfg)
    }
}
