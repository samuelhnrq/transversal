use reqwest::Url;
use sea_orm::DatabaseConnection;

use crate::oauth::OpenIdConfiguration;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub db_url: String,
    pub port: u16,
    pub oauth: OpenIdConfiguration,
    pub oauth_client_id: String,
    pub oauth_autodiscover_url: String,
    pub oauth_client_secret: String,
    pub self_url: Url,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub requests: reqwest::Client,
    pub config: AppConfig,
}
