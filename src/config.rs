use std::env;
use std::sync::OnceLock;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

pub struct AppConfig {
	pub schema: String,
	pub host: String,
	pub port: String,
	pub rathole_image: String,
	pub rathole_config_path: String,
	pub rathole_config_name: String,
}

pub fn initialize_config() -> &'static AppConfig {
	CONFIG.get_or_init(|| AppConfig {
		schema: env::var("OPERATOR_SCHEMA").unwrap_or_else(|_| "http".to_string()),
		host: env::var("OPERATOR_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
		port: env::var("OPERATOR_PORT").unwrap_or_else(|_| "8080".to_string()),
		rathole_image: env::var("RATHOLE_IMAGE")
			.unwrap_or_else(|_| "rapiz1/rathole:v0.5.0".to_string()),
		rathole_config_path: env::var("RATHOLE_CONFIG_PATH").unwrap_or_else(|_| "/tmp".to_string()),
		rathole_config_name: env::var("RATHOLE_CONFIG_PATH")
			.unwrap_or_else(|_| "config.toml".to_string()),
	})
}
