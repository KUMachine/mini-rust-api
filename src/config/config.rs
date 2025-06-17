#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let server_host = dotenvy::var("SERVER_HOST").expect("SERVER_HOST must be set");
        let server_port = dotenvy::var("SERVER_PORT")
            .unwrap()
            .parse()
            .expect("SERVER_PORT must be set");

        Self {
            database_url,
            server_host,
            server_port,
        }
    }
}
