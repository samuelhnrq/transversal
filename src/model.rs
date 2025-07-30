use models::DatabaseConnection;

#[derive(Clone, Debug)]
pub(crate) struct AppConfig {
    pub(crate) db_url: String,
    pub(crate) port: u16,
}

#[derive(Clone, Debug)]
pub(crate) struct AppState {
    pub(crate) db: DatabaseConnection,
    pub(crate) config: AppConfig,
}
